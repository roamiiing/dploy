use serde::Deserialize;

use crate::constants;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    #[serde(flatten)]
    config: TopLevelAppConfig,

    #[serde(default, rename = "override")]
    overrides: Vec<OverrideConfig>,
}

#[derive(Debug, Deserialize, Default)]
pub struct OverrideRule {
    #[serde(default)]
    namespace: Option<String>,

    #[serde(default)]
    command: Option<OverrideRuleCommand>,
}

/// This is meant to be passed into getters
#[derive(Debug, Clone)]
pub struct OverrideContext {
    pub namespace: String,

    pub command: OverrideRuleCommand,
}

#[derive(Debug, Deserialize, Default)]
pub struct OverrideConfig {
    #[serde(rename = "for")]
    rule: OverrideRule,

    #[serde(flatten)]
    config: TopLevelOverrideConfig,
}

#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OverrideRuleCommand {
    Dev,
    Run,
    Deploy,
}

#[derive(Debug, Deserialize, Default)]
pub struct TopLevelAppConfig {
    /// Name of the user's application
    name: String,

    /// Relative path to the Dockerfile
    #[serde(default = "constants::get_default_dockerfile_name")]
    dockerfile: String,

    /// Names of environment variables of the application service
    #[serde(default)]
    env: Vec<String>,

    /// Relative path to .env file
    #[serde(default = "constants::get_default_dotenv_file_name")]
    env_file: String,

    /// Expose namespace to specified environment variable
    #[serde(default)]
    expose_namespace_to_env: Option<String>,

    /// Paths to persistent volumes inside the container
    /// These volumes will be automatically mounted
    #[serde(default)]
    volumes: Vec<String>,

    /// Paths to watch for changes
    #[serde(default)]
    watch: Vec<String>,

    /// Ports exposed by the application service
    #[serde(default)]
    ports: Vec<u16>,

    /// Configuration for Postgres
    #[serde(default)]
    postgres: Option<PostgresConfig>,

    /// Configuration for Keydb
    #[serde(default)]
    keydb: Option<KeydbConfig>,

    /// Configuration for Proxy
    #[serde(default)]
    proxy: Vec<ProxyConfig>,
}

#[derive(Debug, Deserialize, Default)]
pub struct TopLevelOverrideConfig {
    /// Name of the user's application
    #[serde(default)]
    name: Option<String>,

    /// Relative path to the Dockerfile
    #[serde(default)]
    dockerfile: Option<String>,

    /// Names of environment variables of the application service
    #[serde(default)]
    env: Option<Vec<String>>,

    /// Relative path to .env file
    #[serde(default)]
    env_file: Option<String>,

    /// Expose namespace to specified environment variable
    #[serde(default)]
    expose_namespace_to_env: Option<String>,

    /// Paths to persistent volumes inside the container
    /// These volumes will be automatically mounted
    #[serde(default)]
    volumes: Option<Vec<String>>,

    /// Paths to watch for changes
    #[serde(default)]
    watch: Option<Vec<String>>,

    /// Ports exposed by the application service
    #[serde(default)]
    ports: Option<Vec<u16>>,

    /// Configuration for Postgres
    #[serde(default)]
    postgres: Option<PostgresConfig>,

    /// Configuration for Keydb
    #[serde(default)]
    keydb: Option<KeydbConfig>,

    /// Configuration for Proxy
    #[serde(default)]
    proxy: Option<Vec<ProxyConfig>>,
}

impl AppConfig {
    pub fn name(&self, context: &OverrideContext) -> &str {
        self.resolve_field(
            context,
            |config| &config.name,
            |config| config.name.as_ref(),
        )
    }

    pub fn dockerfile(&self, context: &OverrideContext) -> &str {
        self.resolve_field(
            context,
            |config| &config.dockerfile,
            |config| config.dockerfile.as_ref(),
        )
    }

    pub fn env(&self, context: &OverrideContext) -> &[String] {
        self.resolve_field(context, |config| &config.env, |config| config.env.as_ref())
    }

    pub fn env_file(&self, context: &OverrideContext) -> &str {
        self.resolve_field(
            context,
            |config| &config.env_file,
            |config| config.env_file.as_ref(),
        )
    }

    pub fn expose_namespace_to_env(&self, context: &OverrideContext) -> Option<&str> {
        self.resolve_optional_field(
            context,
            |config| config.expose_namespace_to_env.as_deref(),
            |config| config.expose_namespace_to_env.as_deref(),
        )
    }

    pub fn volumes(&self, context: &OverrideContext) -> &[String] {
        self.resolve_field(
            context,
            |config| &config.volumes,
            |config| config.volumes.as_ref(),
        )
    }

    pub fn watch(&self, context: &OverrideContext) -> &[String] {
        self.resolve_field(
            context,
            |config| &config.watch,
            |config| config.watch.as_ref(),
        )
    }

    pub fn ports(&self, context: &OverrideContext) -> &[u16] {
        self.resolve_field(
            context,
            |config| &config.ports,
            |config| config.ports.as_ref(),
        )
    }

    pub fn postgres(&self, context: &OverrideContext) -> Option<&PostgresConfig> {
        self.resolve_optional_field(
            context,
            |config| config.postgres.as_ref(),
            |config| config.postgres.as_ref(),
        )
    }

    pub fn keydb(&self, context: &OverrideContext) -> Option<&KeydbConfig> {
        self.resolve_optional_field(
            context,
            |config| config.keydb.as_ref(),
            |config| config.keydb.as_ref(),
        )
    }

    pub fn proxy(&self, context: &OverrideContext) -> &[ProxyConfig] {
        self.resolve_field(
            context,
            |config| &config.proxy,
            |config| config.proxy.as_ref(),
        )
    }

    fn active_overrides(&self, context: &OverrideContext) -> Vec<&OverrideConfig> {
        self.overrides
            .iter()
            .filter(|override_config| {
                override_config.rule.namespace.is_none()
                    || override_config
                        .rule
                        .namespace
                        .as_ref()
                        .is_some_and(|namespace| namespace == &context.namespace)
            })
            .filter(|override_config| {
                override_config.rule.command.is_none()
                    || override_config
                        .rule
                        .command
                        .as_ref()
                        .is_some_and(|command| command == &context.command)
            })
            .collect()
    }

    fn resolve_field<'a, ReturnType, Field, OverrideField>(
        &'a self,
        context: &OverrideContext,
        field: Field,
        override_field: OverrideField,
    ) -> ReturnType
    where
        Field: Fn(&'a TopLevelAppConfig) -> ReturnType,
        OverrideField: Fn(&'a TopLevelOverrideConfig) -> Option<ReturnType>,
        ReturnType: 'a,
    {
        let active_override = self
            .active_overrides(context)
            .into_iter()
            .filter_map(|override_config| (override_field)(&override_config.config))
            .last();

        if let Some(value) = active_override {
            value
        } else {
            field(&self.config)
        }
    }

    fn resolve_optional_field<'a, ReturnType, Field, OverrideField>(
        &'a self,
        context: &OverrideContext,
        field: Field,
        override_field: OverrideField,
    ) -> Option<ReturnType>
    where
        Field: Fn(&'a TopLevelAppConfig) -> Option<ReturnType>,
        OverrideField: Fn(&'a TopLevelOverrideConfig) -> Option<ReturnType>,
        ReturnType: 'a,
    {
        let active_override = self
            .active_overrides(context)
            .into_iter()
            .filter_map(|override_config| (override_field)(&override_config.config))
            .last();

        if let Some(value) = active_override {
            Some(value)
        } else {
            field(&self.config)
        }
    }
}

#[derive(Debug, Deserialize, Default)]
pub struct PostgresConfig {
    /// Name of the environment variable for the database URL
    #[serde(default)]
    expose_url_to_env: Option<String>,

    /// Name of the database
    #[serde(default)]
    database_name: Option<String>,
}

impl PostgresConfig {
    pub fn expose_url_to_env(&self) -> Option<&str> {
        self.expose_url_to_env.as_deref()
    }

    pub fn database_name(&self) -> Option<&str> {
        self.database_name.as_deref()
    }
}

#[derive(Debug, Deserialize, Default)]
pub struct KeydbConfig {
    /// Name of the environment variable for the database URL
    #[serde(default)]
    expose_url_to_env: Option<String>,
}

impl KeydbConfig {
    pub fn expose_url_to_env(&self) -> Option<&str> {
        self.expose_url_to_env.as_deref()
    }
}

#[derive(Debug, Deserialize)]
pub struct ProxyConfig {
    /// Domain name of the proxy
    /// Note that SSL will be generated automatically
    pub domain: String,

    /// Port inside the container
    pub port: u16,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_field() {
        let config = TopLevelAppConfig {
            name: "dploy-test".to_owned(),
            volumes: vec!["data".to_owned()],
            dockerfile: "Dockerfile".to_owned(),

            postgres: Some(PostgresConfig {
                database_name: Some("dploy".to_owned()),
                ..Default::default()
            }),
            ..Default::default()
        };

        let context = OverrideContext {
            namespace: "default".to_owned(),
            command: OverrideRuleCommand::Deploy,
        };

        let other_context = OverrideContext {
            namespace: "other".to_owned(),
            command: OverrideRuleCommand::Deploy,
        };

        let app_config = AppConfig {
            config,
            overrides: vec![OverrideConfig {
                rule: OverrideRule {
                    namespace: Some("default".to_owned()),
                    command: Some(OverrideRuleCommand::Deploy),
                },
                config: TopLevelOverrideConfig {
                    dockerfile: Some("Dockerfile.prod".to_owned()),
                    ..Default::default()
                },
            }],
        };

        assert_eq!("Dockerfile.prod", app_config.dockerfile(&context));
        assert_eq!("dploy-test", app_config.name(&context));

        assert_eq!("Dockerfile", app_config.dockerfile(&other_context));
        assert_eq!("dploy-test", app_config.name(&other_context));
    }
}
