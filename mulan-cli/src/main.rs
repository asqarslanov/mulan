use anyhow::Result;

use crate::cli::Commands;

mod cli;
mod commands;

fn main() -> Result<()> {
    let cli = &*cli::CLI;

    match cli.command {
        Commands::Init(_) => crate::commands::init::init()?,
        Commands::Run => todo!(),
    }

    std::fs::File::open("notexists")?;

    Ok(())
}
