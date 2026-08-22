//! Error types.

use std::io;

#[derive(Debug)]
pub enum GenError {
    NoTargets,
    CreateDir(io::Error),
    WriteFile(io::Error),
}
