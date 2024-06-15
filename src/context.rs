use crate::{cli, config};

#[derive(Debug)]
pub struct Context {
    args: cli::Args,
    app_config: config::AppConfig,
}

impl Context {
    pub fn new(args: cli::Args, app_config: config::AppConfig) -> Self {
        Self { args, app_config }
    }

    pub fn args(&self) -> &cli::Args {
        &self.args
    }

    pub fn pod_config(&self) -> &config::AppConfig {
        &self.app_config
    }
}
