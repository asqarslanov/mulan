//! Error types.

use std::io;

use mitsein::btree_set1::BTreeSet1;
use relative_path::RelativePathBuf;

/// Errors of [`crate::Config::locate_and_read`].
#[derive(Debug)]
pub enum ConfigError {
    /// An error of the underlying library that handles parsing the config.
    Figment(figment2::Error),

    /// An error while obtaining runtime context.
    Meta(MetaError),
}

/// Errors of [`crate::ConfigMeta::compute`].
#[derive(Debug)]
pub enum MetaError {
    /// Failed to call [`std::env::current_dir`].
    CurrentDir(io::Error),

    /// Unable to locate a config file anywhere.
    SourceNotFound,

    /// Multiple config files found (only one is permitted).
    AmbiguousSource(BTreeSet1<RelativePathBuf>),
}
