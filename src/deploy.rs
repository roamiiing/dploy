use anyhow::Result;
use bollard::{
    container::{self, StartContainerOptions},
    image,
    secret::{ContainerInspectResponse, ImageInspect},
    Docker,
};
use futures_util::TryStreamExt;

use std::{
    collections::{BTreeMap, HashSet},
    io::Write,
    path::Path,
};

use crate::{
    build::build_app_service_image,
    context::Context,
    presentation,
    services::{app::AppService, Services, ToContainerConfig},
};

const ENV_FILE_NAME: &str = ".env";

pub async fn deploy(context: &Context, docker: &Docker) -> Result<()> {
    if dotenvy::from_path(ENV_FILE_NAME).is_ok() {
        presentation::print_env_file_loaded();
    } else {
        presentation::print_env_file_failed_to_load();
    }

    let services = Services::from_context(context);

    if context.should_generate_env_file() {
        presentation::print_env_file_generating();
        generate_env(&services, context)?;
    }

    presentation::print_dependencies_starting();
    deploy_dependencies(&services, context, docker).await?;

    if let Some(service) = services.app() {
        deploy_app_service(service, context, docker).await?;
    }

    if context.should_print_connection_info() {
        let connection_info = services.connection_info();
        presentation::print_connection_info(&connection_info);
    }

    Ok(())
}

pub async fn stop(context: &Context, docker: &Docker) -> Result<()> {
    let services = Services::from_context(context);

    presentation::print_dependencies_stopping();
    stop_dependencies(&services, context, docker).await?;

    if let Some(service) = services.app() {
        stop_app_service(service, context, docker).await?;
    }

    Ok(())
}

async fn deploy_app_service(
    app_service: &AppService,
    context: &Context,
    docker: &Docker,
) -> Result<()> {
    let container_config = app_service.to_container_config(context)?;
    let container_name = container_config.container_name();

    presentation::print_image_building(container_name);
    build_app_service_image(app_service, docker).await?;
    presentation::print_image_built(container_name);

    let existing_container = match docker.inspect_container(container_name, None).await {
        Ok(container) => Some(container),
        Err(bollard::errors::Error::DockerResponseServerError {
            status_code: 404, ..
        }) => None,
        Err(e) => return Err(e.into()),
    };

    if existing_container.is_some() {
        presentation::print_app_container_removing(container_name);
        docker.stop_container(container_name, None).await?;
        docker.remove_container(container_name, None).await?;
    }

    presentation::print_app_container_creating(container_name);
    docker
        .create_container(
            Some(container::CreateContainerOptions {
                name: container_name,
                ..Default::default()
            }),
            container_config.config().clone(),
        )
        .await?;

    presentation::print_app_container_starting(container_name);
    docker
        .start_container(container_name, None::<StartContainerOptions<String>>)
        .await?;

    presentation::print_app_container_success(container_name);

    Ok(())
}

async fn stop_app_service(
    app_service: &AppService,
    context: &Context,
    docker: &Docker,
) -> Result<()> {
    let container_config = app_service.to_container_config(context)?;
    let container_name = container_config.container_name();

    presentation::print_app_container_removing(container_name);

    let existing_container = match docker.inspect_container(container_name, None).await {
        Ok(container) => Some(container),
        Err(bollard::errors::Error::DockerResponseServerError {
            status_code: 404, ..
        }) => None,
        Err(e) => return Err(e.into()),
    };

    if should_stop_container(existing_container.as_ref()) {
        docker.stop_container(container_name, None).await?;
        presentation::print_app_container_stopped(container_name);
    } else {
        presentation::print_app_container_already_stopped(container_name);
    }

    Ok(())
}

async fn stop_dependencies(services: &Services, context: &Context, docker: &Docker) -> Result<()> {
    let container_configs = services.to_container_configs(&context)?;

    for config in container_configs {
        let container_name = config.container_name();

        let existing_container = match docker.inspect_container(container_name, None).await {
            Ok(container) => Some(container),
            Err(bollard::errors::Error::DockerResponseServerError {
                status_code: 404, ..
            }) => None,
            Err(e) => return Err(e.into()),
        };

        presentation::print_dependency_stopping(container_name);
        if should_stop_container(existing_container.as_ref()) {
            docker.stop_container(container_name, None).await?;
            docker.remove_container(container_name, None).await?;
            presentation::print_dependency_stopped(container_name);
        } else {
            presentation::print_dependency_already_stopped(container_name);
        }
    }

    Ok(())
}

fn should_stop_container(existing_container: Option<&ContainerInspectResponse>) -> bool {
    existing_container
        .and_then(|existing_container| existing_container.state.as_ref())
        .and_then(|state| state.running)
        .is_some_and(|running| running)
}

fn generate_env(services: &Services, context: &Context) -> Result<()> {
    let existing_env = get_existing_env();
    let is_generated_first_time = existing_env.is_none();
    let existing_env = existing_env.unwrap_or_default();

    let services_env_vars = services.env_vars();
    let mut own_env_vars_names = HashSet::new();

    for env_name in context.app_config().env() {
        own_env_vars_names.insert(env_name.clone());
    }

    for (env_name, _) in &existing_env {
        own_env_vars_names.insert(env_name.clone());
    }

    for (env_name, _) in &services_env_vars {
        own_env_vars_names.remove(env_name);
    }

    let own_env_vars = {
        let mut own_env_vars = vec![];

        for env_name in own_env_vars_names {
            let env_value = &existing_env
                .get(&env_name)
                .map(|value| value.to_owned())
                .unwrap_or_else(|| "".to_owned());

            own_env_vars.push((env_name.clone(), env_value.clone()));
        }

        own_env_vars
    };

    generate_env_file(&services_env_vars, &own_env_vars)?;

    if is_generated_first_time {
        presentation::print_env_file_generated();
    }

    Ok(())
}

fn get_existing_env() -> Option<BTreeMap<String, String>> {
    let mut existing_env = BTreeMap::new();
    let env_file_path = Path::new(ENV_FILE_NAME);

    if !env_file_path.exists() {
        return None;
    }

    let Ok(iter) = dotenvy::from_path_iter(env_file_path) else {
        return None;
    };

    for item in iter {
        let Ok((key, value)) = item else {
            continue;
        };

        existing_env.insert(key, value);
    }

    Some(existing_env)
}

fn generate_env_file(
    services_env_vars: &[(String, String)],
    own_env_vars: &[(String, String)],
) -> Result<()> {
    let mut file = std::fs::File::create(ENV_FILE_NAME)?;

    for (key, value) in services_env_vars {
        writeln!(file, "{}={}", key, value)?;
    }

    writeln!(file, "\n# Your own variables come after this line")?;
    writeln!(file, "# Feel free to modify them as you want")?;

    for (key, value) in own_env_vars {
        writeln!(file, "{}={}", key, value)?;
    }

    Ok(())
}

async fn deploy_dependencies(
    services: &Services,
    context: &Context,
    docker: &Docker,
) -> Result<()> {
    let container_configs = services.to_container_configs(&context)?;

    for config in container_configs {
        let container_name = config.container_name();
        let image_name = config.image_name();
        let config = config.config();

        presentation::print_dependency_pulling(container_name);
        // docker
        //     .create_image(
        //         Some(image::CreateImageOptions {
        //             from_image: image_name,
        //             tag: "latest",
        //             ..Default::default()
        //         }),
        //         None,
        //         None,
        //     )
        //     .try_collect::<Vec<_>>()
        //     .await?;

        let image_info = docker
            // TODO: allow users to customize version
            .inspect_image(format!("{}:latest", image_name).as_str())
            .await?;
        let existing_container = match docker.inspect_container(container_name, None).await {
            Ok(container) => Some(container),
            Err(bollard::errors::Error::DockerResponseServerError {
                status_code: 404, ..
            }) => None,
            Err(e) => return Err(e.into()),
        };

        if should_recreate_dependency_container(&image_info, existing_container.as_ref()) {
            presentation::print_dependency_creating(container_name);

            if existing_container.is_some() {
                docker
                    .remove_container(
                        container_name,
                        Some(container::RemoveContainerOptions {
                            force: true,
                            ..Default::default()
                        }),
                    )
                    .await?;
            }

            docker
                .create_container(
                    Some(container::CreateContainerOptions {
                        name: container_name,
                        platform: None,
                    }),
                    config.clone(),
                )
                .await?;
        }

        presentation::print_dependency_starting(container_name);
        docker
            .start_container(container_name, None::<StartContainerOptions<String>>)
            .await?;

        presentation::print_dependency_success(container_name);
    }

    Ok(())
}

fn should_recreate_dependency_container(
    image_info: &ImageInspect,
    container_info: Option<&ContainerInspectResponse>,
) -> bool {
    let Some(image_id) = &image_info.id else {
        return true;
    };

    let Some(container_image) = container_info.and_then(|info| info.image.as_ref()) else {
        return true;
    };

    if container_image == image_id {
        return false;
    }

    true
}
