use std::{net::TcpListener, path::Path, time::Duration};

use anyhow::{bail, Context, Result};
use bollard::{Docker, API_DEFAULT_VERSION};
use openssh::{ForwardType, KnownHosts, Session, SessionBuilder};

use crate::{context, presentation};

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(60);

pub async fn get_remote_docker_client(context: &context::Context) -> Result<(Docker, Session)> {
    let Some(credentials) = context.ssh_credentials() else {
        bail!("No SSH credentials provided")
    };

    let mut builder = SessionBuilder::default();

    builder
        .user(credentials.username().to_owned())
        .port(credentials.port())
        .known_hosts_check(KnownHosts::Accept)
        .control_directory(std::env::temp_dir())
        .connect_timeout(DEFAULT_TIMEOUT);

    if let Some(keyfile) = credentials.keyfile() {
        builder.keyfile(keyfile);
    }

    let local_addr = {
        let listener = TcpListener::bind("127.0.0.1:0")?;
        listener.local_addr()?
    };

    presentation::print_remote_host_connecting(credentials.host());
    let temp_dir = builder
        .launch_master(credentials.host())
        .await
        .context("Could not launch ssh")?;

    let session = Session::new_process_mux(temp_dir);
    presentation::print_remote_host_success(credentials.host());

    let socket_path = Path::new("/var/run/docker.sock");
    session
        .request_port_forward(ForwardType::Local, local_addr, socket_path)
        .await
        .context("Could not request port forward")?;

    let docker = Docker::connect_with_http(&local_addr.to_string(), 120, API_DEFAULT_VERSION)
        .context("Could not connect to docker")?;

    docker
        .version()
        .await
        .context("Could not get docker version")?;

    Ok((docker, session))
}
