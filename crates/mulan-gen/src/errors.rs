//! Error types.

use std::io;

/// Errors of [`crate::write_files`].
#[derive(Debug)]
pub enum GenError {
    /// ...
    NoTargets,

    /// ...
    CreateDir(io::Error),

    /// ...
    WriteFile(io::Error),
}
