mod cli;
mod commands;
mod config;

use clap::Parser;
use selfie::{
    command::runner::ShellCommandRunner,
    config::loader::{self, ApplyToConfg, ConfigLoader},
    filesystem::real::RealFileSystem,
};
use tracing::debug;

use crate::{cli::ClapCli, commands::dispatch_command};

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let args = ClapCli::parse();
    debug!("CLI arguments: {:#?}", &args);

    let fs = RealFileSystem;

    let config = {
        // 1. Load config.yaml
        let config = loader::Yaml::new(&fs).load_config()?;

        // 2. Apply CLI args to config (overriding)
        args.apply_to_config(config)
    };

    debug!("Final config: {:#?}", &config);

    // 3. Create command runner for use by commands that need to execute external programs
    let runner = ShellCommandRunner::new("/bin/sh", config.command_timeout());
    // TODO: Pass runner to commands that need it

    // 4. Dispatch and execute the requested command
    dispatch_command(&args.command, &config)?;

    Ok(())
}
