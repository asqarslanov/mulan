use std::process::ExitCode;

/// See [`crate::Command::Init`].
#[derive(clap::Args)]
pub struct Args;

impl self::Args {
    /// Executes the subcommand:
    ///
    /// ```sh
    /// $ mulan init ...
    /// ```
    #[expect(
        clippy::unused_self,
        reason = "to consistently use args.execute() in main"
    )]
    pub fn execute(self) -> miette::Result<ExitCode> {
        todo!();
    }
}
