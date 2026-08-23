use std::process::ExitCode;

/// Executes the subcommand:
///
/// ```sh
/// $ mulan init ...
/// ```
#[expect(
    clippy::unwrap_used,
    reason = "
        A `cliclack` error indicates that we can't print to the console.
        There's no meaningful strategy to recover from this.
        Also, there's no point in creating Miette reports---
        we won't probably be able to print them anyway.
    "
)]
pub fn execute() -> miette::Result<ExitCode> {
    cliclack::intro("Hello").unwrap();
    todo!();
}
