use crate::{context, prelude::*, ssh};

pub async fn get_default_docker_client() -> Result<bollard::Docker> {
    let docker = bollard::Docker::connect_with_defaults()?;

    docker.ping().await.context("Could not ping docker")?;

    Ok(docker)
}

pub async fn get_docker_client_with_session(
    context: &context::Context,
) -> Result<(bollard::Docker, openssh::Session)> {
    let (docker, session) = ssh::get_remote_docker_client(context).await?;

    Ok((docker, session))
}

pub async fn exec_command_detached(
    docker: &bollard::Docker,
    container_name: &str,
    command: &str,
) -> Result<()> {
    let result = docker
        .create_exec(
            container_name,
            bollard::exec::CreateExecOptions::<String> {
                cmd: Some(["sh", "-c", command].into_iter().map(Into::into).collect()),
                ..Default::default()
            },
        )
        .await?;

    docker.start_exec(&result.id, None).await?;

    Ok(())
}

/// This version ignores the error if the container is not found
pub async fn inspect_container(
    docker: &bollard::Docker,
    container_name: &str,
) -> Result<Option<bollard::models::ContainerInspectResponse>> {
    match docker.inspect_container(container_name, None).await {
        Ok(container) => Ok(Some(container)),
        Err(bollard::errors::Error::DockerResponseServerError {
            status_code: 404, ..
        }) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

pub async fn check_container_running(
    docker: &bollard::Docker,
    container_name: &str,
) -> Result<bool> {
    inspect_container(docker, container_name)
        .await
        .map(|container| {
            container.is_some_and(|container| {
                container.state.is_some_and(|state| state.running.is_some())
            })
        })
}
