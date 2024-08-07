use std::{collections::HashMap, path::PathBuf};

use anyhow::Result;
use bollard::{container, models};

use crate::{
    context::{self, Context, HostPortBinding},
    network::DPLOY_NETWORK,
};

use super::{ConnectionInfo, ContainerConfig, ServiceKind, ToContainerConfig};

const CADDY_CONFIGS_PATH: &str = "caddy/configs";

const VOLUMES_MAPPINGS: &[(&str, &str)] = &[
    (CADDY_CONFIGS_PATH, "/etc/caddy"),
    ("caddy/internal/data", "/data/"),
    ("caddy/internal/config", "/config/"),
];

const PORT_MAPPINGS: &[(u16, u16)] = &[(80, 80), (443, 443)];

const IMAGE_NAME: &str = "caddy";

const GENERAL_CADDYFILE_CONTENTS: &str = "\"import /etc/caddy/*.caddy\"";

const SERVICE_KIND: ServiceKind = ServiceKind::Proxy;

pub struct ProxyService {
    name: String,
    project_name: String,
    bindings: Vec<HostPortBinding>,
    configs: Vec<ProxyServiceConfig>,
}

struct ProxyServiceConfig {
    domain: String,
    port: u16,
}

impl ProxyService {
    pub fn from_context(context: &Context) -> Option<Self> {
        let project_name = context.container_name_of(super::ServiceKind::App);
        let name = context.container_name_of(SERVICE_KIND);

        let configs = context.app_config().proxy(context.override_context());

        if configs.is_empty() {
            return None;
        }

        let bindings = PORT_MAPPINGS
            .iter()
            .map(|(host, inner)| HostPortBinding::manual(*host, "0.0.0.0", *inner, &name))
            .collect();

        let configs = configs
            .iter()
            .map(|config| ProxyServiceConfig {
                domain: config.domain.clone(),
                port: config.port,
            })
            .collect();

        Some(Self {
            name,
            project_name,
            bindings,
            configs,
        })
    }

    pub async fn post_up(
        &self,
        context: &context::Context,
        session: &openssh::Session,
        docker: &bollard::Docker,
    ) -> Result<()> {
        // put config for current service
        let config_contents = self.service_config_contents();
        let config_path = PathBuf::from("/etc/caddy").join(format!("{}.caddy", self.project_name));

        let command = format!(
            "echo \"{}\" > {}",
            config_contents,
            config_path.to_string_lossy()
        );

        let result = docker
            .create_exec(
                &self.name,
                bollard::exec::CreateExecOptions::<String> {
                    cmd: Some(
                        vec!["sh", "-c", &command]
                            .into_iter()
                            .map(Into::into)
                            .collect(),
                    ),
                    ..Default::default()
                },
            )
            .await?;

        docker.start_exec(&result.id, None).await?;

        let result = docker
            .create_exec(
                &self.name,
                bollard::exec::CreateExecOptions::<String> {
                    cmd: Some(
                        vec!["sh", "-c", "caddy reload -c /etc/caddy/Caddyfile"]
                            .into_iter()
                            .map(Into::into)
                            .collect(),
                    ),
                    ..Default::default()
                },
            )
            .await?;

        docker.start_exec(&result.id, None).await?;

        Ok(())
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    fn service_config_contents(&self) -> String {
        self.configs
            .iter()
            .map(|config| {
                format!(
                    "{}:443, {}:80 {{\n reverse_proxy {}:{} \n}}\n",
                    config.domain, config.domain, self.project_name, config.port
                )
            })
            .collect()
    }
}

impl ConnectionInfo for ProxyService {
    fn connection_info(&self) -> Vec<String> {
        self.configs
            .iter()
            .map(|config| format!("{} -> {}", config.domain, config.port))
            .collect()
    }
}

impl ToContainerConfig for ProxyService {
    fn to_container_config(&self, context: &Context) -> Result<ContainerConfig> {
        let name = context.container_name_of(SERVICE_KIND);

        let mut config = container::Config {
            image: Some(IMAGE_NAME.to_owned()),
            hostname: Some(name.clone()),
            domainname: Some(name.clone()),

            cmd: Some(
                vec![
                    "sh",
                    "-c",
                    &format!(
                        "echo {} > /etc/caddy/Caddyfile && caddy run --config /etc/caddy/Caddyfile --adapter caddyfile",
                        GENERAL_CADDYFILE_CONTENTS
                    )
                ]
                .into_iter()
                .map(String::from)
                .collect(),
            ),

            networking_config: Some(container::NetworkingConfig {
                endpoints_config: HashMap::from([(
                    DPLOY_NETWORK.to_owned(),
                    models::EndpointSettings::default(),
                )]),
            }),

            ..Default::default()
        };

        let mut host_config = models::HostConfig::default();

        host_config.mounts = Some(
            VOLUMES_MAPPINGS
                .iter()
                .map(|(host, inner)| context.manual_mount(host, inner))
                .collect(),
        );
        host_config.port_bindings = Some(HostPortBinding::to_port_bindings(
            &self.bindings.iter().collect::<Vec<_>>(),
        ));

        host_config.restart_policy = Some(models::RestartPolicy {
            name: Some(models::RestartPolicyNameEnum::ALWAYS),
            ..Default::default()
        });

        config.host_config = Some(host_config);

        Ok(ContainerConfig::new(name, IMAGE_NAME.to_owned(), config))
    }
}
