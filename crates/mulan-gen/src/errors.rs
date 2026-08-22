//! Error types.

use std::io;
use std::path::PathBuf;

/// Errors of [`crate::write_files`].
#[derive(Debug)]
pub enum GenError {
    /// The `generate` list in the config is not specified.
    NoTargets(NoTargetsError),

    /// Failed to create a directory.
    CreateDir(CreateDirError),

    /// Failed to write to a file.
    WriteFile(WriteFileError),
}

/// See [`GenError::NoTargets`].
#[derive(Debug)]
pub struct NoTargetsError;

/// See [`GenError::CreateDir`].
#[derive(Debug)]
pub struct CreateDirError {
    pub error: io::Error,

    /// Doesn't contain a slash (`/`) at the end.
    pub path: PathBuf,
}

/// See [`GenError::WriteFile`].
#[derive(Debug)]
pub struct WriteFileError {
    pub error: io::Error,
    pub path: PathBuf,
}
