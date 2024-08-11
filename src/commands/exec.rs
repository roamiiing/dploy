use crate::{context, docker, prelude::*, presentation, services};

#[derive(Debug, Clone)]
pub struct ExecArgs {
    service: services::ServiceKind,
    command: String,
}

impl ExecArgs {
    pub fn new(service: services::ServiceKind, command: String) -> Self {
        Self { service, command }
    }

    pub fn service(&self) -> services::ServiceKind {
        self.service
    }

    pub fn command(&self) -> &str {
        &self.command
    }
}

pub async fn exec(
    context: &context::Context,
    docker: &bollard::Docker,
    args: &ExecArgs,
) -> Result<()> {
    let service_kind = args.service();
    let container_name = context.container_name_of(service_kind);

    let is_running = docker::check_container_running(docker, &container_name).await?;
    if !is_running {
        bail!("{container_name} is not running");
    }

    let command = args.command();

    presentation::print_command_executing(&container_name);
    docker::exec_command_attached(docker, &container_name, command).await?;

    Ok(())
}
