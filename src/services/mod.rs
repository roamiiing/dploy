pub mod postgres;

pub enum ServiceKind {
    Postgres,
    Keydb,
}

pub trait EnvVars {
    fn env_vars(&self) -> Vec<(String, String)>;
}
