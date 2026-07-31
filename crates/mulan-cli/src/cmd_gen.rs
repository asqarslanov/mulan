use std::process::ExitCode;

use miette::miette;

#[derive(clap::Args)]
pub struct Args;

impl self::Args {
    /// Executes the subcommand:
    ///
    /// ```sh
    /// $ mulan gen ...
    /// ```
    pub fn execute(self) -> miette::Result<ExitCode> {
        let config = mulan_config::Config::locate_and_read().map_err(|err| miette!("{err:?}"))?;
        let output = mulan_parser::compose(&config).map_err(|err| miette!("{err:?}"))?;
        println!("{output:?}");
        Ok(ExitCode::SUCCESS)
    }
}
