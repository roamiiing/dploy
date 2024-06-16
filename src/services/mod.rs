use anyhow::Result;
use bollard::container;

use crate::context::Context;

pub mod postgres;

pub enum ServiceKind {
    Postgres,
    Keydb,
}

pub trait EnvVars {
    fn env_vars(&self) -> Vec<(String, String)>;
}

pub trait ToContainerConfig {
    fn to_container_config(&self, context: &Context) -> Result<container::Config<String>>;
}
