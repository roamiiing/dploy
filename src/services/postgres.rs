use std::collections::HashMap;

use anyhow::Result;
use bollard::{container, models};

use crate::{
    context::{Context, HostPortBinding},
    network::DPLOY_NETWORK,
};

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

    binding: HostPortBinding,
}

impl PostgresService {
    pub fn from_context(context: &Context) -> Option<Self> {
        context.app_config().postgres().map(|config| Self {
            expose_url_to_env: config.expose_url_to_env().map(ToOwned::to_owned),

            database_name: config
                .database_name()
                .unwrap_or(context.app_config().name())
                .to_owned(),
            database_user: DEFAULT_USER.to_owned(),
            database_password: DEFAULT_PASSWORD.to_owned(),

            binding: context.host_port_binding_of(SERVICE_KIND, DEFAULT_PORT),
        })
    }

    pub fn construct_url(&self, host: &str, port: u16) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.database_user, self.database_password, host, port, self.database_name
        )
    }

    pub fn inner_url(&self) -> String {
        let inner_port = self.binding.inner_port();
        let inner_host = self.binding.inner_host();

        self.construct_url(inner_host, inner_port)
    }

    pub fn host_url(&self) -> Option<String> {
        let host_port = self.binding.host_port();
        let host_host = self.binding.host_host();

        host_port.map(|port| self.construct_url(host_host, port))
    }
}

impl EnvVars for PostgresService {
    fn env_vars(&self) -> Vec<(String, String)> {
        let mut vars = Vec::new();

        if let Some(expose_url_to_env) = &self.expose_url_to_env {
            vars.push((expose_url_to_env.clone(), self.inner_url()))
        }

        vars
    }
}

impl ConnectionInfo for PostgresService {
    fn connection_info(&self) -> Vec<String> {
        vec![self.host_url()].into_iter().flatten().collect()
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

            networking_config: Some(container::NetworkingConfig {
                endpoints_config: HashMap::from([(
                    DPLOY_NETWORK.to_owned(),
                    models::EndpointSettings::default(),
                )]),
            }),

            ..Default::default()
        };

        let mut host_config = models::HostConfig::default();

        host_config.mounts = Some(vec![context.mount(SERVICE_KIND, DATA_PATH)]);
        host_config.port_bindings = Some(self.binding.to_port_binding());

        config.host_config = Some(host_config);

        Ok(ContainerConfig::new(name, IMAGE_NAME.to_owned(), config))
    }
}
