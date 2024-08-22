use std::{collections::HashMap, path::PathBuf};

use itertools::Itertools;

use crate::{context, docker, network, prelude::*, services};

const IMAGE_NAME: &str = "caddy";

const CADDY_CONFIGS_INNER_DIR: &str = "/etc/caddy";
const CADDY_CONFIG_INNER_FILE: &str = "Caddyfile";

const VOLUMES_MAPPINGS: &[(&str, &str)] = &[
    ("caddy/configs", CADDY_CONFIGS_INNER_DIR),
    ("caddy/internal/data", "/data/"),
    ("caddy/internal/config", "/config/"),
];

const PORT_MAPPINGS: &[(u16, u16)] = &[(80, 80), (443, 443)];

const GENERAL_CADDYFILE_CONTENTS: &str = "import /etc/caddy/*.caddy";

const RELOAD_CADDY_COMMAND: &str = "caddy reload -c /etc/caddy/Caddyfile";

const SERVICE_KIND: services::ServiceKind = services::ServiceKind::Proxy;

pub struct ProxyService {
    name: String,
    should_run: bool,
    app_service_container_name: String,
    bindings: Vec<context::HostPortBinding>,
    configs: Vec<ProxyServiceConfig>,
}

struct ProxyServiceConfig {
    domain: String,
    port: u16,
}

impl ProxyService {
    pub fn from_context(context: &context::Context) -> Self {
        let app_service_container_name = context.container_name_of(services::ServiceKind::App);
        let name = context.container_name_of(SERVICE_KIND);

        let configs = context.app_config().proxy(context.override_context());

        let bindings = PORT_MAPPINGS
            .iter()
            .map(|(host, inner)| context::HostPortBinding::manual(*host, "0.0.0.0", *inner, &name))
            .collect();

        let configs = configs
            .iter()
            .map(|config| ProxyServiceConfig {
                domain: config.domain.clone(),
                port: config.port,
            })
            .collect();

        let should_run = context.should_create_proxy_service();

        Self {
            name,
            app_service_container_name,
            bindings,
            configs,
            should_run,
        }
    }

    pub async fn post_up(&self, docker: &bollard::Docker) -> Result<()> {
        if !self.should_run {
            return Ok(());
        }

        let is_running = docker::check_container_running(docker, &self.name).await?;
        if !is_running {
            return Ok(());
        }

        // in case no proxy configs were specified, we need to remove the configs
        // in order to close the proxy to the app (in case user deleted it without
        // stopping)
        if self.configs.is_empty() {
            self.delete_configs(docker).await?;
        } else {
            self.put_configs(docker).await?;
        }

        self.reload_caddy(docker).await?;

        Ok(())
    }

    pub async fn post_down(&self, docker: &bollard::Docker) -> Result<()> {
        let is_running = docker::check_container_running(docker, &self.name).await?;
        if !is_running {
            return Ok(());
        }

        self.delete_configs(docker).await?;
        self.reload_caddy(docker).await?;

        Ok(())
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    async fn delete_configs(&self, docker: &bollard::Docker) -> Result<()> {
        let config_path = self.service_config_path();
        let command = format!("rm {config_path}");
        docker::exec_command_detached(docker, &self.name, &command).await?;

        Ok(())
    }

    async fn put_configs(&self, docker: &bollard::Docker) -> Result<()> {
        let config_contents = self.service_config_contents();
        let config_path = self.service_config_path();
        let command = format!(r#"echo "{config_contents}" > {config_path}"#,);
        docker::exec_command_detached(docker, &self.name, &command).await?;

        Ok(())
    }

    async fn reload_caddy(&self, docker: &bollard::Docker) -> Result<()> {
        let caddy_config_path = self.caddy_config_inner_file();
        let command = format!("caddy reload -c {caddy_config_path}");
        docker::exec_command_detached(docker, &self.name, &command).await?;

        Ok(())
    }

    fn service_config_contents(&self) -> String {
        self.configs
            .iter()
            .map(|config| self.https_config(config))
            .join("\n")
            .trim()
            .to_owned()
    }

    fn service_config_path(&self) -> String {
        let Self {
            app_service_container_name,
            ..
        } = self;

        PathBuf::from(CADDY_CONFIGS_INNER_DIR)
            .join(format!("{app_service_container_name}.caddy"))
            .to_string_lossy()
            .to_string()
    }

    fn https_config(&self, config: &ProxyServiceConfig) -> String {
        let ProxyServiceConfig { domain, port } = config;
        let Self {
            app_service_container_name,
            ..
        } = self;

        format!(
            "{domain}:443, {domain}:80 {{ \nreverse_proxy {app_service_container_name}:{port}\n }}",
        )
    }

    fn caddy_config_inner_file(&self) -> String {
        format!("{CADDY_CONFIGS_INNER_DIR}/{CADDY_CONFIG_INNER_FILE}")
    }
}

impl services::ConnectionInfo for ProxyService {
    fn connection_info(&self) -> Vec<String> {
        self.configs
            .iter()
            .map(|ProxyServiceConfig { domain, port }| format!("{domain} -> {port}"))
            .collect()
    }
}

impl services::ToContainerConfig for ProxyService {
    fn to_container_config(&self, context: &context::Context) -> Result<services::ContainerConfig> {
        let name = context.container_name_of(SERVICE_KIND);
        let config_file = self.caddy_config_inner_file();

        let mut config = bollard::container::Config {
            image: Some(IMAGE_NAME.to_owned()),
            hostname: Some(name.clone()),
            domainname: Some(name.clone()),

            cmd: Some(
                vec![
                    "sh",
                    "-c",
                    &format!(
                        r#"echo "{GENERAL_CADDYFILE_CONTENTS}" > {config_file} caddy run --config {config_file} --adapter caddyfile"#,
                    ),
                ]
                .into_iter()
                .map(String::from)
                .collect(),
            ),

            networking_config: Some(bollard::container::NetworkingConfig {
                endpoints_config: HashMap::from([(
                    network::DPLOY_NETWORK.to_owned(),
                    bollard::models::EndpointSettings::default(),
                )]),
            }),

            ..Default::default()
        };

        let mut host_config = bollard::models::HostConfig::default();

        host_config.mounts = Some(
            VOLUMES_MAPPINGS
                .iter()
                .map(|(host, inner)| context.manual_mount(host, inner))
                .collect(),
        );
        host_config.port_bindings = Some(context::HostPortBinding::to_port_bindings(
            &self.bindings.iter().collect::<Vec<_>>(),
        ));

        host_config.restart_policy = Some(bollard::models::RestartPolicy {
            name: Some(bollard::models::RestartPolicyNameEnum::ALWAYS),
            ..Default::default()
        });

        config.host_config = Some(host_config);

        Ok(services::ContainerConfig::new(
            name,
            IMAGE_NAME.to_owned(),
            config,
        ))
    }
}
