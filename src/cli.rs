use clap::{Parser, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};

use crate::services::ServiceKind;

#[derive(Debug, Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
pub struct Args {
    /// Relative path to the config file
    #[clap(short, long, default_value = "dploy.toml")]
    pub config: String,

    #[clap(subcommand)]
    pub command: Command,
}

impl Args {
    pub fn config(&self) -> &str {
        &self.config
    }

    pub fn command(&self) -> &Command {
        &self.command
    }
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Deploy the application with all its dependencies to a remote server
    Deploy {
        /// Host of the remote server
        #[clap(index = 1, default_value = "127.0.0.1")]
        host: String,

        /// Port of the remote server
        #[clap(short, long, default_value_t = 22)]
        port: u16,

        /// Username of the remote server
        #[clap(short, long, default_value = "root")]
        username: String,

        /// Path to the private key file
        #[clap(short, long)]
        keyfile: Option<String>,

        /// Subcommand
        /// Run without any subcommand to start the application
        #[clap(subcommand)]
        command: Option<DeployCommand>,
    },

    /// Run the application with all its dependencies locally
    Run {
        /// Subcommand
        /// Run without any subcommand to start the application
        #[clap(subcommand)]
        command: Option<RunCommand>,
    },

    /// Run only the dependencies of the application locally
    Dev {
        /// Subcommand
        /// Run without any subcommand to start the application
        #[clap(subcommand)]
        command: Option<DevCommand>,
    },
}

#[derive(Debug, Subcommand)]
pub enum DevCommand {
    /// Stop the application
    Stop,

    /// Get logs of the specified service
    Logs {
        /// Number of logs to get. Omit to get 20 last logs + follow real time logs
        #[clap(short, long)]
        tail: Option<u64>,

        /// Service to get logs from
        #[clap(short, long)]
        service: DevLogsService,
    },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum DevLogsService {
    Postgres,
}

impl From<DevLogsService> for ServiceKind {
    fn from(value: DevLogsService) -> Self {
        match value {
            DevLogsService::Postgres => ServiceKind::Postgres,
        }
    }
}

#[derive(Debug, Subcommand)]
pub enum RunCommand {
    /// Stop the application
    Stop,

    /// Get logs of application container
    Logs {
        /// Number of logs to get. Omit to get 20 last logs + follow real time logs
        #[clap(short, long)]
        tail: Option<u64>,

        /// Service to get logs from
        #[clap(short, long, default_value = "app")]
        service: RunLogsService,
    },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum RunLogsService {
    App,
    Postgres,
}

impl From<RunLogsService> for ServiceKind {
    fn from(value: RunLogsService) -> Self {
        match value {
            RunLogsService::App => ServiceKind::App,
            RunLogsService::Postgres => ServiceKind::Postgres,
        }
    }
}

#[derive(Debug, Subcommand)]
pub enum DeployCommand {
    /// Stop the application
    Stop,

    /// Get logs of application container
    Logs {
        /// Number of logs to get. Omit to get 20 last logs + follow realtime logs
        #[clap(short, long)]
        tail: Option<u64>,

        /// Service to get logs from
        #[clap(short, long, default_value = "app")]
        service: DeployLogsService,
    },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum DeployLogsService {
    App,
    Postgres,
}

impl From<DeployLogsService> for ServiceKind {
    fn from(value: DeployLogsService) -> Self {
        match value {
            DeployLogsService::App => ServiceKind::App,
            DeployLogsService::Postgres => ServiceKind::Postgres,
        }
    }
}

impl Command {
    pub fn stop(&self) -> bool {
        use Command::*;

        match self {
            Deploy { command, .. } => matches!(command, Some(DeployCommand::Stop)),
            Run { command, .. } => matches!(command, Some(RunCommand::Stop)),
            Dev { command, .. } => matches!(command, Some(DevCommand::Stop)),
        }
    }
}
