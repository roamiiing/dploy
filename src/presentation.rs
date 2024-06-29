use console::style;

#[inline]
pub fn print_cli_info() {
    println!(
        "\n\nRunning {} v{}\n\n",
        style(env!("CARGO_PKG_NAME")).cyan(),
        env!("CARGO_PKG_VERSION")
    );
}

#[inline]
pub fn print_config_not_found_error() {
    eprintln!("It seems that the config file does not exist.");
    eprintln!(
        "Please make sure the file exists and is named {}",
        style("config.toml").cyan()
    );
    eprintln!("or specify the path to the config file with the --config flag.\n");
}

#[inline]
pub fn print_dependency_pulling(name: &str) {
    println!("[{}] Pulling", style(name).cyan());
}

#[inline]
pub fn print_dependency_creating(name: &str) {
    println!("[{}] Creating", style(name).cyan());
}

#[inline]
pub fn print_dependency_starting(name: &str) {
    println!("[{}] Starting", style(name).cyan());
}

#[inline]
pub fn print_dependency_success(name: &str) {
    println!("[{}] {}\n", style(name).cyan(), style("Success").green());
}

#[inline]
pub fn print_dependency_exists(name: &str) {
    println!(
        "[{}] {}\n",
        style(name).cyan(),
        style("No need to replace").green(),
    );
}

#[inline]
pub fn print_starting_dependencies() {
    println!("{}", style("Starting dependencies").cyan());
}

#[inline]
pub fn print_generating_env_file() {
    println!("{}", style("Generating env file").cyan());
}
