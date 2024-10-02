use std::path::PathBuf;

use anyhow::Result;
use inquire::{Confirm, MultiSelect, Select, Text};
use strum::{Display, EnumIs, EnumIter, IntoEnumIterator};

#[derive(Clone, Display, EnumIs, EnumIter)]
#[strum(serialize_all = "UPPERCASE")]
enum LocaleExtension {
    Json,
    Json5,
    #[strum(serialize = "Jsonnet")]
    Jsonnet,
    Toml,
    Yaml,
}

#[derive(Clone, Display, EnumIs, EnumIter)]
enum LocaleLanguage {
    #[strum(serialize = "en-US (English, United States)")]
    EnUs,
    #[strum(serialize = "ru-RU (Russian, Russia)")]
    RuRu,
}

#[derive(Clone, Display, EnumIter)]
enum TargetPlatform {
    #[strum(serialize = "C/C++")]
    CCpp,
    Gleam,
    Go,
    Java,
    Kotlin,
    Rust,
    #[strum(serialize = "JavaScript/TypeScript")]
    JavascriptTypescript,
    Python,
    React,
    #[strum(serialize = "Solid.js")]
    SolidJs,
    Svelte,
    Swift,
    Vue,
}

pub fn init() -> Result<()> {
    let _locales_dir: PathBuf = Text::new("Locales directory:")
        .with_placeholder("<DIRECTORY>")
        .with_initial_value("locales/")
        .prompt()?
        .into();

    // TODO: report a bug to the `inquire` craete.
    let _locale_extension = Select::new("Locale extension:", LocaleExtension::iter().collect())
        // .with_vim_mode(true)
        // .without_filtering()
        .with_starting_cursor(LocaleExtension::iter().position(|it| it.is_toml()).unwrap())
        .prompt()?;

    let _locale_languages = MultiSelect::new("Locale languages:", LocaleLanguage::iter().collect())
        .with_default(&[LocaleLanguage::iter().position(|it| it.is_en_us()).unwrap()])
        .prompt()?;

    let _target_platforms =
        MultiSelect::new("Target platforms:", TargetPlatform::iter().collect()).prompt()?;

    let _confirmation = Confirm::new("Everything good?")
        .with_default(true)
        .prompt()?;

    Ok(())
}
