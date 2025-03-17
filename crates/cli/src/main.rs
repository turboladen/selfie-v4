mod cli;

use clap::Parser;
use cli::ClapCli;

fn main() {
    let args = ClapCli::parse();
    dbg!(&args);
    todo!()
}
