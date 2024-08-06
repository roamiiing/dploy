#![allow(dead_code)]

use std::fs;

use clap::Parser;

use crate::prelude::*;

mod build;
mod cli;
mod commands;
mod config;
mod context;
mod docker;
mod network;
mod prelude;
mod presentation;
mod services;
mod ssh;
mod utils;

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

    match context.args().command() {
        cli::Command::Dev { command, .. } => match command {
            None => {
                let docker = docker::get_default_docker_client().await?;

                commands::deploy::deploy(&context, &docker).await?;
            }
            Some(cli::DevCommand::Stop) => {
                let docker = docker::get_default_docker_client().await?;

                commands::stop::stop(&context, &docker).await?;
            }
        },

        cli::Command::Run { command, .. } => match command {
            None => {
                let docker = docker::get_default_docker_client().await?;

                commands::deploy::deploy(&context, &docker).await?;
            }
            Some(cli::RunCommand::Stop) => {
                let docker = docker::get_default_docker_client().await?;

                commands::stop::stop(&context, &docker).await?;
            }
        },

        cli::Command::Deploy { command, .. } => match command {
            None => {
                let (docker, session) = docker::get_docker_client_with_session(&context).await?;

                commands::deploy::deploy(&context, &docker).await?;

                session.close().await?;
            }
            Some(cli::DeployCommand::Stop) => {
                let (docker, session) = docker::get_docker_client_with_session(&context).await?;

                commands::stop::stop(&context, &docker).await?;

                session.close().await?;
            }
        },
    }

    Ok(())
}
