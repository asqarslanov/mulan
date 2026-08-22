//! Error types.

use std::io;
use std::path::PathBuf;

/// Errors of [`crate::write_files`].
#[derive(Debug)]
pub enum GenError {
    /// ...
    NoTargets(NoTargetsError),

    /// ...
    CreateDir(CreateDirError),

    /// ...
    WriteFile(WriteFileError),
}

/// ...
#[derive(Debug)]
pub struct NoTargetsError;

/// ...
#[derive(Debug)]
pub struct CreateDirError {
    pub error: io::Error,
    pub path: PathBuf,
}

/// ...
#[derive(Debug)]
pub struct WriteFileError {
    pub error: io::Error,
    pub path: PathBuf,
}
