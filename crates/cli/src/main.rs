mod cli;
mod config;

use clap::Parser;
use selfie::{
    adapters::config_loader,
    config::ports::{ApplyToConfg, ConfigLoader},
    filesystem::adapter::RealFileSystem,
};
use tracing::debug;

use crate::cli::ClapCli;

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let args = ClapCli::parse();
    debug!("CLI arguments: {:#?}", &args);

    let fs = RealFileSystem;

    let config = {
        // 1. Load config.yaml
        let config = config_loader::Yaml::new(&fs).load_config()?;

        // 2. Apply CLI args to config (overriding)
        args.apply_to_config(config)
    };

    debug!("Final config: {:#?}", &config);

    // 3. Run command

    Ok(())
}
