#![allow(dead_code)]

use std::fs;

use anyhow::Result;
use clap::Parser;

mod cli;
mod config;
mod context;
mod presentation;
mod services;
mod utils;

#[tokio::main]
async fn main() -> Result<()> {
    let args = cli::Args::try_parse()?;

    presentation::print_cli_info();

    let file_contents = match fs::read_to_string(&args.config) {
        Ok(contents) => contents,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            presentation::print_config_not_found_error();
            return Err(error.into());
        }
        Err(error) => return Err(error.into()),
    };
    let app_config: config::AppConfig = toml::from_str(&file_contents)?;

    let context = context::Context::new(args, app_config);

    println!("{:#?}", context);

    Ok(())
}
