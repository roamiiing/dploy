use anyhow::Result;
use bollard::container;

use crate::context::Context;

pub mod app;
pub mod postgres;

pub enum ServiceKind {
    /// Service being developed with dploy
    App,

    Postgres,
    Keydb,

    /// Reverse proxy service (Caddy)
    Proxy,
}

impl ServiceKind {
    pub fn to_string(&self) -> &str {
        use ServiceKind::*;

        match self {
            App => "app",
            Postgres => "postgres",
            Keydb => "keydb",
            Proxy => "proxy",
        }
    }

    /// Singleton services are deployed per server
    pub fn is_singleton(&self) -> bool {
        matches!(self, ServiceKind::Proxy)
    }

    /// Local services are deployed per project
    pub fn is_local(&self) -> bool {
        !self.is_singleton()
    }
}

/// Env vars to expose to app service
pub trait EnvVars {
    fn env_vars(&self) -> Vec<(String, String)>;
}

pub struct ContainerConfig {
    container_name: String,
    image_name: String,
    config: container::Config<String>,
}

impl ContainerConfig {
    pub fn new(
        container_name: String,
        image_name: String,
        config: container::Config<String>,
    ) -> Self {
        Self {
            container_name,
            image_name,
            config,
        }
    }

    pub fn container_name(&self) -> &str {
        &self.container_name
    }

    pub fn image_name(&self) -> &str {
        &self.image_name
    }

    pub fn config(&self) -> &container::Config<String> {
        &self.config
    }
}

pub trait ToContainerConfig {
    fn to_container_config(&self, context: &Context) -> Result<ContainerConfig>;
}

pub trait ConnectionInfo {
    fn connection_info(&self) -> Vec<String>;
}

pub struct Services {
    app: Option<app::AppService>,
    postgres: Option<postgres::PostgresService>,
}

impl Services {
    pub fn from_context(context: &Context) -> Self {
        let mut app_service_env_vars = vec![];

        let postgres = postgres::PostgresService::from_context(context);

        if let Some(postgres) = &postgres {
            app_service_env_vars.extend(postgres.env_vars());
        }

        let app = context
            .should_create_app_service()
            .then(|| app::AppService::from_context(context, app_service_env_vars));

        Self { app, postgres }
    }

    pub fn app(&self) -> Option<&app::AppService> {
        self.app.as_ref()
    }

    pub fn to_container_configs(&self, context: &Context) -> Result<Vec<ContainerConfig>> {
        let mut configs = vec![];

        if let Some(postgres) = &self.postgres {
            configs.push(postgres.to_container_config(context)?);
        }

        Ok(configs)
    }

    pub fn env_vars(&self, context: &Context) -> Vec<(String, String)> {
        let mut env_vars = vec![];

        if let Some(postgres) = &self.postgres {
            env_vars.extend(postgres.env_vars());
        }

        if let Some(expose_namespace_to_env) = context
            .app_config()
            .expose_namespace_to_env(context.override_context())
        {
            env_vars.push((
                expose_namespace_to_env.to_owned(),
                context.namespace().to_owned(),
            ));
        }

        env_vars
    }

    pub fn connection_info(&self) -> Vec<(ServiceKind, String)> {
        let mut infos = vec![];

        if let Some(postgres) = &self.postgres {
            infos.extend(
                postgres
                    .connection_info()
                    .into_iter()
                    .map(|s| (ServiceKind::Postgres, s)),
            );
        }

        if let Some(app) = &self.app {
            infos.extend(
                app.connection_info()
                    .into_iter()
                    .map(|s| (ServiceKind::App, s)),
            );
        }

        infos
    }
}
