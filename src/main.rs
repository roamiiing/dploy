#![allow(dead_code)]

use std::fs;

use anyhow::{Context, Result};
use bollard::Docker;
use clap::Parser;
use cli::Command;
use deploy::{deploy, stop};
use ssh::get_remote_docker_client;

mod build;
mod cli;
mod config;
mod context;
mod deploy;
mod network;
mod presentation;
mod services;
mod ssh;
mod utils;

const ENV_FILE_NAME: &str = ".env";

async fn run_cli() -> Result<()> {
    let args = cli::Args::try_parse()?;

    presentation::print_cli_info();

    let file_contents = match fs::read_to_string(&args.config) {
        Ok(contents) => contents,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            presentation::print_config_not_found_error();
            return Err(error.into());
        }
        Err(error) => return Err(error.into()),
    };
    let app_config: config::AppConfig = toml::from_str(&file_contents)?;

    let context = context::Context::new(args, app_config);
    let (docker, session) = if matches!(context.args().command(), Command::Deploy { .. }) {
        let (docker, session) = get_remote_docker_client(&context).await?;
        (docker, Some(session))
    } else {
        (Docker::connect_with_defaults()?, None)
    };

    docker.ping().await.context("Could not ping docker")?;

    if context.args().command().stop() {
        stop(&context, &docker).await?;
    } else {
        deploy(&context, &docker).await?;
    }

    if let Some(session) = session {
        session.close().await?;
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    match run_cli().await {
        Ok(_) => Ok(()),
        Err(error) => {
            eprintln!("{}", error);
            std::process::exit(1);
        }
    }
}
