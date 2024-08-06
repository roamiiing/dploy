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
