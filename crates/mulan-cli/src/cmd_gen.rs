use std::process::ExitCode;

use crate::error_reporting::ToReport as _;

#[derive(clap::Args)]
pub struct Args;

impl self::Args {
    /// Executes the subcommand:
    ///
    /// ```sh
    /// $ mulan gen ...
    /// ```
    #[expect(
        clippy::unused_self,
        reason = "to consistently use args.execute() in main"
    )]
    pub fn execute(self) -> miette::Result<ExitCode> {
        let config = mulan_config::Config::locate_and_read()
            .map_err(|err| err.to_report(&mulan_config::Config::dummy()))?;
        let output = mulan_parser::compose(&config).map_err(|err| err.to_report(&config))?;
        #[expect(
            clippy::print_stdout,
            reason = "debug since there's nothing better to print, yet"
        )]
        {
            println!("{output:?}");
        }
        Ok(ExitCode::SUCCESS)
    }
}
