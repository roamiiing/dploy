use clap::{Parser, Subcommand};

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

        /// Stop the application
        #[clap(short, long, default_value_t = false)]
        stop: bool,
    },

    /// Run the application with all its dependencies locally
    Run {
        /// Stop the application
        #[clap(short, long, default_value_t = false)]
        stop: bool,
    },

    /// Run only the dependencies of the application locally
    Dev {
        /// Stop the application
        #[clap(short, long, default_value_t = false)]
        stop: bool,
    },
}

impl Command {
    pub fn stop(&self) -> bool {
        use Command::*;

        match self {
            Run { stop, .. } | Dev { stop, .. } | Deploy { stop, .. } => *stop,
        }
    }
}
