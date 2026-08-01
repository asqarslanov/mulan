use std::range::Range;

use compact_str::CompactString;
use mitsein::vec1::Vec1;

/// ...
pub trait ToReport {
    /// ...
    fn to_report(&self, config: &mulan_config::Config) -> miette::Report;
}

impl<E: ReportData> ToReport for E {
    fn to_report(&self, config: &mulan_config::Config) -> miette::Report {
        let (source_data, labels) = match self.source_code_data() {
            None => (None, None),
            Some(SourceCodeData {
                source_code,
                file_data,
                labels,
            }) => {
                let labeled_spans: Vec<miette::LabeledSpan> = {
                    labels
                        .into_iter()
                        .map(|label| {
                            let span: miette::SourceSpan = match label.span {
                                LabelSpan::Index(i) => i.into(),
                                LabelSpan::Range(Range { start, end }) => (start..end).into(),
                                LabelSpan::Full => (0..=source_code.len()).into(),
                            };
                            miette::LabeledSpan::new_with_span(label.text, span)
                        })
                        .collect()
                };
                (Some((source_code, file_data)), Some(labeled_spans))
            }
        };
        let mut report: miette::Report = miette::MietteDiagnostic {
            message: self.message(config),
            code: Some(self.code().to_owned()),
            severity: Some(miette::Severity::Error),
            help: self.help(config),
            url: None,
            labels,
        }
        .into();
        if let Some((source_code, file_data)) = source_data {
            report = match file_data {
                Some(file) => report.with_source_code(
                    miette::NamedSource::new(file.name, source_code).with_language(file.language),
                ),
                None => report.with_source_code(source_code),
            };
        }
        report
    }
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
#[derive(Debug)]
enum LabelSpan {
    /// ...
    Index(usize),

    /// ...
    Range(Range<usize>),

    /// ...
    Full,
}

/// ...
#[derive(Debug)]
struct SourceCodeData {
    source_code: String,

    /// ...
    file_data: Option<self::SourceCodeFileData>,

    /// ...
    labels: Vec1<self::SourceCodeLabel>,
}

/// ...
#[derive(Debug)]
struct SourceCodeFileData {
    /// ...
    name: CompactString,

    /// ...
    language: &'static str,
}

/// ...
trait ReportData {
    /// ...
    fn message(&self, config: &mulan_config::Config) -> String;

    /// ...
    fn code(&self) -> &'static str;

    /// ...
    fn help(&self, config: &mulan_config::Config) -> Option<String>;

    /// ...
    fn source_code_data(&self) -> Option<self::SourceCodeData>;
}

impl ToReport for mulan_config::errors::ConfigError {
    fn to_report(&self, config: &mulan_config::Config) -> miette::Report {
        match self {
            Self::Figment(e) => e.to_report(config),
            Self::Meta(e) => e.to_report(config),
        }
    }
}

impl ReportData for mulan_config::errors::FigmentError {}

impl ToReport for mulan_config::errors::MetaError {
    fn to_report(&self, config: &mulan_config::Config) -> miette::Report {
        match self {
            Self::CurrentDir(e) => e.to_report(config),
            Self::SourceNotFound(e) => e.to_report(config),
            Self::AmbiguousSource(e) => e.to_report(config),
        }
    }
}

impl ReportData for mulan_config::errors::CurrentDirError {}

impl ReportData for mulan_config::errors::SourceNotFoundError {}

impl ReportData for mulan_config::errors::AmbiguousSourceError {}

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

impl ReportData for mulan_parser::errors::ReadFileError {}

impl ReportData for mulan_parser::errors::YamlError {}

impl ToReport for mulan_parser::errors::TransformError {
    fn to_report(&self, config: &mulan_config::Config) -> miette::Report {
        match self {
            Self::LocaleNotFound(e) => e.to_report(config),
            Self::InvalidSubkey(e) => e.to_report(config),
            Self::InvalidTemplate(e) => e.to_report(config),
            Self::NotANamespace(e) => e.to_report(config),
            Self::NotAMessage(e) => e.to_report(config),
            Self::UnknownParameters(e) => e.to_report(config),
        }
    }
}

impl ReportData for mulan_parser::errors::LocaleNotFoundError {}

impl ReportData for mulan_parser::errors::InvalidSubkeyError {}

impl ReportData for mulan_parser::errors::InvalidTemplateError {}

impl ReportData for mulan_parser::errors::NotANamespaceError {}

impl ReportData for mulan_parser::errors::NotAMessageError {}

impl ReportData for mulan_parser::errors::UnknownParametersError {}
