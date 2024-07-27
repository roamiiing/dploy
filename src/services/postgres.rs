use std::collections::HashMap;

use anyhow::Result;
use bollard::{container, models};

use crate::context::Context;

use super::{ConnectionInfo, ContainerConfig, EnvVars, ServiceKind, ToContainerConfig};

const DEFAULT_PORT: u16 = 5432;
const DEFAULT_USER: &str = "admin";
const DEFAULT_PASSWORD: &str = "admin";

const IMAGE_NAME: &str = "postgres";
const DATA_PATH: &str = "/var/lib/postgresql/data";

const SERVICE_KIND: ServiceKind = ServiceKind::Postgres;

pub struct PostgresService {
    expose_url_to_env: Option<String>,

    database_name: String,
    database_user: String,
    database_password: String,

    /// host and port inside the dploy network
    external_host: String,
    external_port: u16,
}

impl PostgresService {
    pub fn from_context(context: &Context) -> Option<Self> {
        let port = context.port(DEFAULT_PORT);

        context.app_config().postgres().map(|config| Self {
            expose_url_to_env: config.expose_url_to_env().map(ToOwned::to_owned),

            database_name: config
                .database_name()
                .unwrap_or(context.app_config().name())
                .to_owned(),
            database_user: DEFAULT_USER.to_owned(),
            database_password: DEFAULT_PASSWORD.to_owned(),

            external_host: context.host_of(SERVICE_KIND),
            external_port: port,
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

impl ConnectionInfo for PostgresService {
    fn connection_info(&self) -> Vec<String> {
        vec![self.url()]
    }
}

impl ToContainerConfig for PostgresService {
    fn to_container_config(&self, context: &Context) -> Result<ContainerConfig> {
        let name = context.container_name_of(SERVICE_KIND);

        let mut config = container::Config {
            image: Some(IMAGE_NAME.to_owned()),
            hostname: Some(name.clone()),
            domainname: Some(name.clone()),

            env: Some(vec![
                format!("POSTGRES_DB={}", self.database_name),
                format!("POSTGRES_USER={}", self.database_user),
                format!("POSTGRES_PASSWORD={}", self.database_password),
            ]),

            ..Default::default()
        };

        let mut host_config = models::HostConfig::default();

        let volume_path = context
            .volume_path_of(SERVICE_KIND, DATA_PATH)
            .to_string_lossy()
            .to_string();

        host_config.mounts = Some(vec![models::Mount {
            target: Some(DATA_PATH.to_owned()),
            source: Some(volume_path),
            typ: Some(models::MountTypeEnum::BIND),
            bind_options: Some(models::MountBindOptions {
                // TODO: check if this is the best way to do this
                create_mountpoint: Some(true),
                ..Default::default()
            }),
            ..Default::default()
        }]);

        if context.should_expose_to_host() {
            let port_binding = models::PortBinding {
                host_ip: Some("0.0.0.0".to_owned()),
                host_port: Some(format!("{}", self.external_port)),
            };

            let bindings_map =
                HashMap::from([(format!("{}/tcp", DEFAULT_PORT), Some(vec![port_binding]))]);

            host_config.port_bindings = Some(bindings_map);
        }

        config.host_config = Some(host_config);

        Ok(ContainerConfig::new(name, IMAGE_NAME.to_owned(), config))
    }
}
