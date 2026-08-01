use miette::{LabeledSpan, MietteDiagnostic};

/// ...
pub trait ToReport {
    /// ...
    fn to_report(&self, config: &mulan_config::Config) -> miette::Report;
}

impl<E: ReportData> ToReport for E {
    fn to_report(&self, config: &mulan_config::Config) -> miette::Report {
        MietteDiagnostic {
            message: self.message(config),
            code: Some(self.code().to_owned()),
            severity: Some(miette::Severity::Error),
            help: self.help(config),
            url: None,
            labels: self.labels(config),
        }
        .into()
    }
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
    fn source_code(&self) -> Option<String>;

    /// ...
    fn labels(&self, config: &mulan_config::Config) -> Option<Vec<LabeledSpan>>;
}

impl ToReport for mulan_parser::errors::ComposeError {
    fn to_report(&self, config: &mulan_config::Config) -> miette::Report {
        match self {
            Self::Read(err) => err.to_report(config),
            Self::Transform(err) => err.to_report(config),
        }
    }
}

impl ReportData for mulan_parser::errors::InputError {
    fn message(&self, config: &mulan_config::Config) -> String {
        todo!()
    }

    fn code(&self) -> &'static str {
        match self {
            Self::ReadFile { .. } => "parser::input::io",
            Self::Format(_) => "parser::input::format",
        }
    }

    fn help(&self, config: &mulan_config::Config) -> Option<String> {
        todo!()
    }

    fn source_code(&self) -> Option<String> {
        todo!()
    }

    fn labels(&self, config: &mulan_config::Config) -> Option<Vec<LabeledSpan>> {
        todo!()
    }
}

impl ReportData for mulan_parser::errors::TransformError {
    fn message(&self, config: &mulan_config::Config) -> String {
        todo!()
    }

    fn code(&self) -> &'static str {
        match self {
            Self::LocaleNotFound(_) => "parser::transform::locale_not_found",
            Self::InvalidSubkey { .. } => "parser::transform::invalid_subkey",
            Self::InvalidTemplate { .. } => "parser::transform::invalid_template",
            Self::NotANamespace { .. } => "parser::transform::not_a_namespace",
            Self::NotAMessage { .. } => "parser::transform::not_a_message",
            Self::UnknownParameters { .. } => "parser::transform::unknown_parameters",
        }
    }

    fn help(&self, config: &mulan_config::Config) -> Option<String> {
        todo!()
    }

    fn source_code(&self) -> Option<String> {
        todo!()
    }

    fn labels(&self, config: &mulan_config::Config) -> Option<Vec<LabeledSpan>> {
        todo!()
    }
}
