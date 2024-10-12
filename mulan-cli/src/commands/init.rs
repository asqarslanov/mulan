use std::{fmt::Display, path::PathBuf};

use anyhow::Result;
use strum::{EnumIs, EnumIter, EnumMessage, IntoEnumIterator, IntoStaticStr};

fn header(text: impl Display) -> impl Display {
    console::style(text).bold()
}

fn key(name: &str) -> impl Display {
    console::style(format!(" {name} "))
        .on_black()
        .white()
        .bright()
        .bold()
}

fn key_description(name: &str, action: &str) -> impl Display {
    format!("{} to {}", key(name), action)
}

fn multiselect_header(text: impl Display) -> impl Display {
    let separator = console::style("/").yellow();

    format!(
        "{} {separator} {} {separator} {}",
        header(text),
        key_description("space", "select"),
        key_description(if cfg!(windows) { "enter" } else { "return" }, "continue"),
    )
}

pub fn init() -> Result<bool> {
    cliclack::intro(
        console::style(" Mulan Project Wizard ")
            .on_red()
            .black()
            .bold(),
    )?;

    // HACK: make `cliclack` catch Ctrl+C signals.
    ctrlc::set_handler(|| ())?;

    let _locales_dir: PathBuf = cliclack::input(header("Locales directory"))
        .default_input("locales/")
        .interact()?;

    let _locale_extension = cliclack::select(header("Locale extension"))
        .items(
            &LocaleExtension::iter()
                .map(|it| (it, <&_>::from(it), it.get_message().unwrap()))
                .collect::<Box<_>>(),
        )
        .initial_value(LocaleExtension::Json5)
        .interact()?;

    let _locale_languages = cliclack::multiselect(multiselect_header("Locale languages"))
        .filter_mode()
        .items(
            &LocaleLanguage::iter()
                .map(|it| (it, <&_>::from(it), ""))
                .collect::<Box<_>>(),
        )
        .initial_values(vec![LocaleLanguage::EnUs])
        .required(false)
        .interact()?;

    let _target_platforms = cliclack::multiselect(multiselect_header("Target platforms"))
        .items(
            &TargetPlatform::iter()
                .map(|it| (it, <&_>::from(it), ""))
                .collect::<Box<_>>(),
        )
        .required(false)
        .interact()?;

    let confirmation = cliclack::confirm(header("Everything good?"))
        .initial_value(true)
        .interact()?;

    if confirmation {
        cliclack::outro_note(
            console::style("You're all set!").green().bold(),
            console::style("Check `.mulan/mulan.toml`"),
        )?;
    } else {
        cliclack::outro_cancel("Operation cancelled.")?;
    }

    Ok(confirmation)
}

#[derive(Clone, Copy, PartialEq, Eq, EnumIter, EnumMessage, IntoStaticStr)]
#[strum(serialize_all = "UPPERCASE")]
enum LocaleExtension {
    #[strum(message = "good ol' JSON, not very ergonomic but still a solid choice")]
    Json,
    #[strum(message = "recommended")]
    Json5,
    #[strum(
        to_string = "Jsonnet",
        message = "very powerful, but if you're not sure, choose JSON5"
    )]
    Jsonnet,
    #[strum(message = "not recommended, no support for multiline tables")]
    Toml,
    #[strum(message = "well, you do you...")]
    Yaml,
}

#[derive(Clone, Copy, PartialEq, Eq, EnumIs, EnumIter, IntoStaticStr)]
enum LocaleLanguage {
    #[strum(to_string = "en-US (English, United States)")]
    EnUs,
    #[strum(to_string = "ru-RU (Russian, Russia)")]
    RuRu,
}

#[derive(Clone, Copy, PartialEq, Eq, EnumIter, IntoStaticStr)]
enum TargetPlatform {
    #[strum(to_string = "C/C++")]
    CCpp,
    Gleam,
    Go,
    Java,
    Kotlin,
    Rust,
    #[strum(to_string = "JavaScript/TypeScript")]
    JavascriptTypescript,
    Python,
    React,
    #[strum(to_string = "Solid.js")]
    SolidJs,
    Svelte,
    Swift,
    Vue,
}
