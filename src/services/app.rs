use std::{collections::HashMap, env};

use anyhow::Result;
use bollard::{container, image, models};

use crate::{
    context::Context,
    network::DPLOY_NETWORK,
    utils::{network::free_port, string::escape_sh},
};

use super::{ConnectionInfo, ContainerConfig, EnvVars, ServiceKind, ToContainerConfig};

const SERVICE_KIND: ServiceKind = ServiceKind::App;

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
            .should_expose_app_service_to_host()
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
            image_name: context.container_name_of(SERVICE_KIND),
            container_name: context.container_name_of(SERVICE_KIND),
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
                .map(|volume| context.mount(SERVICE_KIND, volume))
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

            networking_config: Some(container::NetworkingConfig {
                endpoints_config: HashMap::from([(
                    DPLOY_NETWORK.to_owned(),
                    models::EndpointSettings::default(),
                )]),
            }),

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

impl ConnectionInfo for AppService {
    fn connection_info(&self) -> Vec<String> {
        self.ports_mapping
            .iter()
            .map(|(host_port, container_port)| format!("127.0.0.1:{host_port} >> {container_port}"))
            .collect()
    }
}
