use std::{
    collections::{BTreeMap, HashSet},
    io::Write,
    path::Path,
    sync::Arc,
    time::{Duration, Instant},
};

use futures_util::TryStreamExt;
use notify::Watcher;

use crate::{
    build,
    commands::stop::stop,
    context, network,
    prelude::*,
    presentation,
    services::{self, ToContainerConfig},
};

use super::logs;

const ENV_FILE_NAME: &str = ".env";
const WATCH_POLL_INTERVAL: Duration = Duration::from_secs(1);
const WATCH_COOLDOWN: Duration = Duration::from_secs(3);

pub async fn deploy(
    context: &context::Context,
    docker: &bollard::Docker,
    services: &services::Services,
) -> Result<()> {
    if dotenvy::from_path(ENV_FILE_NAME).is_ok() {
        presentation::print_env_file_loaded();
    } else {
        presentation::print_env_file_failed_to_load();
    }

    if context.should_generate_env_file() {
        presentation::print_env_file_generating();
        generate_env(&services, context)?;
    }

    if context.should_create_network() {
        presentation::print_network_creating();
        network::create_dploy_network(docker).await?;
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

pub async fn deploy_watch(
    context: Arc<context::Context>,
    docker: Arc<bollard::Docker>,
    services: &services::Services,
    watch_paths: &[&Path],
) -> Result<()> {
    if watch_paths.is_empty() {
        bail!("Called with --watch flag but no paths were provided. Please provide at least one path to watch in the dploy.toml");
    }

    deploy(&context, &docker, services).await?;
    let mut handle = tokio::spawn(logs::logs(
        Arc::clone(&context),
        Arc::clone(&docker),
        services::ServiceKind::App,
        None,
    ));

    let (tx, rx) = std::sync::mpsc::channel();
    let (tx_abort, rx_abort) = std::sync::mpsc::channel();

    let mut debouncer = notify_debouncer_full::new_debouncer(WATCH_POLL_INTERVAL, None, tx)?;

    let watcher = debouncer.watcher();

    for path in watch_paths {
        watcher
            .watch(path, notify::RecursiveMode::Recursive)
            .context("Could not start watcher. Please make sure the folder exists")?;
    }

    ctrlc::set_handler(move || {
        presentation::print_ctrlc_received();
        tx_abort.send(()).unwrap();
    })?;

    let mut last_deploy = Instant::now();

    // don't care about blocking here
    loop {
        if rx_abort.try_recv().is_ok() {
            break;
        }

        if let Ok(res) = rx.try_recv() {
            if let Ok(events) = res {
                if Instant::now() - last_deploy < WATCH_COOLDOWN
                    || events.is_empty()
                    || !events.iter().any(|event| event.kind.is_modify())
                {
                    continue;
                }

                presentation::print_watch_files_changed();

                handle.abort();

                if let Some(service) = services.app() {
                    deploy_app_service(service, &context, &docker).await?;
                }

                handle = tokio::spawn(logs::logs(
                    Arc::clone(&context),
                    Arc::clone(&docker),
                    services::ServiceKind::App,
                    None,
                ));

                last_deploy = Instant::now();
            }
        }
    }

    handle.abort();

    presentation::print_ctrlc_started();
    stop(&context, &docker, services).await?;

    Ok(())
}

async fn deploy_app_service(
    app_service: &services::app::AppService,
    context: &context::Context,
    docker: &bollard::Docker,
) -> Result<()> {
    let container_config = app_service.to_container_config(context)?;
    let container_name = container_config.container_name();

    presentation::print_image_building(container_name);
    build::build_app_service_image(app_service, docker).await?;
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
            Some(bollard::container::CreateContainerOptions {
                name: container_name,
                ..Default::default()
            }),
            container_config.config().clone(),
        )
        .await?;

    presentation::print_app_container_starting(container_name);
    docker
        .start_container(
            container_name,
            None::<bollard::container::StartContainerOptions<String>>,
        )
        .await?;

    presentation::print_app_container_success(container_name);

    Ok(())
}

fn generate_env(services: &services::Services, context: &context::Context) -> Result<()> {
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
    services: &services::Services,
    context: &context::Context,
    docker: &bollard::Docker,
) -> Result<()> {
    let container_configs = services.to_container_configs(&context)?;

    for config in container_configs {
        let container_name = config.container_name();
        let image_name = config.image_name();
        let config = config.config();

        presentation::print_dependency_pulling(container_name);
        docker
            .create_image(
                Some(bollard::image::CreateImageOptions {
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
                        Some(bollard::container::RemoveContainerOptions {
                            force: true,
                            ..Default::default()
                        }),
                    )
                    .await?;
            }

            docker
                .create_container(
                    Some(bollard::container::CreateContainerOptions {
                        name: container_name,
                        platform: None,
                    }),
                    config.clone(),
                )
                .await?;
        }

        presentation::print_dependency_starting(container_name);
        docker
            .start_container(
                container_name,
                None::<bollard::container::StartContainerOptions<String>>,
            )
            .await?;

        presentation::print_dependency_success(container_name);
    }

    Ok(())
}

fn should_recreate_dependency_container(
    image_info: &bollard::models::ImageInspect,
    container_info: Option<&bollard::models::ContainerInspectResponse>,
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
