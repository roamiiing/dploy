use std::env;

use anyhow::Result;
use bollard::{container, image, models};

use crate::{
    context::Context,
    utils::{network::free_port, string::escape_sh},
};

use super::{ContainerConfig, EnvVars, ServiceKind, ToContainerConfig};

#[derive(Debug)]
pub struct AppService {
    app_name: String,
    image_name: String,
    container_name: String,
    env_vars: Vec<(String, String)>,
    ports_mapping: Vec<(u16, u16)>,
    volumes: Vec<String>,
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
            volumes: context
                .app_config()
                .volumes()
                .iter()
                .map(|volume| volume.to_owned())
                .collect(),
        }
    }

    pub fn ports_mapping(&self) -> &[(u16, u16)] {
        &self.ports_mapping
    }

    pub fn to_image_build_config(&self) -> image::BuildImageOptions<String> {
        let config = image::BuildImageOptions {
            t: self.image_name.clone(),
            ..Default::default()
        };

        config
    }
}

impl ToContainerConfig for AppService {
    fn to_container_config(&self, context: &Context) -> Result<ContainerConfig> {
        let mut host_config = models::HostConfig::default();

        host_config.mounts = Some(
            self.volumes
                .iter()
                .map(|volume| models::Mount {
                    target: Some(volume.to_owned()),
                    source: Some(
                        context
                            .volume_path_of(ServiceKind::App, volume)
                            .to_string_lossy()
                            .to_string(),
                    ),
                    ..Default::default()
                })
                .collect(),
        );

        host_config.port_bindings = Some(
            self.ports_mapping
                .iter()
                .map(|(host_port, container_port)| {
                    (
                        // TODO: DPLY-18 support not only tcp
                        format!("{}/tcp", container_port),
                        Some(vec![models::PortBinding {
                            host_ip: Some("0.0.0.0".to_owned()),
                            host_port: Some(format!("{}", host_port)),
                        }]),
                    )
                })
                .collect(),
        );

        let config = container::Config {
            image: Some(self.image_name.clone()),
            hostname: Some(self.container_name.clone()),
            domainname: Some(self.container_name.clone()),

            env: Some(
                self.env_vars
                    .iter()
                    .map(|(key, value)| (key, escape_sh(value)))
                    .map(|(key, value)| format!("{key}={value}"))
                    .collect(),
            ),

            host_config: Some(host_config),

            ..Default::default()
        };

        Ok(ContainerConfig::new(
            self.container_name.clone(),
            self.image_name.clone(),
            config,
        ))
    }
}

impl EnvVars for AppService {
    fn env_vars(&self) -> Vec<(String, String)> {
        self.env_vars.clone()
    }
}
