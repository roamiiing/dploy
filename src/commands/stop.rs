use crate::{
    context,
    prelude::*,
    presentation,
    services::{self, ToContainerConfig},
};

pub async fn stop(context: &context::Context, docker: &bollard::Docker) -> Result<()> {
    let services = services::Services::from_context(context);

    if let Some(service) = services.app() {
        stop_app_service(service, context, docker).await?;
    }

    presentation::print_dependencies_stopping();
    stop_dependencies(&services, context, docker).await?;

    Ok(())
}

async fn stop_app_service(
    app_service: &services::app::AppService,
    context: &context::Context,
    docker: &bollard::Docker,
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

async fn stop_dependencies(
    services: &services::Services,
    context: &context::Context,
    docker: &bollard::Docker,
) -> Result<()> {
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
            presentation::print_dependency_stopped(container_name);
        } else {
            presentation::print_dependency_already_stopped(container_name);
        }
    }

    Ok(())
}

fn should_stop_container(
    existing_container: Option<&bollard::models::ContainerInspectResponse>,
) -> bool {
    existing_container
        .and_then(|existing_container| existing_container.state.as_ref())
        .and_then(|state| state.running)
        .is_some_and(|running| running)
}
