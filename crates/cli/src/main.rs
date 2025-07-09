mod cli;
mod commands;
mod config;
mod event_processor;
mod formatters;
mod tables;
mod terminal_progress_reporter;

use std::process;

use clap::Parser;
use selfie::{
    config::{
        YamlLoader,
        loader::{ApplyToConfg, ConfigLoader},
    },
    fs::real::RealFileSystem,
};
use terminal_progress_reporter::TerminalProgressReporter;
use tracing::debug;

use crate::{cli::ClapCli, commands::dispatch_command};

fn init_tracing(verbose: bool) {
    let level = if verbose {
        tracing::Level::DEBUG
    } else {
        tracing::Level::WARN
    };

    tracing_subscriber::fmt().with_max_level(level).init();
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = ClapCli::parse();

    // Initialize tracing based on verbose flag
    init_tracing(args.verbose);
    debug!("CLI arguments: {:#?}", &args);

    let fs = RealFileSystem;

    // Use `config` for most things; use `original_config` for `config` commands, where we want to
    // deal strictly with the config file.
    let (config, original_config) = {
        // 1. Load config.yaml
        let config = YamlLoader::new(&fs).load_config()?;

        // 2. Apply CLI args to config (overriding)
        (args.apply_to_config(config.clone()), config)
    };

    debug!("Final config: {:#?}", &config);

    // TODO: Maybe don't need to build this until it's needed?
    let reporter = TerminalProgressReporter::new(config.use_colors());

    // 3. Dispatch and execute the requested command
    let exit_code = dispatch_command(&args.command, &config, original_config, reporter).await;

    process::exit(exit_code)
}
