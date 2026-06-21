use std::process::ExitCode;

#[derive(clap::Args)]
pub struct Args;

impl self::Args {
    /// Execute the subcommand:
    ///
    /// ```sh
    /// $ mulan gen ...
    /// ```
    pub fn execute(self) -> miette::Result<ExitCode> {
        todo!();
    }
}
