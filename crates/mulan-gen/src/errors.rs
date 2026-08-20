//! Error types.

use std::io;

#[derive(Debug)]
pub enum WriteError {
    CreateDir(io::Error),
}
