use std::io;
use std::process::ExitCode;

/// Ctrl+C interruption exit code.
const SIGINT: u8 = 128 + 2;

/// Executes the subcommand:
///
/// ```sh
/// $ mulan init ...
/// ```
pub fn execute() -> miette::Result<ExitCode> {
    match interactive_prompt() {
        Ok(()) => Ok(ExitCode::SUCCESS),
        Err(err) if matches!(err.kind(), io::ErrorKind::Interrupted) => Ok(ExitCode::from(SIGINT)),
        Err(err) => Err(err).unwrap(),
    }
}

fn interactive_prompt() -> io::Result<()> {
    cliclack::intro("Mulan")?;
    let a: String = cliclack::input("test").interact()?;
    cliclack::outro("You're all set")?;
    Ok(())
}
