use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    /// Name of the user's application
    name: String,

    /// Relative path to the Dockerfile
    #[serde(default)]
    dockerfile: Option<String>,

    /// Names of environment variables of the application service
    #[serde(default)]
    env: Vec<String>,

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
}

impl AppConfig {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn dockerfile(&self) -> Option<&str> {
        self.dockerfile.as_deref()
    }

    pub fn env(&self) -> &[String] {
        &self.env
    }

    pub fn volumes(&self) -> &[String] {
        &self.volumes
    }

    pub fn watch(&self) -> &[String] {
        &self.watch
    }

    pub fn ports(&self) -> &[u16] {
        &self.ports
    }

    pub fn postgres(&self) -> Option<&PostgresConfig> {
        self.postgres.as_ref()
    }

    pub fn keydb(&self) -> Option<&KeydbConfig> {
        self.keydb.as_ref()
    }
}

#[derive(Debug, Deserialize)]
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

#[derive(Debug, Deserialize)]
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
