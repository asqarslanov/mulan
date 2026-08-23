//! Error types.

use std::io;

use mitsein::btree_set1::BTreeSet1;
use relative_path::RelativePathBuf;

/// Errors of [`crate::Config::locate_and_read`].
#[derive(Debug)]
pub enum ConfigError {
    /// An error of the underlying library that handles parsing the config.
    Figment(FigmentError),

    /// An error while obtaining runtime context.
    Meta(MetaError),
}

/// See [`ConfigError::Figment`].
#[derive(Debug)]
pub struct FigmentError {
    pub inner: figment2::Error,
}

/// Errors of [`crate::ConfigMeta::compute`].
#[derive(Debug)]
pub enum MetaError {
    /// Failed to call [`std::env::current_dir`].
    CurrentDir(CurrentDirError),

    /// Unable to locate a config file anywhere.
    SourceNotFound(SourceNotFoundError),

    /// Multiple config files found (only one is permitted).
    AmbiguousSource(AmbiguousSourceError),
}

/// See [`MetaError::CurrentDir`].
#[derive(Debug)]
pub struct CurrentDirError {
    pub inner: io::Error,
}

/// See [`MetaError::SourceNotFound`].
#[derive(Debug)]
pub struct SourceNotFoundError;

/// See [`MetaError::AmbiguousSource`].
#[derive(Debug)]
pub struct AmbiguousSourceError {
    pub possible_sources: BTreeSet1<RelativePathBuf>,
}

/// Errors of [`crate::Config::locate_without_parents`].
#[derive(Debug)]
pub enum LocateError {
    /// ...
    NotFound(NotFoundError),

    /// ...
    Io(LocateIoError),
}

/// See [`LocateError::NotFound`].
#[derive(Debug)]
pub struct NotFoundError;

/// See [`LocateError::Io`].
#[derive(Debug)]
pub struct LocateIoError {
    /// The path we were trying to check the existence of.
    pub path: RelativePathBuf,

    pub error: io::Error,
}
