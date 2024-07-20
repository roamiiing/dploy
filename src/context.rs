use crate::{
    cli::{Args, Command},
    config::AppConfig,
    services::ServiceKind,
    utils::network::free_port,
};

#[derive(Debug)]
pub struct Context {
    args: Args,
    app_config: AppConfig,
}

impl Context {
    pub fn new(args: Args, app_config: AppConfig) -> Self {
        Self { args, app_config }
    }

    pub fn args(&self) -> &Args {
        &self.args
    }

    pub fn app_config(&self) -> &AppConfig {
        &self.app_config
    }

    pub fn container_name_of(&self, service_kind: ServiceKind) -> String {
        let prefix = self.app_config.name();

        let suffix = match service_kind {
            ServiceKind::Postgres => "postgres",
            ServiceKind::Keydb => "keydb",
            ServiceKind::App => self.app_config.name(),
        };

        // TODO: Allow users to customize the "default" part
        // to deploy different versions of the same service
        // simultaneously
        format!("{prefix}_{suffix}_default")
    }

    pub fn should_expose_to_host(&self) -> bool {
        use Command::*;

        matches!(self.args.command(), Dev { .. })
    }

    pub fn should_expose_app_service_to_host(&self) -> bool {
        use Command::*;

        matches!(self.args.command(), Run { .. })
    }

    pub fn should_create_app_service(&self) -> bool {
        use Command::*;

        matches!(self.args.command(), Deploy | Run { .. })
    }

    pub fn should_generate_env_file(&self) -> bool {
        use Command::*;

        matches!(self.args.command(), Dev { .. } | Run { .. })
    }

    pub fn host_of(&self, service_kind: ServiceKind) -> String {
        use Command::*;

        match self.args.command() {
            Deploy | Run { .. } => self.container_name_of(service_kind),
            _ => "127.0.0.1".to_owned(),
        }
    }

    pub fn port(&self, inner_port: u16) -> u16 {
        use Command::*;

        match self.args.command() {
            Dev { .. } => free_port(),
            _ => inner_port,
        }
    }
}
