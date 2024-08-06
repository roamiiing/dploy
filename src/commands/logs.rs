use std::io::Write;

use futures_util::TryStreamExt;

use crate::{context, prelude::*, presentation, services};

pub async fn logs(
    context: &context::Context,
    docker: &bollard::Docker,
    service: services::ServiceKind,
    count: Option<u64>,
) -> Result<()> {
    let logs_count = count.unwrap_or(20);
    let should_follow = count.is_none();

    let container_name = context.container_name_of(service);

    presentation::print_logs_count(&container_name, logs_count, should_follow);

    let existing_container = match docker.inspect_container(&container_name, None).await {
        Ok(container) => Some(container),
        Err(bollard::errors::Error::DockerResponseServerError {
            status_code: 404, ..
        }) => None,
        Err(e) => return Err(e.into()),
    };

    let Some(existing_container) = existing_container else {
        bail!("Your app is not running. Deploy it first");
    };

    if !existing_container
        .state
        .and_then(|state| state.running)
        .unwrap_or(false)
    {
        bail!("Your app is not running. Deploy it first");
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
