pub const DEFAULT_NAMESPACE: &str = "default";
pub const DEFAULT_DOCKERFILE_NAME: &str = "Dockerfile";
pub const DEFAULT_DOTENV_FILE_NAME: &str = ".env";

pub fn get_default_dockerfile_name() -> String {
    DEFAULT_DOCKERFILE_NAME.to_string()
}

pub fn get_default_dotenv_file_name() -> String {
    DEFAULT_DOTENV_FILE_NAME.to_string()
}
