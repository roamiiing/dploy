use clap::{Parser, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};

use crate::{constants, services::ServiceKind};

#[derive(Debug, Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
pub struct Args {
    /// Relative path to the config file
    #[clap(short, long, default_value = "dploy.toml")]
    pub config: String,

    /// Namespace (or postfix) to use
    #[clap(short, long, default_value = constants::DEFAULT_NAMESPACE)]
    pub namespace: String,

    #[clap(subcommand)]
    pub command: Command,
}

impl Args {
    pub fn config(&self) -> &str {
        &self.config
    }

    pub fn namespace(&self) -> &str {
        &self.namespace
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
        #[clap(index = 1)]
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

        /// Watch for file changes and restart the application
        #[clap(short, long, default_value_t = false)]
        watch: bool,
    },

    /// Run the application with all its dependencies locally
    Run {
        /// Subcommand
        /// Run without any subcommand to start the application
        #[clap(subcommand)]
        command: Option<RunCommand>,

        /// Watch for file changes and restart the application
        #[clap(short, long, default_value_t = false)]
        watch: bool,
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
            _ => false,
        }
    }

    pub fn watch(&self) -> bool {
        use Command::*;

        match self {
            Run { watch, .. } => *watch,
            _ => false,
        }
    }
}
