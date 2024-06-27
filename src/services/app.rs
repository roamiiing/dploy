use crate::{context::Context, utils::network::free_port};

use super::EnvVars;

#[derive(Debug)]
pub struct AppService {
    env_vars: Vec<(String, String)>,
    ports_mapping: Option<Vec<(u16, u16)>>,
}

impl AppService {
    pub fn from_context(context: &Context, env_vars: Vec<(String, String)>) -> Self {
        let ports_mapping = context.should_expose_to_host().then(|| {
            context
                .app_config()
                .ports()
                .iter()
                .map(|port| (free_port(), *port))
                .collect()
        });

        Self {
            env_vars,
            ports_mapping,
        }
    }
}

impl EnvVars for AppService {
    fn env_vars(&self) -> Vec<(String, String)> {
        self.env_vars.clone()
    }
}
