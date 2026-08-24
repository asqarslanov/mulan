use std::fs::{self, File};
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::ExitCode;

use itertools::Itertools as _;
use mitsein::small_vec1::SmallVec1;
use mulan_config::errors::{LocateError, NotFoundError};
use mulan_config::{Language, RustTarget, Target};
use relative_path::RelativePathBuf;
use serde::Serialize;

use crate::error_reporting::ToReport as _;
use crate::i18n::{Locale, t};

/// Ctrl+C interruption exit code.
const SIGINT: u8 = 128 + 2;

/// Executes the subcommand:
///
/// ```sh
/// $ mulan init ...
/// ```
pub fn execute() -> miette::Result<ExitCode> {
    match mulan_config::Config::locate_without_parents() {
        Ok(path) => {
            return Err(ConfigExistsError { path }.to_report(&mulan_config::Config::dummy()));
        }
        Err(LocateError::Io(e)) => return Err(e.to_report(&mulan_config::Config::dummy())),
        Err(LocateError::NotFound(NotFoundError)) => (),
    }
    match prompt_and_init() {
        Ok(()) => Ok(ExitCode::SUCCESS),
        Err(PromptAndInitError::Cancel) => {
            // Cancelled at the end.
            Ok(ExitCode::from(SIGINT))
        }
        Err(PromptAndInitError::Cliclack(err))
            if matches!(err.kind(), io::ErrorKind::Interrupted) =>
        {
            // Ctrl+C interrupt.
            Ok(ExitCode::from(SIGINT))
        }
        Err(PromptAndInitError::Cliclack(err)) => {
            // A `cliclack` error indicates that we couldn't print to the console.
            // There's no meaningful recovery strategy, so we just `panic!`.
            // There's no point in creating rich Miette reports---
            // we won't probably be able to print them anyway.
            panic!("{err}");
        }
        Err(PromptAndInitError::Miette(report)) => Err(report),
    }
}

/// A Mulan config already exists, so we can't initialize a new one.
#[derive(Debug)]
pub struct ConfigExistsError {
    /// The path of the existing Mulan config.
    pub path: RelativePathBuf,
}

/// Errors of [`prompt_and_init`].
#[derive(Debug)]
enum PromptAndInitError {
    /// The user manually didn't confirm initialization.
    Cancel,

    /// A [`mod@cliclack`] error
    /// (e.g., Ctrl+C or couldn't print to the console).
    Cliclack(io::Error),

    /// A ready-to-print report.
    Miette(miette::Report),
}

/// Shows an interactive [`mod@cliclack`] menu and initializes Mulan
/// in the current directory.
fn prompt_and_init() -> Result<(), PromptAndInitError> {
    cliclack::intro(t::cmd_init::Intro.get_in(Locale::default()))
        .map_err(PromptAndInitError::Cliclack)?;
    let init_options = InitConfig::interactive_prompt().map_err(PromptAndInitError::Cliclack)?;
    let confirm = {
        cliclack::confirm(t::cmd_init::PromptConfirm.get_in(Locale::default()))
            .initial_value(true)
            .interact()
            .map_err(PromptAndInitError::Cliclack)?
    };
    if !confirm {
        cliclack::outro_cancel(t::cmd_init::Canceled.get_in(Locale::default()))
            .map_err(PromptAndInitError::Cliclack)?;
        return Err(PromptAndInitError::Cancel);
    }
    let config_path = {
        init_options.write_to_file().map_err(|err| {
            PromptAndInitError::Miette(err.to_report(&mulan_config::Config::dummy()))
        })?
    };
    cliclack::note(
        "",
        t::cmd_init::CreatedConfig {
            path: config_path.as_str(),
        }
        .get_in(Locale::default()),
    )
    .map_err(PromptAndInitError::Cliclack)?;
    let create_locales = {
        cliclack::confirm(t::cmd_init::PromptCreateLocales.get_in(Locale::default()))
            .initial_value(true)
            .interact()
            .map_err(PromptAndInitError::Cliclack)?
    };
    if create_locales {
        create_locale_files(&init_options.locales).map_err(|err| {
            PromptAndInitError::Miette(err.to_report(&mulan_config::Config::dummy()))
        })?;
        cliclack::note("", t::cmd_init::DefinedLocales.get_in(Locale::default()))
            .map_err(PromptAndInitError::Cliclack)?;
    }
    cliclack::outro(t::cmd_init::Outro.get_in(Locale::default()))
        .map_err(PromptAndInitError::Cliclack)?;
    Ok(())
}

/// A simplified version of [`mulan_config::Config`]
/// the user builds with a [`mod@cliclack`] interactive menu.
#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
struct InitConfig {
    /// Maps to [`mulan_config::Config::locales`].
    locales: Vec<Language>,

    /// Maps to [`mulan_config::Config::main_locale`].
    main_locale: Language,

    /// Maps to [`mulan_config::Config::generate`].
    generate: Option<SmallVec1<[Target; 1]>>,
}

/// Couldn't create a new Mulan config.
#[derive(Debug)]
pub enum NewConfigError {
    Create(CreateConfigError),
    Write(WriteConfigError),
}

/// See [`NewConfigError::Create`].
#[derive(Debug)]
pub struct CreateConfigError {
    pub error: io::Error,

    /// The path where we tried to create a config.
    pub path: RelativePathBuf,
}

/// See [`NewConfigError::Write`].
#[derive(Debug)]
pub struct WriteConfigError {
    pub error: io::Error,

    /// The path to the config we were trying to write to.
    pub path: RelativePathBuf,
}

impl InitConfig {
    /// Requests values to build this type from the user
    /// via an interactive [`mod@cliclack`] menu.
    fn interactive_prompt() -> io::Result<Self> {
        let locales = Self::prompt_locales(&[Language::EnUs, Language::RuRu])?;
        let main_locale = Self::prompt_main_locale(&locales)?;
        let generate = Self::prompt_generate()?.map(SmallVec1::from_one);
        Ok(Self {
            locales,
            main_locale,
            generate,
        })
    }

    /// Creates a new config file, writes data to it in a pretty form.
    fn write_to_file(&self) -> Result<RelativePathBuf, NewConfigError> {
        let path = RelativePathBuf::from("mulan.toml");
        let contents = toml::to_string_pretty(self).expect("should never fail");
        let mut file = match File::create_new(path.as_str()) {
            Ok(file) => file,
            Err(error) => return Err(NewConfigError::Create(CreateConfigError { error, path })),
        };
        if let Err(error) = file.write_all(contents.as_bytes()) {
            return Err(NewConfigError::Write(WriteConfigError { error, path }));
        }
        Ok(path)
    }

    /// Request a [`Self::locales`] value from the user.
    fn prompt_locales(all_possible_locales: &[Language]) -> io::Result<Vec<Language>> {
        let collect_vec = {
            all_possible_locales
                .iter()
                .map(|&lang| (lang, lang.tag(), lang.name()))
                .collect_vec()
        };
        cliclack::multiselect(t::cmd_init::Locales.get_in(Locale::default()))
            .items(&collect_vec)
            .interact()
    }

    /// Request a [`Self::main_locale`] value from the user.
    fn prompt_main_locale(locales: &[Language]) -> io::Result<Language> {
        let items = {
            locales
                .iter()
                .map(|&lang| (lang, lang.tag(), lang.name()))
                .collect_vec()
        };
        cliclack::select(t::cmd_init::MainLocale.get_in(Locale::default()))
            .items(&items)
            .interact()
    }

    /// Request a [`Self::generate`] value from the user.
    fn prompt_generate() -> io::Result<Option<Target>> {
        let add_rust_target: bool = {
            let prompt = t::cmd_init::generate::Prompt.get_in(Locale::default());
            let yes_label = t::cmd_init::generate::YesLabel.get_in(Locale::default());
            let yes_hint = t::cmd_init::generate::YesHint.get_in(Locale::default());
            let no_label = t::cmd_init::generate::NoLabel.get_in(Locale::default());
            let no_hint = t::cmd_init::generate::NoHint.get_in(Locale::default());
            cliclack::select(prompt)
                .item(true, yes_label, yes_hint)
                .item(false, no_label, no_hint)
                .interact()?
        };
        if !add_rust_target {
            return Ok(None);
        }
        let path: PathBuf = {
            cliclack::input(t::cmd_init::generate::Path.get_in(Locale::default()))
                .default_input("src/mulan.rs")
                .validate_on_enter(|input: &String| RelativePathBuf::from_path(input).map(|_| ()))
                .interact()?
        };
        let file = RelativePathBuf::from_path(path).expect("validated above");
        Ok(Some(Target::Rust(RustTarget { file })))
    }
}

/// Errors of [`create_locale_files`].
#[derive(Debug)]
pub enum CreateLocalesError {
    /// Failed to create a locales directory (maybe it already exists?).
    CreateDir(CreateLocalesDirError),

    /// Failed to create a locale file (maybe it already exists?).
    CreateFile(CreateLocaleFileError),

    /// Failed to write data to the newly created locale file.
    WriteFile(WriteLocaleFileError),
}

/// See [`CreateLocalesError::CreateDir`].
#[derive(Debug)]
pub struct CreateLocalesDirError {
    pub error: io::Error,

    /// Doesn't have a slash (`/`) at the end.
    pub path: RelativePathBuf,
}

/// See [`CreateLocalesError::CreateFile`].
#[derive(Debug)]
pub struct CreateLocaleFileError {
    pub error: io::Error,
    pub path: RelativePathBuf,
}

/// See [`CreateLocalesError::WriteFile`].
#[derive(Debug)]
pub struct WriteLocaleFileError {
    pub error: io::Error,
    pub path: RelativePathBuf,
}

/// Create a locales directory and all needed files inside of it
/// and logs the result to the user.
///
/// # Errors
///
/// - Such a directory already exists.
/// - Any of such files already exist.
/// - OS errors.
fn create_locale_files(locales: &[Language]) -> Result<(), CreateLocalesError> {
    #[derive(Serialize)]
    struct ExampleLocale {
        greeting: &'static str,
    }
    let dir_path = RelativePathBuf::from("locales");
    fs::create_dir(dir_path.as_str()).map_err(|error| {
        CreateLocalesError::CreateDir(CreateLocalesDirError {
            error,
            path: dir_path.clone(),
        })
    })?;
    for locale in locales {
        let contents = serde_saphyr::to_string(&match locale {
            Language::RuRu => ExampleLocale {
                greeting: "Привет, {name}!",
            },
            _ => ExampleLocale {
                greeting: "Hello, {name}!",
            },
        })
        .expect("should never fail");
        let path = dir_path.join(locale.tag().as_str()).with_extension("yaml");
        let mut file = match File::create_new(path.as_str()) {
            Ok(file) => file,
            Err(error) => {
                let error = CreateLocaleFileError { error, path };
                return Err(CreateLocalesError::CreateFile(error));
            }
        };
        if let Err(error) = file.write_all(contents.as_bytes()) {
            let error = WriteLocaleFileError { error, path };
            return Err(CreateLocalesError::WriteFile(error));
        }
    }
    Ok(())
}
