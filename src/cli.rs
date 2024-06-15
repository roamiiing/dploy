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
    /// Deploy the application to a remote server
    Deploy,

    /// Run the application locally
    Run {
        #[clap(short, long, default_value_t = false)]
        stop: bool,
    },

    /// Run only the dependencies of the application
    Dev {
        #[clap(short, long, default_value_t = false)]
        stop: bool,
    },
}

impl Command {
    pub fn stop(&self) -> bool {
        use Command::*;

        match self {
            Run { stop } | Dev { stop } => *stop,
            _ => false,
        }
    }
}
