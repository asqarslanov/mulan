//! # Mulan Parser
//!
//! This crate builds an intermediate representation of a user's project's
//! locales. This representation can later conveniently be used to generate
//! bindings to user-selected target platforms.
//!
//! This crate is responsible for locating and scanning user locale files,
//! parsing and validating them, and transforming this data to a more type-safe
//! form.
//!
//! See the [`Bundle::from_fs`] function.

#![cfg_attr(
    test,
    allow(
        clippy::unwrap_used,
        reason = "panicking tests don't produce silent bugs"
    )
)]

pub use self::identifier::Identifier;
use self::identifier::Word;
pub use self::schemas::bundle::{Bundle, DottedKey, Namespace, Node, Translations};
use self::schemas::locale_map::LocaleMap;
pub use self::schemas::locale_map::RawDottedKey;
pub use self::template::{Tag, Template, TemplatePart};
use crate::errors::BundleFromFsError;

mod chumsky_parse;
pub mod errors;
mod identifier;
mod schemas;
mod template;

impl Bundle {
    /// Does all the heavy-lifting (locating locales, parsing, transforming).
    /// The result of this function can be used to generate bindings.
    pub fn from_fs(config: &mulan_config::Config) -> Result<Self, BundleFromFsError> {
        let mut locale_map = LocaleMap::from_fs(config).map_err(BundleFromFsError::Read)?;
        let main_locale = {
            locale_map
                .locales
                .remove(&config.main_locale)
                .expect("all locales should've been read when building `locale_map`")
        };
        let word_parser = Word::chumsky_parser();
        let ident_parser = Identifier::chumsky_parser(&word_parser);
        let _key_parser = DottedKey::chumsky_parser(&ident_parser);
        let tag_parser = Tag::chumsky_parser(&ident_parser);
        let template_part_parser = TemplatePart::chumsky_parser(&tag_parser);
        let template_parser = Template::chumsky_parser(&template_part_parser);
        self::schemas::transform(
            config,
            &locale_map,
            &main_locale,
            &ident_parser,
            &template_parser,
        )
        .map_err(BundleFromFsError::Transform)
    }
}
