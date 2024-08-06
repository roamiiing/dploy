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
        cli::Command::Dev { command, .. } => {
            let docker = docker::get_default_docker_client().await?;

            match command {
                None => {
                    commands::deploy::deploy(&context, &docker).await?;
                }
                Some(cli::DevCommand::Stop) => {
                    commands::stop::stop(&context, &docker).await?;
                }
            }
        }

        cli::Command::Run { command, .. } => {
            let docker = docker::get_default_docker_client().await?;

            match command {
                None => {
                    commands::deploy::deploy(&context, &docker).await?;
                }
                Some(cli::RunCommand::Stop) => {
                    commands::stop::stop(&context, &docker).await?;
                }
                Some(cli::RunCommand::Logs { tail, .. }) => {
                    commands::logs::logs(&context, &docker, tail.clone()).await?;
                }
            }
        }

        cli::Command::Deploy { command, .. } => {
            let (docker, session) = docker::get_docker_client_with_session(&context).await?;

            match command {
                None => {
                    commands::deploy::deploy(&context, &docker).await?;
                }
                Some(cli::DeployCommand::Stop) => {
                    commands::stop::stop(&context, &docker).await?;
                }
                Some(cli::DeployCommand::Logs { tail, .. }) => {
                    commands::logs::logs(&context, &docker, tail.clone()).await?;
                }
            }

            session.close().await?;
        }
    }

    Ok(())
}
