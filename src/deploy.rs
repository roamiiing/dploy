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

use crate::{context::Context, presentation, services::Services};

const ENV_FILE_NAME: &str = ".env";

pub async fn deploy(context: &Context, docker: &Docker) -> Result<()> {
    let services = Services::from_context(context);

    if context.should_generate_env_file() {
        presentation::print_generating_env_file();
        generate_env(&services, context)?;
    }

    presentation::print_starting_dependencies();
    deploy_dependencies(&services, context, docker).await?;

    Ok(())
}

pub fn generate_env(services: &Services, context: &Context) -> Result<()> {
    let existing_env = get_existing_env();
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

    Ok(())
}

pub fn get_existing_env() -> BTreeMap<String, String> {
    let mut existing_env = BTreeMap::new();
    let env_file_path = Path::new(ENV_FILE_NAME);

    if !env_file_path.exists() {
        return existing_env;
    }

    let Ok(iter) = dotenvy::from_path_iter(env_file_path) else {
        return existing_env;
    };

    for item in iter {
        let Ok((key, value)) = item else {
            continue;
        };

        existing_env.insert(key, value);
    }

    existing_env
}

pub fn generate_env_file(
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

pub async fn deploy_dependencies(
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
        docker
            .create_image(
                Some(image::CreateImageOptions {
                    from_image: image_name,
                    tag: "latest",
                    ..Default::default()
                }),
                None,
                None,
            )
            .try_collect::<Vec<_>>()
            .await?;

        let image_info = docker
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

            presentation::print_dependency_starting(container_name);
            docker
                .start_container(container_name, None::<StartContainerOptions<String>>)
                .await?;

            presentation::print_dependency_success(container_name);
        } else {
            presentation::print_dependency_exists(container_name);
        }
    }

    Ok(())
}

pub fn should_recreate_dependency_container(
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
