mod cli;
mod commands;
mod config;

use std::process;

use clap::Parser;
use selfie::{
    commands::ShellCommandRunner,
    config::{
        YamlLoader,
        loader::{ApplyToConfg, ConfigLoader},
    },
    fs::real::RealFileSystem,
    progress_reporter::terminal::TerminalProgressReporter,
};
use tracing::debug;

use crate::{cli::ClapCli, commands::dispatch_command};

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let args = ClapCli::parse();
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

    // 3. Create command runner for use by commands that need to execute external programs
    let runner = ShellCommandRunner::new("/bin/sh", config.command_timeout());
    let reporter = TerminalProgressReporter::new(config.use_colors());
    // TODO: Pass runner to commands that need it

    // 4. Dispatch and execute the requested command
    let exit_code = dispatch_command(&args.command, &config, original_config, reporter);

    process::exit(exit_code)
}
