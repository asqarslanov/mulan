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
        println!("{config:?}");
        let input = mulan_parser::Input::read().map_err(|err| miette!("{err:?}"))?;
        println!("{input:?}");
        Ok(ExitCode::SUCCESS)
    }
}
