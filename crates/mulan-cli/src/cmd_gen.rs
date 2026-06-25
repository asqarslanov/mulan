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
        let input = mulan_parser::Input::read().map_err(|err| miette!("{err:?}"));
        println!("{input:?}");
        Ok(ExitCode::SUCCESS)
    }
}
