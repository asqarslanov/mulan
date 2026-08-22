//! Error types.

use std::io;

/// Errors of [`crate::write_files`].
#[derive(Debug)]
pub enum GenError {
    /// ...
    NoTargets,

    /// ...
    CreateDir(CreateDirError),

    /// ...
    WriteFile(WriteFileError),
}

/// ...
#[derive(Debug)]
pub struct CreateDirError {
    pub inner: io::Error,
}

/// ...
#[derive(Debug)]
pub struct WriteFileError {
    pub inner: io::Error,
}
