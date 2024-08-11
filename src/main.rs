#![allow(dead_code)]

use std::{fs, sync::Arc};

use clap::Parser;

use crate::prelude::*;

mod build;
mod cli;
mod commands;
mod config;
mod constants;
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

    let namespace = args.namespace();
    if namespace != constants::DEFAULT_NAMESPACE {
        presentation::print_namespace_info(namespace);
    }

    let override_context = config::OverrideContext {
        namespace: namespace.to_string(),
        command: args.command().into(),
    };

    let file_contents = match fs::read_to_string(&args.config) {
        Ok(contents) => contents,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            presentation::print_config_not_found_error();
            return Err(error.into());
        }
        Err(error) => return Err(error.into()),
    };
    let app_config: config::AppConfig = toml::from_str(&file_contents)?;

    let context = Arc::new(context::Context::new(args, app_config, override_context));
    let services = services::Services::from_context(&context);

    match context.args().command() {
        cli::Command::Dev { command, .. } => {
            let docker = docker::get_default_docker_client().await?;

            match command {
                None => {
                    commands::deploy::deploy(&context, &docker, &services).await?;
                }
                Some(cli::DevCommand::Stop) => {
                    commands::stop::stop(&context, &docker, &services).await?;
                }
                Some(cli::DevCommand::Logs { tail, service, .. }) => {
                    commands::logs::logs(
                        Arc::clone(&context),
                        Arc::new(docker),
                        (*service).into(),
                        *tail,
                    )
                    .await?;
                }
                Some(cli::DevCommand::Exec {
                    service, command, ..
                }) => {
                    let args = commands::exec::ExecArgs::new((*service).into(), command.clone());
                    commands::exec::exec(&context, &docker, &args).await?;
                }
            }
        }

        // Run with watch
        cli::Command::Run {
            command: None,
            watch: true,
        } => {
            let docker = docker::get_default_docker_client().await?;
            commands::deploy::deploy_watch(
                Arc::clone(&context),
                Arc::new(docker),
                &services,
                &context
                    .app_config()
                    .watch(context.override_context())
                    .iter()
                    .map(|path| path.as_ref())
                    .collect::<Vec<_>>(),
            )
            .await?;
        }

        cli::Command::Run { command, .. } => {
            let docker = docker::get_default_docker_client().await?;

            match command {
                None => {
                    commands::deploy::deploy(&context, &docker, &services).await?;
                }
                Some(cli::RunCommand::Stop) => {
                    commands::stop::stop(&context, &docker, &services).await?;
                }
                Some(cli::RunCommand::Logs { tail, service, .. }) => {
                    commands::logs::logs(
                        Arc::clone(&context),
                        Arc::new(docker),
                        (*service).into(),
                        *tail,
                    )
                    .await?;
                }
                Some(cli::RunCommand::Exec {
                    service, command, ..
                }) => {
                    let args = commands::exec::ExecArgs::new((*service).into(), command.clone());
                    commands::exec::exec(&context, &docker, &args).await?;
                }
            }
        }

        cli::Command::Deploy {
            command: None,
            watch: true,
            ..
        } => {
            let (docker, session) = docker::get_docker_client_with_session(&context).await?;
            commands::deploy::deploy_watch(
                Arc::clone(&context),
                Arc::new(docker),
                &services,
                &context
                    .app_config()
                    .watch(context.override_context())
                    .iter()
                    .map(|path| path.as_ref())
                    .collect::<Vec<_>>(),
            )
            .await?;
            session.close().await?;
        }

        cli::Command::Deploy { command, .. } => {
            let (docker, session) = docker::get_docker_client_with_session(&context).await?;

            match command {
                None => {
                    commands::deploy::deploy(&context, &docker, &services).await?;
                }
                Some(cli::DeployCommand::Stop) => {
                    commands::stop::stop(&context, &docker, &services).await?;
                }
                Some(cli::DeployCommand::Logs { tail, service, .. }) => {
                    commands::logs::logs(
                        Arc::clone(&context),
                        Arc::new(docker),
                        (*service).into(),
                        *tail,
                    )
                    .await?;
                }
                Some(cli::DeployCommand::Exec {
                    service, command, ..
                }) => {
                    let args = commands::exec::ExecArgs::new((*service).into(), command.clone());
                    commands::exec::exec(&context, &docker, &args).await?;
                }
            }

            session.close().await?;
        }
    }

    Ok(())
}
