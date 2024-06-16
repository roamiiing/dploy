use crate::context::Context;

use super::{EnvVars, ServiceKind};

const DEFAULT_PORT: u16 = 5432;
const DEFAULT_USER: &str = "admin";
const DEFAULT_PASSWORD: &str = "admin";

pub struct PostgresService {
    expose_url_to_env: Option<String>,

    database_name: String,
    database_user: String,
    database_password: String,

    external_host: String,
    external_port: u16,
}

impl PostgresService {
    pub fn new(context: &Context) -> Option<Self> {
        context.app_config().postgres().map(|config| Self {
            expose_url_to_env: config.expose_url_to_env().map(ToOwned::to_owned),

            database_name: config
                .database_name()
                .unwrap_or(context.app_config().name())
                .to_owned(),
            database_user: DEFAULT_USER.to_owned(),
            database_password: DEFAULT_PASSWORD.to_owned(),

            external_host: context.host_of(ServiceKind::Postgres),
            external_port: context.port(DEFAULT_PORT),
        })
    }

    pub fn url(&self) -> String {
        let Self {
            database_name,
            database_user,
            database_password,
            external_host,
            external_port,
            ..
        } = self;

        format!(
            "postgres://{database_user}:{database_password}@{external_host}:{external_port}/{database_name}"
        )
    }
}

impl EnvVars for PostgresService {
    fn env_vars(&self) -> Vec<(String, String)> {
        let mut vars = Vec::new();

        if let Some(expose_url_to_env) = &self.expose_url_to_env {
            vars.push((expose_url_to_env.clone(), self.url()))
        }

        vars
    }
}
