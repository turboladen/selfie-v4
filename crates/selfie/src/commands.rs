//! Command execution abstractions and implementations

pub mod runner;
pub mod shell;

#[cfg(test)]
mod tests;

pub use runner::{CommandError, CommandOutput, CommandRunner, OutputChunk};
pub use shell::ShellCommandRunner;
