pub const DEFAULT_NAMESPACE: &str = "default";
pub const DEFAULT_DOCKERFILE_NAME: &str = "Dockerfile";
pub const DEFAULT_DOTENV_FILE_NAME: &str = ".env";
pub const DEFAULT_DOCKER_CONTEXT: &str = ".";
pub const DEFAULT_DOCKER_IGNORE_FILE: &str = ".dockerignore";

pub fn get_default_dockerfile_name() -> String {
    DEFAULT_DOCKERFILE_NAME.to_string()
}

pub fn get_default_dotenv_file_name() -> String {
    DEFAULT_DOTENV_FILE_NAME.to_string()
}

pub fn get_default_docker_context() -> String {
    DEFAULT_DOCKER_CONTEXT.to_string()
}

pub fn get_default_ignore_files() -> Vec<String> {
    vec![DEFAULT_DOCKER_IGNORE_FILE.to_string()]
}
