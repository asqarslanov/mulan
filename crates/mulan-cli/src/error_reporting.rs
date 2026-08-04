//! Conversions from custom Mulan error types to human-readable reports that can
//! be displayed in the CLI.
//!
//! This module defines the [`ToReport`] trait and implements it for all used
//! error types.

use std::fmt::Display;
use std::iter;
use std::range::Range;

use compact_str::{CompactString, CompactStringExt as _, ToCompactString as _, format_compact};
use indoc::{formatdoc, indoc};
use itertools::Itertools as _;
use mitsein::iter1::{IntoIterator1 as _, IteratorExt as _};
use mitsein::small_vec1::SmallVec1;

/// I plan to add support for different cases for identifier names
/// (kebab-case, snake_case, camelCase).
///
/// When I do it, `() = CASE_GUARDRAIL` will fail to compile,
/// so I can refactor these places manually.
const CASE_GUARDRAIL: () = ();

/// A trait to converting strongly typed errors to human-readable
/// [`miette::Report`]s with [`ToReport::to_report`].
///
/// This trait is implemented automatically for all types that implement
/// [`self::ReportData`].
///
/// ---
///
/// Typically, you want to implement [`ToReport`] manually
/// for types that act as branchers for underlying types that also implement
/// [`ToReport`].
///
/// ```ignore
/// impl ToReport for MyError {
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

/// ...
trait ReportData {
    /// ...
    fn message(&self, config: &mulan_config::Config) -> String;

    /// A globally unique diagnostic code in the Rust path format
    /// (e.g., `parser::validate::not_a_message`).
    fn code(&self) -> &'static str;

    /// A user-friendly message on what could be the cause of the error
    /// or how to fix it.
    fn help(&self, config: &mulan_config::Config) -> Option<String>;

    /// ...
    fn source_code_data(&self) -> Option<self::SourceCodeData>;

    /// ...
    fn related(&self, config: &mulan_config::Config) -> impl Iterator<Item = miette::Report>;
}

/// See [`ReportData::source_code_data`].
#[derive(Debug)]
struct SourceCodeData {
    source_code: String,

    /// ...
    file_data: Option<self::SourceCodeFileData>,

    /// ...
    labels: SmallVec1<[self::SourceCodeLabel; 1]>,
}

/// ...
#[derive(Debug)]
struct SourceCodeFileData {
    /// ...
    name: CompactString,

    /// ...
    language: self::SourceCodeLanguage,
}

/// ...
#[derive(Debug)]
enum SourceCodeLanguage {
    Yaml,
}

/// ...
#[derive(Debug)]
struct SourceCodeLabel {
    /// ...
    text: Option<String>,

    /// ...
    span: self::LabelSpan,
}

/// ...
#[allow(dead_code)]
#[derive(Debug)]
enum LabelSpan {
    /// ...
    Index(usize),

    /// ...
    OffsetLen(usize, usize),

    /// ...
    Range(Range<usize>),

    /// ...
    Full,
}

impl<E: ReportData> ToReport for E {
    fn to_report(&self, config: &mulan_config::Config) -> miette::Report {
        let (source_code, labels) = match self.source_code_data() {
            None => (None, None),
            Some(data) => {
                let labels: SmallVec1<[miette::LabeledSpan; 1]> = {
                    let to_label_span = |label: self::SourceCodeLabel| -> miette::LabeledSpan {
                        use self::LabelSpan as S;
                        let span: miette::SourceSpan = match label.span {
                            S::Index(i) => i.into(),
                            S::OffsetLen(offset, len) if len == 1 => offset.into(),
                            S::OffsetLen(offset, len) => (offset..offset + len).into(),
                            S::Range(Range { start, end }) if start + 1 == end => start.into(),
                            S::Range(Range { start, end }) => (start..end).into(),
                            S::Full => (0..=data.source_code.len()).into(),
                        };
                        miette::LabeledSpan::new_with_span(label.text, span)
                    };
                    data.labels.into_iter1().map(to_label_span).collect1()
                };
                let source_code = match data.file_data {
                    Some(file) => {
                        use self::SourceCodeLanguage as L;
                        let l = match file.language {
                            L::Yaml => "YAML",
                        };
                        self::SourceCodeKind::File(
                            miette::NamedSource::new(file.name, data.source_code).with_language(l),
                        )
                    }
                    None => self::SourceCodeKind::Unnamed(data.source_code),
                };
                (Some(source_code), Some(labels))
            }
        };
        let value = self::ReportableError {
            message: self.message(config),
            code: self.code(),
            help: self.help(config),
            source_code,
            labels,
            related: self.related(config).try_collect1().ok(),
        };
        let report = miette::Report::from(value);
        report
    }
}

/// ...
#[derive(Debug, thiserror::Error)]
#[error("{message}")]
struct ReportableError {
    /// The error message for [`std::error::Error`].
    message: String,

    /// The value for [`miette::Diagnostic::code`].
    code: &'static str,

    /// The value for [`miette::Diagnostic::help`].
    help: Option<String>,

    /// The value for [`miette::Diagnostic::source_code`].
    source_code: Option<self::SourceCodeKind>,

    /// The value for [`miette::Diagnostic::labels`].
    labels: Option<SmallVec1<[miette::LabeledSpan; 1]>>,

    /// The value for [`miette::Diagnostic::related`].
    related: Option<SmallVec1<[miette::Report; 1]>>,
}

/// ...
#[derive(Debug)]
enum SourceCodeKind {
    /// ...
    Unnamed(String),

    /// ...
    File(miette::NamedSource<String>),
}

impl miette::Diagnostic for self::ReportableError {
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
            SourceCodeKind::Unnamed(string) => string as &dyn miette::SourceCode,
            SourceCodeKind::File(named_source) => named_source,
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

impl ToReport for mulan_config::errors::ConfigError {
    fn to_report(&self, dummy_config: &mulan_config::Config) -> miette::Report {
        match self {
            Self::Figment(e) => e.to_report(dummy_config),
            Self::Meta(e) => e.to_report(dummy_config),
        }
    }
}

impl ReportData for mulan_config::errors::FigmentError {
    fn message(&self, _: &mulan_config::Config) -> String {
        use figment2::error::Kind as K;
        match &self.inner.kind {
            K::Message(msg) => msg.trim_end().to_owned(),
            K::InvalidType(actual, expected) => formatdoc! {"
                invalid type
                  key `{}`
                  is expected to be `{expected}`
                  but actually has type `{actual}`\
                ",
                (&self.inner.path).join_compact("."),
            },
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
        None
    }

    fn source_code_data(&self) -> Option<self::SourceCodeData> {
        None
    }

    fn related(&self, _: &mulan_config::Config) -> impl Iterator<Item = miette::Report> {
        iter::empty()
    }
}

impl ToReport for mulan_config::errors::MetaError {
    fn to_report(&self, config: &mulan_config::Config) -> miette::Report {
        match self {
            Self::CurrentDir(e) => e.to_report(config),
            Self::SourceNotFound(e) => e.to_report(config),
            Self::AmbiguousSource(e) => e.to_report(config),
        }
    }
}

impl ReportData for mulan_config::errors::CurrentDirError {
    fn message(&self, _: &mulan_config::Config) -> String {
        formatdoc! {"
            failed to get current working directory
            - OS error: {}\
            ",
            self.inner,
        }
    }

    fn code(&self) -> &'static str {
        "config::current_dir"
    }

    fn help(&self, _: &mulan_config::Config) -> Option<String> {
        Some(formatdoc! {"
            make sure that
            - the current working directory exists
            - you have permissions to access it\
        "})
    }

    fn source_code_data(&self) -> Option<self::SourceCodeData> {
        None
    }

    fn related(&self, _: &mulan_config::Config) -> impl Iterator<Item = miette::Report> {
        iter::empty()
    }
}

impl ReportData for mulan_config::errors::SourceNotFoundError {
    fn message(&self, _: &mulan_config::Config) -> String {
        "Mulan config not found in any parent dirctory".to_owned()
    }

    fn code(&self) -> &'static str {
        "config::not_found"
    }

    fn help(&self, _: &mulan_config::Config) -> Option<String> {
        Some(
            indoc! {"
                make sure you're inside your project that uses Mulan
                or run `mulan init` to get started

                Mulan is an i18n framework
                more info: <https://github.com/asqarslanov/mulan>\
            "}
            .to_owned(),
        )
    }

    fn source_code_data(&self) -> Option<self::SourceCodeData> {
        None
    }

    fn related(&self, _config: &mulan_config::Config) -> impl Iterator<Item = miette::Report> {
        iter::empty()
    }
}

impl ReportData for mulan_config::errors::AmbiguousSourceError {
    fn message(&self, _: &mulan_config::Config) -> String {
        "multiple possible config locations".to_owned()
    }

    fn code(&self) -> &'static str {
        "config::ambiguous_source"
    }

    fn help(&self, _: &mulan_config::Config) -> Option<String> {
        Some(format!(
            "only choose one config to remain, and delete the other{}",
            if self.possible_sources.len().get() == 2 {
                ""
            } else {
                "s"
            },
        ))
    }

    fn source_code_data(&self) -> Option<self::SourceCodeData> {
        Some(self::SourceCodeData {
            source_code: self.possible_sources.iter1().into_iter().join("\n"),
            file_data: None,
            labels: {
                let mut line_i_start = 0;
                self.possible_sources
                    .iter1()
                    .enumerate()
                    .map(|(i, path)| {
                        let text = Some(
                            match i {
                                0 => "maybe this?",
                                1 => "or maybe this?",
                                _ => "or maybe even this?",
                            }
                            .to_owned(),
                        );
                        let span = self::LabelSpan::OffsetLen(line_i_start, path.as_str().len());
                        line_i_start += path.as_str().len() + 1;
                        self::SourceCodeLabel { text, span }
                    })
                    .collect1()
            },
        })
    }

    fn related(&self, _config: &mulan_config::Config) -> impl Iterator<Item = miette::Report> {
        iter::empty()
    }
}

impl ToReport for mulan_parser::errors::ComposeError {
    fn to_report(&self, config: &mulan_config::Config) -> miette::Report {
        match self {
            Self::Read(e) => e.to_report(config),
            Self::Transform(e) => e.to_report(config),
        }
    }
}

impl ToReport for mulan_parser::errors::InputError {
    fn to_report(&self, config: &mulan_config::Config) -> miette::Report {
        match self {
            Self::ReadFile(e) => e.to_report(config),
            Self::Yaml(e) => e.to_report(config),
        }
    }
}

impl ReportData for mulan_parser::errors::ReadFileError {
    fn message(&self, _config: &mulan_config::Config) -> String {
        formatdoc! {"
            failed to read {}
            OS error: {}\
            ",
            self.path.display(),
            self.error,
        }
    }

    fn code(&self) -> &'static str {
        "parser::read::fs"
    }

    fn help(&self, _config: &mulan_config::Config) -> Option<String> {
        Some(formatdoc! {"
            make sure
            - {} exists
            - it contains valid UTF-8
            - you have permissions to read it\
            ",
            self.path.display(),
        })
    }

    fn source_code_data(&self) -> Option<self::SourceCodeData> {
        None
    }

    fn related(&self, _config: &mulan_config::Config) -> impl Iterator<Item = miette::Report> {
        iter::empty()
    }
}

impl ReportData for mulan_parser::errors::YamlError {
    fn message(&self, _config: &mulan_config::Config) -> String {
        use serde_saphyr::Error as E;
        match self.inner.without_snippet() {
            E::DuplicateMappingKey { key, location: _ } => {
                format!(
                    "duplicate mapping key{}",
                    key.as_ref()
                        .map_or_else(String::default, |k| format!(": `{k}`")),
                )
            }
            e => e.render(),
        }
    }

    fn code(&self) -> &'static str {
        use serde_saphyr::Error as E;
        match self.inner.without_snippet() {
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
                "remove duplicates to make all keys in the same namespace unique".to_owned()
            }
            _ => return None,
        })
    }

    fn source_code_data(&self) -> Option<self::SourceCodeData> {
        let span = self.inner.location()?.span();
        let offset = usize::try_from(span.offset()).ok()?;
        let len = usize::try_from(span.len()).ok()?;
        Some(self::SourceCodeData {
            source_code: self.source_code.clone(),
            file_data: Some(self::SourceCodeFileData {
                name: self.filename.to_string_lossy().into(),
                language: self::SourceCodeLanguage::Yaml,
            }),
            labels: SmallVec1::from_one(self::SourceCodeLabel {
                text: Some("here".to_owned()),
                span: LabelSpan::OffsetLen(offset, len),
            }),
        })
    }

    fn related(&self, _config: &mulan_config::Config) -> impl Iterator<Item = miette::Report> {
        iter::empty()
    }
}

impl ToReport for mulan_parser::errors::TransformError {
    fn to_report(&self, config: &mulan_config::Config) -> miette::Report {
        match self {
            Self::InvalidSubkey(e) => e.to_report(config),
            Self::InvalidTemplate(e) => e.to_report(config),
            Self::NotANamespace(e) => e.to_report(config),
            Self::NotAMessage(e) => e.to_report(config),
            Self::UnknownParameters(e) => e.to_report(config),
        }
    }
}

impl ReportData for mulan_parser::errors::InvalidSubkeyError {
    fn message(&self, _config: &mulan_config::Config) -> String {
        formatdoc! {"
            found invalid key
              locale: {}
              {}\
            ",
            self.locale.tag(),
            match &self.parent_key {
                Some(key) => format_compact!("namespace `{}`", key.to_compact_string1()),
                None => "root namespace".to_compact_string(),
            },
        }
    }

    fn code(&self) -> &'static str {
        "parser::validate::invalid_key"
    }

    fn help(&self, _config: &mulan_config::Config) -> Option<String> {
        Some("it should look like a variable in a programming language".to_owned())
    }

    fn source_code_data(&self) -> Option<self::SourceCodeData> {
        None
    }

    fn related(&self, config: &mulan_config::Config) -> impl Iterator<Item = miette::Report> {
        iter::once(self.errors.to_report(config))
    }
}

impl ReportData for mulan_parser::errors::InvalidTemplateError {
    fn message(&self, _config: &mulan_config::Config) -> String {
        formatdoc! {"
            found invalid message
              locale: {}
              key: `{}`\
            ",
            self.locale.tag(),
            self.key.to_compact_string1(),
        }
    }

    fn code(&self) -> &'static str {
        "parser::validate::invalid_template"
    }

    fn help(&self, _config: &mulan_config::Config) -> Option<String> {
        None
    }

    fn source_code_data(&self) -> Option<self::SourceCodeData> {
        None
    }

    fn related(&self, config: &mulan_config::Config) -> impl Iterator<Item = miette::Report> {
        iter::once(self.errors.to_report(config))
    }
}

impl ReportData for mulan_parser::errors::NotANamespaceError {
    fn message(&self, _config: &mulan_config::Config) -> String {
        formatdoc! {"
            expected a namespace, found a message
              locale: {}
              key: `{}`\
            ",
            self.locale.tag(),
            self.key.to_compact_string1(),
        }
    }

    fn code(&self) -> &'static str {
        "parser::validate::not_a_namespace"
    }

    fn help(&self, config: &mulan_config::Config) -> Option<String> {
        Some(format!(
            "see how `{}` is defined in {}",
            self.key.to_compact_string1(),
            config.main_locale.tag(),
        ))
    }

    fn source_code_data(&self) -> Option<self::SourceCodeData> {
        None
    }

    fn related(&self, _config: &mulan_config::Config) -> impl Iterator<Item = miette::Report> {
        iter::empty()
    }
}

impl ReportData for mulan_parser::errors::NotAMessageError {
    fn message(&self, _config: &mulan_config::Config) -> String {
        formatdoc! {"
            expected a message, found a namespace
              locale: {}
              key: `{}`\
            ",
            self.locale.tag(),
            self.key.to_compact_string1(),
        }
    }

    fn code(&self) -> &'static str {
        "parser::validate::not_a_message"
    }

    fn help(&self, config: &mulan_config::Config) -> Option<String> {
        Some(format!(
            "see how `{}` is defined in {}",
            self.key.to_compact_string1(),
            config.main_locale.tag(),
        ))
    }

    fn source_code_data(&self) -> Option<self::SourceCodeData> {
        None
    }

    fn related(&self, _config: &mulan_config::Config) -> impl Iterator<Item = miette::Report> {
        iter::empty()
    }
}

impl ReportData for mulan_parser::errors::UnknownParametersError {
    fn message(&self, _config: &mulan_config::Config) -> String {
        formatdoc! {"
            unknown parameters
              locale: {}
              key: `{}`\
            ",
            self.locale.tag(),
            self.key.to_compact_string1(),
        }
    }

    fn code(&self) -> &'static str {
        "parser::validate::unknown_parameters"
    }

    fn help(&self, config: &mulan_config::Config) -> Option<String> {
        Some(formatdoc! {"
            a translation of a message is allowed to have less parameters than the original ({}), \
            but never more\
            ",
            config.main_locale.tag(),
        })
    }

    fn source_code_data(&self) -> Option<self::SourceCodeData> {
        () = CASE_GUARDRAIL;
        Some(self::SourceCodeData {
            source_code: {
                self.parameters
                    .iter1()
                    .map(|param| param.to_kebab_case())
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
                        let param = param.to_kebab_case();
                        let text = Some(
                            match i {
                                0 => "remove this parameter",
                                1 => "and this",
                                _ => "and also this",
                            }
                            .to_owned(),
                        );
                        let span = self::LabelSpan::OffsetLen(line_i_start, param.len().get());
                        line_i_start += param.len().get() + 1;
                        self::SourceCodeLabel { text, span }
                    })
                    .collect1()
            },
        })
    }

    fn related(&self, _config: &mulan_config::Config) -> impl Iterator<Item = miette::Report> {
        iter::empty()
    }
}

impl ReportData for mulan_parser::errors::ChumskyAllErrors {
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

    fn source_code_data(&self) -> Option<self::SourceCodeData> {
        let error = self.errors.first();
        let source = &self.source;
        self::ChumskyErrorWrapper { error, source }.source_code_data()
    }

    fn related(&self, config: &mulan_config::Config) -> impl Iterator<Item = miette::Report> {
        let source = &self.source;
        self.errors
            .iter()
            .map(|error| self::ChumskyErrorWrapper { error, source }.to_report(config))
            .skip(1)
    }
}

/// References a [`mulan_parser::errors::ChumskySingleError`]
/// with its corresponding source code, which makes this type
/// convertible to `miette::Report`.
#[derive(Debug)]
struct ChumskyErrorWrapper<'err> {
    error: &'err mulan_parser::errors::ChumskySingleError,

    /// ...
    source: &'err str,
}

impl ReportData for self::ChumskyErrorWrapper<'_> {
    fn message(&self, _config: &mulan_config::Config) -> String {
        self.error.message.clone()
    }

    fn code(&self) -> &'static str {
        "parser::syntax"
    }

    fn help(&self, _config: &mulan_config::Config) -> Option<String> {
        None
    }

    fn source_code_data(&self) -> Option<self::SourceCodeData> {
        Some(self::SourceCodeData {
            source_code: self.source.to_owned(),
            file_data: None,
            labels: SmallVec1::from_one(self::SourceCodeLabel {
                text: None,
                span: self::LabelSpan::Range(self.error.span),
            }),
        })
    }

    fn related(&self, _config: &mulan_config::Config) -> impl Iterator<Item = miette::Report> {
        iter::empty()
    }
}
