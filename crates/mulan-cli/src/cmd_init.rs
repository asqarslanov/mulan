use std::io;
use std::process::ExitCode;

use mulan_config::errors::{LocateError, NotFoundError};
use relative_path::RelativePathBuf;

use crate::error_reporting::ToReport;

/// Ctrl+C interruption exit code.
const SIGINT: u8 = 128 + 2;

/// Executes the subcommand:
///
/// ```sh
/// $ mulan init ...
/// ```
pub fn execute() -> miette::Result<ExitCode> {
    match mulan_config::Config::locate_without_parents() {
        Ok(path) => {
            return Err(ConfigExistsError { path }.to_report(&mulan_config::Config::dummy()));
        }
        Err(LocateError::Io(e)) => return Err(e.to_report(&mulan_config::Config::dummy())),
        Err(LocateError::NotFound(NotFoundError)) => (),
    }

    match interactive_prompt() {
        Ok(()) => Ok(ExitCode::SUCCESS),
        Err(err) if matches!(err.kind(), io::ErrorKind::Interrupted) => Ok(ExitCode::from(SIGINT)),
        Err(err) => Err(err).unwrap(),
    }
}

#[derive(Debug)]
pub struct ConfigExistsError {
    pub path: RelativePathBuf,
}

fn interactive_prompt() -> io::Result<()> {
    cliclack::intro("Mulan")?;
    let a: String = cliclack::input("test").interact()?;
    cliclack::outro("You're all set")?;
    Ok(())
}
