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
