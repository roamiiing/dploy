use std::env;

use bollard::{container, image};

use crate::{context::Context, utils::network::free_port};

use super::{ContainerConfig, EnvVars, ServiceKind};

#[derive(Debug)]
pub struct AppService {
    app_name: String,
    image_name: String,
    container_name: String,
    env_vars: Vec<(String, String)>,
    ports_mapping: Vec<(u16, u16)>,
}

impl AppService {
    pub fn from_context(context: &Context, env_vars: Vec<(String, String)>) -> Self {
        let ports_mapping = context
            .should_expose_to_host()
            .then(|| {
                context
                    .app_config()
                    .ports()
                    .iter()
                    .map(|port| (free_port(), *port))
                    .collect()
            })
            .unwrap_or_else(|| vec![]);

        let mut env_vars = env_vars;

        // TODO: refactor this to store all env in context
        // this will allow to also parameterize other services
        for env_name in context.app_config().env() {
            env_vars.push((env_name.to_owned(), env::var(&env_name).unwrap_or_default()));
        }

        Self {
            app_name: context.app_config().name().to_owned(),
            image_name: context.container_name_of(ServiceKind::App),
            container_name: context.container_name_of(ServiceKind::App),
            env_vars,
            ports_mapping,
        }
    }

    pub fn ports_mapping(&self) -> &[(u16, u16)] {
        &self.ports_mapping
    }

    pub fn to_image_build_config(&self) -> image::BuildImageOptions<String> {
        let config = image::BuildImageOptions {
            t: self.image_name.clone(),
            session: Some(self.image_name.to_owned()),
            ..Default::default()
        };

        config
    }

    pub fn to_container_config(&self) -> ContainerConfig {
        let config = container::Config {
            image: Some(self.image_name.clone()),
            hostname: Some(self.container_name.clone()),
            domainname: Some(self.container_name.clone()),

            env: Some(
                self.env_vars
                    .iter()
                    .map(|(key, value)| format!("{key}={value}"))
                    .collect(),
            ),

            ..Default::default()
        };

        ContainerConfig::new(self.container_name.clone(), self.image_name.clone(), config)
    }
}

impl EnvVars for AppService {
    fn env_vars(&self) -> Vec<(String, String)> {
        self.env_vars.clone()
    }
}
