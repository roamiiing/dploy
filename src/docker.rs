use std::{
    io::{Read, Write},
    time::Duration,
};

use futures::StreamExt;
use termion::raw::IntoRawMode;
use tokio::io::AsyncWriteExt;

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

pub async fn exec_command_attached(
    docker: &bollard::Docker,
    container_name: &str,
    command: &str,
) -> Result<()> {
    let exec = docker
        .create_exec(
            container_name,
            bollard::exec::CreateExecOptions::<String> {
                cmd: Some(["sh", "-c", command].into_iter().map(Into::into).collect()),
                attach_stdout: Some(true),
                attach_stderr: Some(true),
                attach_stdin: Some(true),
                tty: Some(true),
                ..Default::default()
            },
        )
        .await?;

    let result = docker
        .start_exec(&exec.id, Some(bollard::exec::StartExecOptions::default()))
        .await?;

    match result {
        bollard::exec::StartExecResults::Attached {
            mut input,
            mut output,
        } => {
            // pipe stdin into the docker exec stream input
            tokio::spawn(async move {
                let mut stdin = termion::async_stdin().bytes();

                loop {
                    if let Some(Ok(byte)) = stdin.next() {
                        input.write_all(&[byte]).await.ok();
                    } else {
                        tokio::time::sleep(Duration::from_nanos(10)).await;
                    }
                }
            });

            // set stdout in raw mode so we can do tty stuff
            let stdout = std::io::stdout();
            let mut stdout = stdout.lock().into_raw_mode()?;

            // pipe docker exec output into stdout
            while let Some(Ok(output)) = output.next().await {
                stdout.write_all(output.into_bytes().as_ref())?;
                stdout.flush()?;
            }
        }
        _ => unreachable!(),
    }

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
