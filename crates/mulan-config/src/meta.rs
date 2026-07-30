//! See [`ConfigMeta`].

use std::ffi::OsStr;
use std::path::PathBuf;
use std::{env, io};

use figment2::Figment;
use itertools::Itertools as _;
use mitsein::btree_set1::BTreeSet1;
use mitsein::iter1::IteratorExt as _;
use relative_path::RelativePathBuf;

/// See [`crate::Config::meta`]. Build with [`ConfigMeta::compute`].
#[derive(Debug, Default, PartialEq, Eq)]
pub struct ConfigMeta {
    /// See [`std::env::current_dir`].
    pub current_dir: PathBuf,

    /// The path of the project root directory Mulan is operating on,
    /// relative to [`Self::current_dir`].
    pub root_dir: RelativePathBuf,
}

/// Errors of [`ConfigMeta::compute`].
#[derive(Debug)]
pub enum MetaError {
    /// Failed to call [`std::env::current_dir`].
    CurrentDir(io::Error),

    /// Unable to locate a config file anywhere.
    SourceNotFound,

    /// Multiple config files found (only one is permitted).
    AmbiguousSource(BTreeSet1<RelativePathBuf>),
}

impl ConfigMeta {
    /// Obtain runtime context needed for the config.
    pub(super) fn compute(figment: &Figment) -> Result<Self, MetaError> {
        let current_dir = env::current_dir().map_err(MetaError::CurrentDir)?;
        let (root_dir, _config_file) = {
            figment
                .metadata()
                .filter_map(|metadata| {
                    let source_absolute = {
                        metadata
                            .source
                            .as_ref()
                            .expect("all sources are predetermined")
                            .file_path()
                            .expect("config is only read from a file")
                    };
                    if source_absolute.is_relative() {
                        return None;
                    }
                    let (root_dir_absolute, config_file) = {
                        source_absolute
                            .parent()
                            .zip(source_absolute.file_name().and_then(OsStr::to_str))
                            .expect("config source should point to a file")
                    };
                    let root_dir_raw = pathdiff::diff_paths(root_dir_absolute, &current_dir)
                        .expect("current_dir can be subtracted from config source");
                    let root_dir = RelativePathBuf::from_path(root_dir_raw)
                        .expect("pathdiff::diff_paths returns a relative path");
                    Some((root_dir, config_file))
                })
                .exactly_one()
                .map_err(|locations| {
                    locations
                        .try_into_iter1()
                        .map_or(MetaError::SourceNotFound, |sources_raw| {
                            let sources = {
                                sources_raw
                                    .map(|(root_dir, config_file)| root_dir.join(config_file))
                                    .collect1()
                            };
                            MetaError::AmbiguousSource(sources)
                        })
                })?
        };
        Ok(Self {
            current_dir,
            root_dir,
        })
    }
}
