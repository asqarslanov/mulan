use std::process::ExitCode;

use clap::crate_description;

mod cmd_gen;
mod cmd_init;
mod error_reporting;
#[rustfmt::skip]
mod i18n;

fn main() -> miette::Result<ExitCode> {
    let cli = <self::Cli as clap::Parser>::parse();
    match cli.command {
        Command::Gen => self::cmd_gen::execute(),
        Command::Init => self::cmd_init::execute(),
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
    ///
    Init,

    /// Generate i18n bindings for your targets
    Gen,
}
