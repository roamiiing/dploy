use std::{io::Write, sync::Arc};

use futures_util::TryStreamExt;

use crate::{context, docker, prelude::*, presentation, services};

pub async fn logs(
    context: Arc<context::Context>,
    docker: Arc<bollard::Docker>,
    service: services::ServiceKind,
    count: Option<u64>,
) -> Result<()> {
    let logs_count = count.unwrap_or(20);
    let should_follow = count.is_none();
    let container_name = context.container_name_of(service);

    let is_running = docker::check_container_running(&*docker, &container_name).await?;
    if !is_running {
        bail!("Cannot show logs because the container is not running. Deploy it first.");
    }

    let logs = docker.logs(
        &container_name,
        Some(bollard::container::LogsOptions {
            stdout: true,
            stderr: true,
            follow: should_follow,
            tail: logs_count.to_string(),
            ..Default::default()
        }),
    );

    presentation::print_logs_count(&container_name, logs_count, should_follow);

    logs.try_for_each(|chunk| async {
        let mut stdout = std::io::stdout();

        let bytes = match chunk {
            bollard::container::LogOutput::StdIn { message } => message,
            bollard::container::LogOutput::StdOut { message } => message,
            bollard::container::LogOutput::StdErr { message } => message,
            bollard::container::LogOutput::Console { message } => message,
        };

        stdout.write_all(&bytes).expect("Failed to write to stdout");
        stdout.flush().expect("Failed to flush stdout");

        Ok(())
    })
    .await?;

    Ok(())
}
