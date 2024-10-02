use std::sync::LazyLock;

use clap::{crate_description, Args, Parser, Subcommand};

#[derive(Parser)]
#[command(about = crate_description!(), version, propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Generate locale bindings [WIP]
    #[command(alias = "r")]
    Run,
    /// Initialize [WIP]
    Init(InitArgs),
}

#[derive(Args)]
pub struct RunArgs {}

#[derive(Args)]
pub struct InitArgs {}

pub static CLI: LazyLock<Cli> = LazyLock::new(Cli::parse);
