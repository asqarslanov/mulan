use std::io;
use std::path::PathBuf;
use std::process::ExitCode;

use itertools::Itertools as _;
use mulan_config::errors::{LocateError, NotFoundError};
use mulan_config::{Language, RustTarget, Target};
use relative_path::RelativePathBuf;

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
    let user_choice = match UserChoice::interactive_prompt() {
        Ok(data) => data,
        Err(err) if matches!(err.kind(), io::ErrorKind::Interrupted) => {
            // Ctrl+C interrupt.
            return Ok(ExitCode::from(SIGINT));
        }
        Err(err) => {
            // A `cliclack` error indicates that we couldn't print to the console.
            // There's no meaningful recovery strategy, so we just `panic!`.
            // There's no point in creating rich Miette reports---
            // we won't probably be able to print them anyway.
            panic!("{err}");
        }
    };
    todo!("create mulan.toml and locales/");
}

///
#[derive(Debug)]
pub struct ConfigExistsError {
    ///
    pub path: RelativePathBuf,
}

///
#[derive(Debug)]
struct UserChoice {
    ///
    locales: Vec<Language>,

    ///
    main_locale: Language,

    ///
    generate: Vec<Target>,
}

impl UserChoice {
    ///
    fn interactive_prompt() -> io::Result<Self> {
        cliclack::intro("Mulan")?;
        let locales = Self::prompt_locales(&[Language::EnUs, Language::RuRu])?;
        let main_locale = Self::prompt_main_locale(&locales)?;
        let generate = Self::prompt_generate()?;
        cliclack::outro("You're all set")?;
        Ok(UserChoice {
            locales,
            main_locale,
            generate,
        })
    }

    ///
    fn prompt_locales(all_possible_locales: &[Language]) -> io::Result<Vec<Language>> {
        cliclack::multiselect("locales")
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
        cliclack::select("main locale").items(&items).interact()
    }

    ///
    fn prompt_generate() -> io::Result<Vec<Target>> {
        let mut generate = Vec::new();
        loop {
            let add = cliclack::select("add a generation target")
                .item(false, "no", "don't")
                .item(true, "yes", "add")
                .interact()?;
            if !add {
                break;
            }
            let path: PathBuf = cliclack::input("path")
                .default_input("src/mulan.rs")
                .interact()?;
            generate.push(Target::Rust(RustTarget {
                file: RelativePathBuf::from_path(path).unwrap(),
            }));
        }
        todo!();
    }
}
