use std::process::ExitCode;

use anyhow::Result;

use crate::cli::Commands;

mod cli;
mod commands;

fn main() -> Result<ExitCode> {
    let cli = &*cli::CLI;

    match cli.command {
        Commands::Test => return commands::test::main().map(|()| ExitCode::SUCCESS),
        Commands::Init(_) => {
            if !crate::commands::init::init()? {
                return Ok(ExitCode::FAILURE);
            }
        }
        Commands::Apply(_) => todo!(),
    }

    Ok(ExitCode::SUCCESS)
}
