use std::path::PathBuf;

use anyhow::Result;
use strum::{EnumIs, EnumIter, EnumMessage, IntoEnumIterator, IntoStaticStr};

pub fn init() -> Result<bool> {
    cliclack::intro("mulan init")?;

    let _locales_dir: PathBuf = cliclack::input("Locales directory")
        .default_input("locales/")
        .interact()?;

    let _locale_extension = cliclack::select("Locale extension")
        .items(
            &LocaleExtension::iter()
                .map(|it| (it, <&_>::from(it), it.get_message().unwrap()))
                .collect::<Box<_>>(),
        )
        .initial_value(LocaleExtension::Json5)
        .interact()?;

    let _locale_languages = cliclack::multiselect("Locale languages / press 'Space' to select")
        .filter_mode()
        .items(
            &LocaleLanguage::iter()
                .map(|it| (it, <&_>::from(it), ""))
                .collect::<Box<_>>(),
        )
        .initial_values(vec![LocaleLanguage::EnUs])
        .required(false)
        .interact()?;

    let _target_platforms = cliclack::multiselect("Target platforms / press 'Space' to select")
        .items(
            &TargetPlatform::iter()
                .map(|it| (it, <&_>::from(it), ""))
                .collect::<Box<_>>(),
        )
        .required(false)
        .interact()?;

    let confirmation = cliclack::confirm("Everything good?")
        .initial_value(true)
        .interact()?;

    if confirmation {
        cliclack::outro_note("You're all set!", "Check `.mulan/mulan.toml`")?;
    } else {
        cliclack::outro_cancel("See you later!")?;
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
