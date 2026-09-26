//! Conversions from custom Mulan error types to human-readable reports that can
//! be displayed in the CLI.
//!
//! This module defines the [`ToReport`] trait and implements it for all used
//! error types.

use std::fmt::Display;
use std::iter;
use std::path::PathBuf;
use std::range::Range;

use compact_str::{CompactStringExt as _, ToCompactString as _, format_compact};
use itertools::Itertools as _;
use mitsein::iter1::{IntoIterator1 as _, IteratorExt as _};
use mitsein::small_vec1::SmallVec1;

use crate::i18n::{Locale, t};

/// A trait to converting strongly typed errors to human-readable
/// [`miette::Report`]s with [`ToReport::to_report`].
///
/// This trait is implemented automatically for all types that implement
/// [`self::Reportable`]. If possible, prefer implementing [`Reportable`].
///
/// ---
///
/// Typically, you want to implement [`ToReport`] manually
/// for types that act as branchers for underlying types that also implement
/// [`ToReport`].
///
/// ```ignore
/// impl self::ToReport for MyError {
///     fn to_report(&self, config: &mulan_config::Config) -> miette::Report {
///         match self {
///             Self::Foo(e) => e.to_report(config),
///             Self::Bar(e) => e.to_report(config),
///         }
///     }
/// }
/// ```
pub trait ToReport {
    /// Converts this error to a [`miette::Report`], respecting the user's
    /// config.
    fn to_report(&self, config: &mulan_config::Config) -> miette::Report;
}

/// Rich metadata about an error to turn into a pretty human-readable report.
///
/// Implement this for error types that represent a specific kind of error
/// (can be assigned a [`self::Reportable::code`]).
trait Reportable {
    /// The main reason this error has occured, displayed to the user.
    fn message(&self, config: &mulan_config::Config) -> String;

    /// A globally unique diagnostic code in the Rust path format
    /// (e.g., `parser::validate::not_a_message`).
    fn code(&self) -> &'static str;

    /// A user-friendly message on what could be the cause of the error
    /// or how to fix it.
    fn help(&self, config: &mulan_config::Config) -> Option<String>;

    /// A text block with annotations (labels with arrows) that expains
    /// the error visually.
    ///
    /// Most often, used to show the source code.
    fn annotation_block(&self, config: &mulan_config::Config) -> Option<self::AnnotationBlock>;

    /// Additional reports displayed under this one.
    ///
    /// This method is used for stacking multiple reports together.
    /// If this is not what you want, simply return [`iter::empty()`].
    fn related(&self, config: &mulan_config::Config) -> impl Iterator<Item = miette::Report>;
}

/// The return type of [`self::Reportable::annotation_block`].
#[derive(Debug)]
struct AnnotationBlock {
    /// The text to annotate. E.g., source code.
    text: String,

    /// Used if this block refers to text from a file.
    file_data: Option<self::SourceFileData>,

    /// Annotations that point to specific parts of the text with arrows.
    ///
    /// At least one label must be present for the block to render.
    labels: SmallVec1<[self::SourceLabel; 1]>,
}

/// See [`self::AnnotationBlock::file_data`].
#[derive(Debug)]
struct SourceFileData {
    /// The filename relative to the project root where the error has occured.
    name: PathBuf,

    /// The markup language name used for syntax highlighting.
    language: self::SourceLanguage,
}

/// See [`self::AnnotationBlock::labels`].
#[derive(Debug)]
struct SourceLabel {
    /// The message of this annotation.
    ///
    /// Short text with a concise explanation. Or just `"here"`.
    text: String,

    /// What exact part of the source text this annotation refers to.
    span: self::SpanKind,
}

/// See [`self::SourceLabel::span`].
#[expect(
    dead_code,
    reason = "some variants aren't used yet, though they can be useful"
)]
#[derive(Debug)]
enum SpanKind {
    /// A single problematic byte (0-based index).
    Index(usize),

    /// A problematic byte range (offset from the start and length, 0-based).
    OffsetLen(usize, usize),

    /// A problematic byte range (0-based indexing).
    Range(Range<usize>),

    /// Signifies that the entire source is problematic
    /// (use with caution, check the output first).
    Full,
}

/// See [`self::SourceFileData::language`].
#[derive(Debug)]
enum SourceLanguage {
    Yaml,
}

impl<E: self::Reportable> self::ToReport for E {
    fn to_report(&self, config: &mulan_config::Config) -> miette::Report {
        let (source_code, labels) = match self.annotation_block(config) {
            None => (None, None),
            Some(data) => {
                let labels: SmallVec1<[miette::LabeledSpan; 1]> = {
                    let to_label_span = |label: self::SourceLabel| -> miette::LabeledSpan {
                        use self::SpanKind as S;
                        let span: miette::SourceSpan = match label.span {
                            S::Index(i) => i.into(),
                            S::OffsetLen(offset, 1) => offset.into(),
                            S::OffsetLen(offset, len) => (offset..offset + len).into(),
                            S::Range(Range { start, end }) if start + 1 == end => start.into(),
                            S::Range(Range { start, end }) => (start..end).into(),
                            S::Full => (0..=data.text.len()).into(),
                        };
                        miette::LabeledSpan::new_with_span(Some(label.text), span)
                    };
                    data.labels.into_iter1().map(to_label_span).collect1()
                };
                let source_code = match data.file_data {
                    Some(file) => {
                        use self::SourceLanguage as L;
                        let l = match file.language {
                            L::Yaml => "YAML",
                        };
                        self::SourceKind::File(
                            miette::NamedSource::new(file.name.to_string_lossy(), data.text)
                                .with_language(l),
                        )
                    }
                    None => self::SourceKind::Unnamed(data.text),
                };
                (Some(source_code), Some(labels))
            }
        };
        miette::Report::from(self::ReportData {
            message: self.message(config),
            code: self.code(),
            help: self.help(config),
            source_code,
            labels,
            related: self.related(config).try_collect1().ok(),
        })
    }
}

/// A structure that contains all data an error report might need to implement
/// [`miette::Diagnostic`].
///
/// All [`self::Reportable`] errors in this file are first converted
/// to [`self::ReportData`] before being transformed to a [`miette::Report`].
#[derive(Debug, thiserror::Error)]
#[error("{message}")]
struct ReportData {
    /// The error message for [`std::error::Error`].
    message: String,

    /// The value for [`miette::Diagnostic::code`].
    code: &'static str,

    /// The value for [`miette::Diagnostic::help`].
    help: Option<String>,

    /// The value for [`miette::Diagnostic::source_code`].
    source_code: Option<self::SourceKind>,

    /// The value for [`miette::Diagnostic::labels`].
    labels: Option<SmallVec1<[miette::LabeledSpan; 1]>>,

    /// The value for [`miette::Diagnostic::related`].
    related: Option<SmallVec1<[miette::Report; 1]>>,
}

/// Represents different kinds of values that implement [`miette::SourceCode`].
#[derive(Debug)]
enum SourceKind {
    /// Plain text without syntax highligthing and file data.
    Unnamed(String),

    /// Refers to the contents of a file. Can have syntax highligthing.
    File(miette::NamedSource<String>),
}

impl miette::Diagnostic for self::ReportData {
    fn code(&self) -> Option<Box<dyn Display>> {
        Some(Box::new(self.code))
    }

    fn severity(&self) -> Option<miette::Severity> {
        Some(miette::Severity::Error)
    }

    fn help(&self) -> Option<Box<dyn Display + '_>> {
        self.help.as_ref().map(|s| Box::new(s) as _)
    }

    fn url(&self) -> Option<Box<dyn Display>> {
        None
    }

    fn source_code(&self) -> Option<&dyn miette::SourceCode> {
        self.source_code.as_ref().map(|kind| match kind {
            SourceKind::Unnamed(string) => string as &dyn miette::SourceCode,
            SourceKind::File(named_source) => named_source,
        })
    }

    fn labels(&self) -> Option<Box<dyn Iterator<Item = miette::LabeledSpan> + '_>> {
        self.labels
            .clone()
            .map(|labels| Box::new(labels.into_iter()) as _)
    }

    fn related(&self) -> Option<Box<dyn Iterator<Item = &dyn miette::Diagnostic> + '_>> {
        self.related
            .as_ref()
            .map(|related| Box::new(related.iter().map(AsRef::as_ref)) as _)
    }

    fn diagnostic_source(&self) -> Option<&dyn miette::Diagnostic> {
        None
    }
}

impl self::Reportable for crate::cmd_init::ConfigExistsError {
    fn message(&self, _: &mulan_config::Config) -> String {
        let path = self.path.as_str();
        t::errors::cli::init::config::already_exists::Message { path }.get_in(Locale::default())
    }

    fn code(&self) -> &'static str {
        "cli::init::config::already_exists"
    }

    fn help(&self, _: &mulan_config::Config) -> Option<String> {
        None
    }

    fn annotation_block(&self, _: &mulan_config::Config) -> Option<self::AnnotationBlock> {
        None
    }

    fn related(&self, _: &mulan_config::Config) -> impl Iterator<Item = miette::Report> {
        iter::empty()
    }
}

impl self::ToReport for crate::cmd_init::NewConfigError {
    fn to_report(&self, dummy_config: &mulan_config::Config) -> miette::Report {
        match self {
            Self::Create(e) => e.to_report(dummy_config),
            Self::Write(e) => e.to_report(dummy_config),
        }
    }
}

impl self::Reportable for crate::cmd_init::CreateConfigError {
    fn message(&self, _: &mulan_config::Config) -> String {
        let os_error = &self.error.to_compact_string();
        let path = self.path.as_str();
        t::errors::cli::init::config::create_file::Message { os_error, path }
            .get_in(Locale::default())
    }

    fn code(&self) -> &'static str {
        "cli::init::config::create_file"
    }

    fn help(&self, _: &mulan_config::Config) -> Option<String> {
        let message = t::errors::cli::init::config::create_file::Help.get_in(Locale::default());
        Some(message.to_owned())
    }

    fn annotation_block(&self, _: &mulan_config::Config) -> Option<self::AnnotationBlock> {
        None
    }

    fn related(&self, _: &mulan_config::Config) -> impl Iterator<Item = miette::Report> {
        iter::empty()
    }
}

impl self::Reportable for crate::cmd_init::WriteConfigError {
    fn message(&self, _: &mulan_config::Config) -> String {
        let os_error = &self.error.to_compact_string();
        let path = self.path.as_str();
        t::errors::cli::init::config::write_file::Message { os_error, path }
            .get_in(Locale::default())
    }

    fn code(&self) -> &'static str {
        "cli::init::config::write_file"
    }

    fn help(&self, _: &mulan_config::Config) -> Option<String> {
        let message = t::errors::cli::init::config::write_file::Help.get_in(Locale::default());
        Some(message.to_owned())
    }

    fn annotation_block(&self, _: &mulan_config::Config) -> Option<self::AnnotationBlock> {
        None
    }

    fn related(&self, _: &mulan_config::Config) -> impl Iterator<Item = miette::Report> {
        iter::empty()
    }
}

impl self::ToReport for crate::cmd_init::CreateLocalesError {
    fn to_report(&self, dummy_config: &mulan_config::Config) -> miette::Report {
        match self {
            Self::CreateDir(e) => e.to_report(dummy_config),
            Self::CreateFile(e) => e.to_report(dummy_config),
            Self::WriteFile(e) => e.to_report(dummy_config),
        }
    }
}

impl self::Reportable for crate::cmd_init::CreateLocalesDirError {
    fn message(&self, _: &mulan_config::Config) -> String {
        let os_error = &self.error.to_compact_string();
        let path = self.path.as_str();
        t::errors::cli::init::locale::create_dir::Message { os_error, path }
            .get_in(Locale::default())
    }

    fn code(&self) -> &'static str {
        "cli::init::locale::create_dir"
    }

    fn help(&self, _: &mulan_config::Config) -> Option<String> {
        let message = t::errors::cli::init::locale::create_dir::Help.get_in(Locale::default());
        Some(message.to_owned())
    }

    fn annotation_block(&self, _: &mulan_config::Config) -> Option<self::AnnotationBlock> {
        None
    }

    fn related(&self, _: &mulan_config::Config) -> impl Iterator<Item = miette::Report> {
        iter::empty()
    }
}

impl self::Reportable for crate::cmd_init::CreateLocaleFileError {
    fn message(&self, _: &mulan_config::Config) -> String {
        let os_error = &self.error.to_compact_string();
        let path = self.path.as_str();
        t::errors::cli::init::locale::create_file::Message { os_error, path }
            .get_in(Locale::default())
    }

    fn code(&self) -> &'static str {
        "cli::init::locale::create_file"
    }

    fn help(&self, _: &mulan_config::Config) -> Option<String> {
        let message = t::errors::cli::init::locale::create_file::Help.get_in(Locale::default());
        Some(message.to_owned())
    }

    fn annotation_block(&self, _: &mulan_config::Config) -> Option<self::AnnotationBlock> {
        None
    }

    fn related(&self, _: &mulan_config::Config) -> impl Iterator<Item = miette::Report> {
        iter::empty()
    }
}

impl self::Reportable for crate::cmd_init::WriteLocaleFileError {
    fn message(&self, _: &mulan_config::Config) -> String {
        let os_error = &self.error.to_compact_string();
        let path = self.path.as_str();
        t::errors::cli::init::locale::write_file::Message { os_error, path }
            .get_in(Locale::default())
    }

    fn code(&self) -> &'static str {
        "cli::init::locale::write_file"
    }

    fn help(&self, _: &mulan_config::Config) -> Option<String> {
        let message = t::errors::cli::init::locale::write_file::Help.get_in(Locale::default());
        Some(message.to_owned())
    }

    fn annotation_block(&self, _: &mulan_config::Config) -> Option<self::AnnotationBlock> {
        None
    }

    fn related(&self, _: &mulan_config::Config) -> impl Iterator<Item = miette::Report> {
        iter::empty()
    }
}

impl self::ToReport for mulan_config::errors::ConfigError {
    fn to_report(&self, dummy_config: &mulan_config::Config) -> miette::Report {
        match self {
            Self::Figment(e) => e.to_report(dummy_config),
            Self::Meta(e) => e.to_report(dummy_config),
        }
    }
}

impl self::Reportable for mulan_config::errors::FigmentError {
    fn message(&self, _: &mulan_config::Config) -> String {
        use figment2::error::Kind as K;
        match &self.inner.kind {
            K::Message(msg) => msg.trim_end().to_owned(),
            K::InvalidType(actual, expected) => t::errors::config::parse::invalid_type::Message {
                actual: &actual.to_compact_string(),
                expected,
                key: &(&self.inner.path).join_compact("."),
            }
            .get_in(Locale::default()),
            K::UnknownVariant(actual, expected) => {
                t::errors::config::parse::unknown_variant::Message {
                    actual: &if actual.is_empty() {
                        " an empty string".to_compact_string()
                    } else {
                        format_compact!(": `{actual}`")
                    },
                    expected: &{
                        expected
                            .iter()
                            .map(|variant| format_compact!("`{variant}`"))
                            .join_compact(", ")
                    },
                    key: &(&self.inner.path).join_compact("."),
                }
                .get_in(Locale::default())
            }
            K::UnknownField(actual, _) => {
                t::errors::config::parse::unknown_field::Message { actual }
                    .get_in(Locale::default())
            }
            _ => self.inner.to_string(),
        }
    }

    fn code(&self) -> &'static str {
        use figment2::error::Kind as K;
        match self.inner.kind {
            K::InvalidType(_, _) => "config::parse::invalid_type",
            K::InvalidValue(_, _) => "config::parse::invalid_value",
            K::InvalidLength(_, _) => "config::parse::invalid_length",
            K::UnknownVariant(_, _) => "config::parse::unknown_variant",
            K::UnknownField(_, _) => "config::parse::unknown_field",
            K::MissingField(_) => "config::parse::missing_field",
            K::DuplicateField(_) => "config::parse::duplicate_field",
            K::ISizeOutOfRange(_) => "config::parse::isize_out_of_range",
            K::USizeOutOfRange(_) => "config::parse::usize_out_of_range",
            K::Unsupported(_) => "config::parse::unsupported",
            K::UnsupportedKey(_, _) => "config::parse::unsupported_key",
            K::Message(_) => "config::parse",
        }
    }

    fn help(&self, _: &mulan_config::Config) -> Option<String> {
        use figment2::error::Kind as K;
        match self.inner.kind {
            K::UnknownField(_, _) => {
                let msg = t::errors::config::parse::unknown_field::Help.get_in(Locale::default());
                Some(msg.to_owned())
            }
            _ => None,
        }
    }

    fn annotation_block(&self, _: &mulan_config::Config) -> Option<self::AnnotationBlock> {
        None
    }

    fn related(&self, _: &mulan_config::Config) -> impl Iterator<Item = miette::Report> {
        iter::empty()
    }
}

impl self::ToReport for mulan_config::errors::MetaError {
    fn to_report(&self, config: &mulan_config::Config) -> miette::Report {
        match self {
            Self::CurrentDir(e) => e.to_report(config),
            Self::SourceNotFound(e) => e.to_report(config),
            Self::AmbiguousSource(e) => e.to_report(config),
        }
    }
}

impl self::Reportable for mulan_config::errors::CurrentDirError {
    fn message(&self, _: &mulan_config::Config) -> String {
        let os_error = &self.inner.to_compact_string();
        t::errors::config::current_dir::Message { os_error }.get_in(Locale::default())
    }

    fn code(&self) -> &'static str {
        "config::current_dir"
    }

    fn help(&self, _: &mulan_config::Config) -> Option<String> {
        let message = t::errors::config::current_dir::Help.get_in(Locale::default());
        Some(message.to_owned())
    }

    fn annotation_block(&self, _: &mulan_config::Config) -> Option<self::AnnotationBlock> {
        None
    }

    fn related(&self, _: &mulan_config::Config) -> impl Iterator<Item = miette::Report> {
        iter::empty()
    }
}

impl self::Reportable for mulan_config::errors::SourceNotFoundError {
    fn message(&self, _: &mulan_config::Config) -> String {
        let message = t::errors::config::not_found::Message.get_in(Locale::default());
        message.to_owned()
    }

    fn code(&self) -> &'static str {
        "config::not_found"
    }

    fn help(&self, _: &mulan_config::Config) -> Option<String> {
        let message = t::errors::config::not_found::Help.get_in(Locale::default());
        Some(message.to_owned())
    }

    fn annotation_block(&self, _: &mulan_config::Config) -> Option<self::AnnotationBlock> {
        None
    }

    fn related(&self, _: &mulan_config::Config) -> impl Iterator<Item = miette::Report> {
        iter::empty()
    }
}

impl self::Reportable for mulan_config::errors::AmbiguousSourceError {
    fn message(&self, _: &mulan_config::Config) -> String {
        let message = t::errors::config::ambiguous_source::Message.get_in(Locale::default());
        message.to_owned()
    }

    fn code(&self) -> &'static str {
        "config::ambiguous_source"
    }

    fn help(&self, _: &mulan_config::Config) -> Option<String> {
        let plural_ending = if self.possible_sources.len().get() == 2 {
            ""
        } else {
            "s"
        };
        Some(t::errors::config::ambiguous_source::Help { plural_ending }.get_in(Locale::default()))
    }

    fn annotation_block(&self, _: &mulan_config::Config) -> Option<self::AnnotationBlock> {
        Some(self::AnnotationBlock {
            text: self.possible_sources.iter1().into_iter().join("\n"),
            file_data: None,
            labels: {
                let mut line_i_start = 0;
                self.possible_sources
                    .iter1()
                    .enumerate()
                    .map(|(i, path)| {
                        let text = match i {
                            0 => t::errors::config::ambiguous_source::annotation_block::FirstLabel
                                .get_in(Locale::default()),
                            1 => t::errors::config::ambiguous_source::annotation_block::SecondLabel
                                .get_in(Locale::default()),
                            _ => t::errors::config::ambiguous_source::annotation_block::OtherLabels
                                .get_in(Locale::default()),
                        }
                        .to_owned();
                        let span = self::SpanKind::OffsetLen(line_i_start, path.as_str().len());
                        line_i_start += path.as_str().len() + 1;
                        self::SourceLabel { text, span }
                    })
                    .collect1()
            },
        })
    }

    fn related(&self, _config: &mulan_config::Config) -> impl Iterator<Item = miette::Report> {
        iter::empty()
    }
}

impl self::Reportable for mulan_config::errors::LocateIoError {
    fn message(&self, _: &mulan_config::Config) -> String {
        let os_error = &self.error.to_compact_string();
        let path = self.path.as_str();
        t::errors::config::locate::io::Message { os_error, path }.get_in(Locale::default())
    }

    fn code(&self) -> &'static str {
        "config::locate::io"
    }

    fn help(&self, _: &mulan_config::Config) -> Option<String> {
        let message = t::errors::config::locate::io::Help.get_in(Locale::default());
        Some(message.to_owned())
    }

    fn annotation_block(&self, _: &mulan_config::Config) -> Option<self::AnnotationBlock> {
        None
    }

    fn related(&self, _: &mulan_config::Config) -> impl Iterator<Item = miette::Report> {
        iter::empty()
    }
}

impl self::ToReport for mulan_parser::errors::BundleFromFsError {
    fn to_report(&self, config: &mulan_config::Config) -> miette::Report {
        match self {
            Self::Read(e) => e.to_report(config),
            Self::Transform(e) => e.to_report(config),
        }
    }
}

impl self::ToReport for mulan_parser::errors::LocaleMapError {
    fn to_report(&self, config: &mulan_config::Config) -> miette::Report {
        match self {
            Self::ReadFile(e) => e.to_report(config),
            Self::Yaml(e) => e.to_report(config),
        }
    }
}

impl self::Reportable for mulan_parser::errors::ReadFileError {
    fn message(&self, _config: &mulan_config::Config) -> String {
        let os_error = &self.error.to_compact_string();
        let path = &self.path.to_string_lossy();
        t::errors::parser::read::fs::Message { os_error, path }.get_in(Locale::default())
    }

    fn code(&self) -> &'static str {
        "parser::read::fs"
    }

    fn help(&self, _config: &mulan_config::Config) -> Option<String> {
        let path = &self.path.to_string_lossy();
        Some(t::errors::parser::read::fs::Help { path }.get_in(Locale::default()))
    }

    fn annotation_block(&self, _config: &mulan_config::Config) -> Option<self::AnnotationBlock> {
        None
    }

    fn related(&self, _config: &mulan_config::Config) -> impl Iterator<Item = miette::Report> {
        iter::empty()
    }
}

impl self::Reportable for mulan_parser::errors::YamlError {
    fn message(&self, _config: &mulan_config::Config) -> String {
        use serde_saphyr::Error as E;
        match self.inner.without_snippet() {
            E::DuplicateMappingKey { key, location: _ } => {
                let key = &{
                    key.as_ref()
                        .map_or_else(String::default, |k| format!(": `{k}`"))
                };
                t::errors::parser::read::yaml::duplicate_mapping_key::Message { key }
                    .get_in(Locale::default())
            }
            e => e.render(),
        }
    }

    #[expect(
        clippy::too_many_lines,
        reason = "
            This error type is external.
            Remapping everything to an in-house data type would be insane.
            I match against every variant to observe whether new variants
            are added with new updates.
        "
    )]
    fn code(&self) -> &'static str {
        use serde_saphyr::Error as E;
        match self.inner.without_snippet() {
            E::InvalidOptions { .. } => "parser::read::yaml::invalid_options",
            E::Eof { .. } => "parser::read::yaml::eof",
            E::MultipleDocuments { .. } => "parser::read::yaml::multiple_documents",
            E::Unexpected { .. } => "parser::read::yaml::unexpected",
            E::MergeValueNotMapOrSeqOfMaps { .. } => {
                "parser::read::yaml::merge_value_not_map_or_seq_of_maps"
            }
            E::MergeKeyNotAllowed { .. } => "parser::read::yaml::merge_key_not_allowed",
            E::InvalidBinaryBase64 { .. } => "parser::read::yaml::invalid_binary_base64",
            E::BinaryNotUtf8 { .. } => "parser::read::yaml::binary_not_utf8",
            E::TaggedScalarCannotDeserializeIntoString { .. } => {
                "parser::read::yaml::tagged_scalar_cannot_deserialize_into_string"
            }
            E::UnexpectedSequenceEnd { .. } => "parser::read::yaml::unexpected_sequence_end",
            E::UnexpectedMappingEnd { .. } => "parser::read::yaml::unexpected_mapping_end",
            E::InvalidBooleanStrict { .. } => "parser::read::yaml::invalid_boolean_strict",
            E::InvalidCharNull { .. } => "parser::read::yaml::invalid_char_null",
            E::InvalidCharNotSingleScalar { .. } => {
                "parser::read::yaml::invalid_char_not_single_scalar"
            }
            E::NullIntoString { .. } => "parser::read::yaml::null_into_string",
            E::BytesNotSupportedMissingBinaryTag { .. } => {
                "parser::read::yaml::bytes_not_supported_missing_binary_tag"
            }
            E::UnexpectedValueForUnit { .. } => "parser::read::yaml::unexpected_value_for_unit",
            E::ExpectedEmptyMappingForUnitStruct { .. } => {
                "parser::read::yaml::expected_empty_mapping_for_unit_struct"
            }
            E::UnexpectedContainerEndWhileSkippingNode { .. } => {
                "parser::read::yaml::unexpected_container_end_while_skipping_node"
            }
            E::InternalSeedReusedForMapKey { .. } => {
                "parser::read::yaml::internal_seed_reused_for_map_key"
            }
            E::ValueRequestedBeforeKey { .. } => "parser::read::yaml::value_requested_before_key",
            E::ExpectedStringKeyForExternallyTaggedEnum { .. } => {
                "parser::read::yaml::expected_string_key_for_externally_tagged_enum"
            }
            E::ExternallyTaggedEnumExpectedScalarOrMapping { .. } => {
                "parser::read::yaml::externally_tagged_enum_expected_scalar_or_mapping"
            }
            E::UnexpectedValueForUnitEnumVariant { .. } => {
                "parser::read::yaml::unexpected_value_for_unit_enum_variant"
            }
            E::InvalidUtf8Input => "parser::read::yaml::invalid_utf8_input",
            E::AliasReplayCounterOverflow { .. } => {
                "parser::read::yaml::alias_replay_counter_overflow"
            }
            E::AliasReplayLimitExceeded { .. } => "parser::read::yaml::alias_replay_limit_exceeded",
            E::AliasExpansionLimitExceeded { .. } => {
                "parser::read::yaml::alias_expansion_limit_exceeded"
            }
            E::AliasReplayStackDepthExceeded { .. } => {
                "parser::read::yaml::alias_replay_stack_depth_exceeded"
            }
            E::FoldedBlockScalarMustIndentContent { .. } => {
                "parser::read::yaml::folded_block_scalar_must_indent_content"
            }
            E::InternalDepthUnderflow { .. } => "parser::read::yaml::internal_depth_underflow",
            E::InternalRecursionStackEmpty { .. } => {
                "parser::read::yaml::internal_recursion_stack_empty"
            }
            E::RecursiveReferencesRequireWeakTypes { .. } => {
                "parser::read::yaml::recursive_references_require_weak_types"
            }
            E::InvalidScalar { .. } => "parser::read::yaml::invalid_scalar",
            E::NonFiniteFloat { .. } => "parser::read::yaml::non_finite_float",
            E::SerdeInvalidType { .. } => "parser::read::yaml::serde_invalid_type",
            E::SerdeInvalidValue { .. } => "parser::read::yaml::serde_invalid_value",
            E::SerdeUnknownVariant { .. } => "parser::read::yaml::serde_unknown_variant",
            E::SerdeUnknownField { .. } => "parser::read::yaml::serde_unknown_field",
            E::SerdeMissingField { .. } => "parser::read::yaml::serde_missing_field",
            E::UnexpectedContainerEndWhileReadingKeyNode { .. } => {
                "parser::read::yaml::unexpected_container_end_while_reading_key_node"
            }
            E::DuplicateMappingKey { .. } => "parser::read::yaml::duplicate_mapping_key",
            E::TaggedEnumMismatch { .. } => "parser::read::yaml::tagged_enum_mismatch",
            E::SerdeVariantId { .. } => "parser::read::yaml::serde_variant_id",
            E::ExpectedMappingEndAfterEnumVariantValue { .. } => {
                "parser::read::yaml::expected_mapping_end_after_enum_variant_value"
            }
            E::ContainerEndMismatch { .. } => "parser::read::yaml::container_end_mismatch",
            E::UnknownAnchor { .. } => "parser::read::yaml::unknown_anchor",
            E::CyclicInclude { .. } => "parser::read::yaml::cyclic_include",
            E::UnsupportedIncludeForm { .. } => "parser::read::yaml::unsupported_include_form",
            E::ResolverError { .. } => "parser::read::yaml::resolver",
            E::AliasError { .. } => "parser::read::yaml::alias",
            E::HookError { .. } => "parser::read::yaml::hook",
            E::UnresolvedProperty { .. } => "parser::read::yaml::unresolved_property",
            E::InvalidPropertyName { .. } => "parser::read::yaml::invalid_property_name",
            E::PropertyRequiredButUnset { .. } => "parser::read::yaml::property_required_but_unset",
            E::PropertyRequiredButEmpty { .. } => "parser::read::yaml::property_required_but_empty",
            E::Budget { .. } => "parser::read::yaml::budget",
            E::IOError { .. } => "parser::read::yaml::io",
            E::QuotingRequired { .. } => "parser::read::yaml::quoting_required",
            E::CannotBorrowTransformedString { .. } => {
                "parser::read::yaml::cannot_borrow_transformed_string"
            }
            E::IndentationError { .. } => "parser::read::yaml::indentation",
            _ => "parser::read::yaml",
        }
    }

    fn help(&self, _config: &mulan_config::Config) -> Option<String> {
        use serde_saphyr::Error as E;
        Some(match self.inner.without_snippet() {
            E::DuplicateMappingKey { .. } => {
                t::errors::parser::read::yaml::duplicate_mapping_key::Help
                    .get_in(Locale::default())
                    .to_owned()
            }
            _ => return None,
        })
    }

    fn annotation_block(&self, _config: &mulan_config::Config) -> Option<self::AnnotationBlock> {
        let span = self.inner.location()?.span();
        let offset = usize::try_from(span.offset()).ok()?;
        let len = usize::try_from(span.len()).ok()?;
        Some(self::AnnotationBlock {
            text: self.source_code.clone(),
            file_data: Some(self::SourceFileData {
                name: self.filename.clone(),
                language: self::SourceLanguage::Yaml,
            }),
            labels: SmallVec1::from_one(self::SourceLabel {
                text: t::errors::parser::read::yaml::annotation_block::Here
                    .get_in(Locale::default())
                    .to_owned(),
                span: SpanKind::OffsetLen(offset, len),
            }),
        })
    }

    fn related(&self, _config: &mulan_config::Config) -> impl Iterator<Item = miette::Report> {
        iter::empty()
    }
}

impl self::ToReport for mulan_parser::errors::TransformError {
    fn to_report(&self, config: &mulan_config::Config) -> miette::Report {
        match self {
            Self::InvalidKey(e) => e.to_report(config),
            Self::InvalidTemplate(e) => e.to_report(config),
            Self::NotANamespace(e) => e.to_report(config),
            Self::NotAMessage(e) => e.to_report(config),
            Self::UnknownParameters(e) => e.to_report(config),
        }
    }
}

impl self::Reportable for mulan_parser::errors::InvalidKeyError {
    fn message(&self, _config: &mulan_config::Config) -> String {
        let locale = self.locale.tag();
        let parent_key = &self.parent_key.as_ref().map_or_else(
            || "root namespace".to_compact_string(),
            |key| format_compact!("namespace: `{}`", key.to_compact_string1()),
        );
        t::errors::parser::validate::invalid_key::Message { locale, parent_key }
            .get_in(Locale::default())
    }

    fn code(&self) -> &'static str {
        "parser::validate::invalid_key"
    }

    fn help(&self, _config: &mulan_config::Config) -> Option<String> {
        let message = t::errors::parser::validate::invalid_key::Help.get_in(Locale::default());
        Some(message.to_owned())
    }

    fn annotation_block(&self, _config: &mulan_config::Config) -> Option<self::AnnotationBlock> {
        None
    }

    fn related(&self, config: &mulan_config::Config) -> impl Iterator<Item = miette::Report> {
        iter::once(self.errors.to_report(config))
    }
}

impl self::Reportable for mulan_parser::errors::InvalidTemplateError {
    fn message(&self, _config: &mulan_config::Config) -> String {
        let locale = self.locale.tag();
        let key = &self.key.to_compact_string1();
        t::errors::parser::validate::invalid_template::Message { locale, key }
            .get_in(Locale::default())
    }

    fn code(&self) -> &'static str {
        "parser::validate::invalid_template"
    }

    fn help(&self, _config: &mulan_config::Config) -> Option<String> {
        None
    }

    fn annotation_block(&self, _config: &mulan_config::Config) -> Option<self::AnnotationBlock> {
        None
    }

    fn related(&self, config: &mulan_config::Config) -> impl Iterator<Item = miette::Report> {
        iter::once(self.errors.to_report(config))
    }
}

impl self::Reportable for mulan_parser::errors::NotANamespaceError {
    fn message(&self, _config: &mulan_config::Config) -> String {
        let locale = self.locale.tag();
        let key = &self.key.to_compact_string1();
        t::errors::parser::validate::not_a_namespace::Message { locale, key }
            .get_in(Locale::default())
    }

    fn code(&self) -> &'static str {
        "parser::validate::not_a_namespace"
    }

    fn help(&self, config: &mulan_config::Config) -> Option<String> {
        let key = &self.key.to_compact_string1();
        let main_locale = config.main_locale.tag();
        let message = t::errors::parser::validate::not_a_namespace::Help { key, main_locale };
        Some(message.get_in(Locale::default()))
    }

    fn annotation_block(&self, _config: &mulan_config::Config) -> Option<self::AnnotationBlock> {
        None
    }

    fn related(&self, _config: &mulan_config::Config) -> impl Iterator<Item = miette::Report> {
        iter::empty()
    }
}

impl self::Reportable for mulan_parser::errors::NotAMessageError {
    fn message(&self, _config: &mulan_config::Config) -> String {
        let locale = self.locale.tag();
        let key = &self.key.to_compact_string1();
        t::errors::parser::validate::not_a_message::Message { locale, key }
            .get_in(Locale::default())
    }

    fn code(&self) -> &'static str {
        "parser::validate::not_a_message"
    }

    fn help(&self, config: &mulan_config::Config) -> Option<String> {
        let key = &self.key.to_compact_string1();
        let main_locale = config.main_locale.tag();
        let message = t::errors::parser::validate::not_a_message::Help { key, main_locale };
        Some(message.get_in(Locale::default()))
    }

    fn annotation_block(&self, _config: &mulan_config::Config) -> Option<self::AnnotationBlock> {
        None
    }

    fn related(&self, _config: &mulan_config::Config) -> impl Iterator<Item = miette::Report> {
        iter::empty()
    }
}

impl self::Reportable for mulan_parser::errors::UnknownParametersError {
    fn message(&self, _config: &mulan_config::Config) -> String {
        let locale = self.locale.tag();
        let key = &self.key.to_compact_string1();
        t::errors::parser::validate::unknown_parameters::Message { locale, key }
            .get_in(Locale::default())
    }

    fn code(&self) -> &'static str {
        "parser::validate::unknown_parameters"
    }

    fn help(&self, config: &mulan_config::Config) -> Option<String> {
        let main_locale = config.main_locale.tag();
        let message = t::errors::parser::validate::unknown_parameters::Help { main_locale };
        Some(message.get_in(Locale::default()))
    }

    fn annotation_block(&self, config: &mulan_config::Config) -> Option<self::AnnotationBlock> {
        use t::errors::parser::validate::unknown_parameters::annotation_block as tt;
        Some(self::AnnotationBlock {
            text: {
                self.parameters
                    .iter1()
                    .map(|param| param.to_compact_string1(config.key_case))
                    .into_iter()
                    .join("\n")
            },
            file_data: None,
            labels: {
                let mut line_i_start = 0;
                self.parameters
                    .iter1()
                    .enumerate()
                    .map(|(i, param)| {
                        let param = param.to_compact_string1(config.key_case);
                        let text = match i {
                            0 => tt::FirstLabel.get_in(Locale::default()),
                            1 => tt::SecondLabel.get_in(Locale::default()),
                            _ => tt::OtherLabels.get_in(Locale::default()),
                        }
                        .to_owned();
                        let span = self::SpanKind::OffsetLen(line_i_start, param.len().get());
                        line_i_start += param.len().get() + 1;
                        self::SourceLabel { text, span }
                    })
                    .collect1()
            },
        })
    }

    fn related(&self, _config: &mulan_config::Config) -> impl Iterator<Item = miette::Report> {
        iter::empty()
    }
}

impl self::Reportable for mulan_parser::errors::ChumskyAllErrors {
    fn message(&self, config: &mulan_config::Config) -> String {
        let error = self.errors.first();
        let source = &self.source;
        self::ChumskyErrorWrapper { error, source }.message(config)
    }

    fn code(&self) -> &'static str {
        let error = self.errors.first();
        let source = &self.source;
        self::ChumskyErrorWrapper { error, source }.code()
    }

    fn help(&self, config: &mulan_config::Config) -> Option<String> {
        let error = self.errors.first();
        let source = &self.source;
        self::ChumskyErrorWrapper { error, source }.help(config)
    }

    fn annotation_block(&self, config: &mulan_config::Config) -> Option<self::AnnotationBlock> {
        let error = self.errors.first();
        let source = &self.source;
        self::ChumskyErrorWrapper { error, source }.annotation_block(config)
    }

    fn related(&self, config: &mulan_config::Config) -> impl Iterator<Item = miette::Report> {
        let source = &self.source;
        let to_report = |error| self::ChumskyErrorWrapper { error, source }.to_report(config);
        self.errors.iter().map(to_report).skip(1)
    }
}

/// References a [`mulan_parser::errors::ChumskySingleError`]
/// with its corresponding source code, which makes this type
/// convertible to [`miette::Report`].
#[derive(Debug)]
struct ChumskyErrorWrapper<'err> {
    error: &'err mulan_parser::errors::ChumskySingleError,

    /// The original string we failed to parse.
    source: &'err str,
}

impl self::Reportable for self::ChumskyErrorWrapper<'_> {
    fn message(&self, _config: &mulan_config::Config) -> String {
        self.error.message.clone()
    }

    fn code(&self) -> &'static str {
        "parser::syntax"
    }

    fn help(&self, _config: &mulan_config::Config) -> Option<String> {
        None
    }

    fn annotation_block(&self, _config: &mulan_config::Config) -> Option<self::AnnotationBlock> {
        Some(self::AnnotationBlock {
            text: self.source.to_owned(),
            file_data: None,
            labels: SmallVec1::from_one(self::SourceLabel {
                text: t::errors::parser::syntax::annotation_block::Here
                    .get_in(Locale::default())
                    .to_owned(),
                span: self::SpanKind::Range(self.error.span),
            }),
        })
    }

    fn related(&self, _config: &mulan_config::Config) -> impl Iterator<Item = miette::Report> {
        iter::empty()
    }
}

impl self::ToReport for mulan_gen::errors::GenError {
    fn to_report(&self, config: &mulan_config::Config) -> miette::Report {
        match self {
            Self::NoTargets(e) => e.to_report(config),
            Self::CreateDir(e) => e.to_report(config),
            Self::WriteFile(e) => e.to_report(config),
        }
    }
}

impl self::Reportable for mulan_gen::errors::NoTargetsError {
    fn message(&self, _config: &mulan_config::Config) -> String {
        t::errors::generate::no_targets::Message
            .get_in(Locale::default())
            .to_owned()
    }

    fn code(&self) -> &'static str {
        "gen::no_targets"
    }

    fn help(&self, _config: &mulan_config::Config) -> Option<String> {
        let message = t::errors::generate::no_targets::Help.get_in(Locale::default());
        Some(message.to_owned())
    }

    fn annotation_block(&self, _config: &mulan_config::Config) -> Option<self::AnnotationBlock> {
        None
    }

    fn related(&self, _config: &mulan_config::Config) -> impl Iterator<Item = miette::Report> {
        iter::empty()
    }
}

impl self::Reportable for mulan_gen::errors::CreateDirError {
    fn message(&self, _config: &mulan_config::Config) -> String {
        let os_error = &self.error.to_compact_string();
        let path = &self.path.to_string_lossy();
        t::errors::generate::create_dir::Message { os_error, path }.get_in(Locale::default())
    }

    fn code(&self) -> &'static str {
        "gen::create_dir"
    }

    fn help(&self, _config: &mulan_config::Config) -> Option<String> {
        let message = t::errors::generate::create_dir::Help.get_in(Locale::default());
        Some(message.to_owned())
    }

    fn annotation_block(&self, _config: &mulan_config::Config) -> Option<self::AnnotationBlock> {
        None
    }

    fn related(&self, _config: &mulan_config::Config) -> impl Iterator<Item = miette::Report> {
        iter::empty()
    }
}

impl self::Reportable for mulan_gen::errors::WriteFileError {
    fn message(&self, _config: &mulan_config::Config) -> String {
        let os_error = &self.error.to_compact_string();
        let path = &self.path.to_string_lossy();
        t::errors::generate::write_file::Message { os_error, path }.get_in(Locale::default())
    }

    fn code(&self) -> &'static str {
        "gen::write_file"
    }

    fn help(&self, _config: &mulan_config::Config) -> Option<String> {
        let message = t::errors::generate::write_file::Help.get_in(Locale::default());
        Some(message.to_owned())
    }

    fn annotation_block(&self, _config: &mulan_config::Config) -> Option<self::AnnotationBlock> {
        None
    }

    fn related(&self, _config: &mulan_config::Config) -> impl Iterator<Item = miette::Report> {
        iter::empty()
    }
}
