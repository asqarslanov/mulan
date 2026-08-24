use std::io::Write;
use std::path::PathBuf;
use std::process::ExitCode;
use std::{fs, io};

use either::Either;
use itertools::Itertools as _;
use mitsein::small_vec1::SmallVec1;
use mulan_config::errors::{LocateError, NotFoundError};
use mulan_config::{Language, RustTarget, Target};
use relative_path::RelativePathBuf;
use serde::Serialize;

use crate::error_reporting::ToReport as _;

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
        Err(Either::Right(miette_report)) => Err(miette_report),
        Err(Either::Left(io_err)) if matches!(io_err.kind(), io::ErrorKind::Interrupted) => {
            // Ctrl+C interrupt.
            Ok(ExitCode::from(SIGINT))
        }
        Err(Either::Left(io_err)) => {
            // A `cliclack` error indicates that we couldn't print to the console.
            // There's no meaningful recovery strategy, so we just `panic!`.
            // There's no point in creating rich Miette reports---
            // we won't probably be able to print them anyway.
            panic!("{io_err}");
        }
    }
}

///
#[derive(Debug)]
pub struct ConfigExistsError {
    ///
    pub path: RelativePathBuf,
}

///
#[derive(Debug)]
pub struct CreateConfigError {
    pub error: io::Error,

    ///
    pub path: RelativePathBuf,
}

///
fn prompt_and_init() -> Result<(), Either<io::Error, miette::Report>> {
    todo!("use mulan");
    cliclack::intro("Mulan").map_err(Either::Left)?;
    let user_choice = UserChoice::interactive_prompt().map_err(Either::Left)?;
    user_choice
        .write_to_file()
        .map_err(|err| err.to_report(&mulan_config::Config::dummy()))
        .map_err(Either::Right)?;
    create_locale_files(&user_choice.locales);
    cliclack::outro("You're all set").map_err(Either::Left)?;
    Ok(())
}

///
#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
struct UserChoice {
    ///
    locales: Vec<Language>,

    ///
    main_locale: Language,

    ///
    generate: Option<SmallVec1<[Target; 1]>>,
}

impl UserChoice {
    ///
    fn interactive_prompt() -> io::Result<Self> {
        let locales = Self::prompt_locales(&[Language::EnUs, Language::RuRu])?;
        let main_locale = Self::prompt_main_locale(&locales)?;
        let generate = Self::prompt_generate()?.map(SmallVec1::from_one);
        Ok(UserChoice {
            locales,
            main_locale,
            generate,
        })
    }

    ///
    fn write_to_file(&self) -> Result<(), CreateConfigError> {
        let path = RelativePathBuf::from("mulan.toml");
        let contents = toml::to_string_pretty(self).expect("should never fail");
        fs::write(path.as_str(), contents).map_err(|error| CreateConfigError { error, path })?;
        Ok(())
    }

    ///
    fn prompt_locales(all_possible_locales: &[Language]) -> io::Result<Vec<Language>> {
        cliclack::multiselect("Select locales you want to support")
            .items(
                &all_possible_locales
                    .iter()
                    .map(|&lang| (lang, lang.tag(), lang.name()))
                    .collect_vec(),
            )
            .interact()
    }

    ///
    fn prompt_main_locale(locales: &[Language]) -> io::Result<Language> {
        let items = {
            locales
                .iter()
                .map(|&lang| (lang, lang.tag(), lang.name()))
                .collect_vec()
        };
        cliclack::select("Choose the main locale")
            .items(&items)
            .interact()
    }

    ///
    fn prompt_generate() -> io::Result<Option<Target>> {
        let yes_hint = "you will need to specify a path to where bindings should be generated";
        let no_hint = "you can always add it later";
        let add = cliclack::select("Add a Rust codegen target?")
            .item(true, "Yes", yes_hint)
            .item(false, "No", no_hint)
            .interact()?;
        if !add {
            return Ok(None);
        }
        let path: PathBuf = {
            cliclack::input("path")
                .default_input("src/mulan.rs")
                .validate_on_enter(|input: &String| RelativePathBuf::from_path(input).map(|_| ()))
                .interact()?
        };
        let file = RelativePathBuf::from_path(path).expect("validated above");
        Ok(Some(Target::Rust(RustTarget { file })))
    }
}

///
fn create_locale_files(locales: &[Language]) -> io::Result<()> {
    #[derive(Serialize)]
    struct ExampleLocale {
        greeting: &'static str,
    }

    let dir_path = RelativePathBuf::from("locales");
    fs::create_dir(dir_path.as_str());
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
        let mut file = fs::File::create_new(path.as_str()).unwrap();
        file.write_all(contents.as_bytes());
    }
    Ok(())
}
