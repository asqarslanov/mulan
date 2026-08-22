use std::process::ExitCode;

use clap::crate_description;

mod cmd_gen;
mod error_reporting;
mod i18n;

fn main() -> miette::Result<ExitCode> {
    let cli = <self::Cli as clap::Parser>::parse();
    match cli.command {
        Command::Gen(args) => args.execute(),
    }
}

#[derive(clap::Parser)]
#[command(about = crate_description!(), version)]
struct Cli {
    #[command(subcommand)]
    command: self::Command,
}

#[derive(clap::Subcommand)]
enum Command {
    /// Generate i18n bindings for your targets
    Gen(self::cmd_gen::Args),
}
