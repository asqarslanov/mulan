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
//! See the [`compose`] function.

#![cfg_attr(
    test,
    allow(
        clippy::unwrap_used,
        reason = "panicking tests don't produce silent bugs"
    )
)]

pub use self::identifier::Identifier;
use self::identifier::Word;
use self::schemas::input::Input;
pub use self::schemas::input::RawDottedKey;
pub use self::schemas::output::{DottedKey, Namespace, Node, Output, Translations};
pub use self::template::{Tag, Template, TemplatePart};
use crate::errors::ComposeError;

mod chumsky_parse;
pub mod errors;
mod identifier;
mod schemas;
mod template;

/// Does all the heavy-lifting (locating locales, parsing, transforming).
/// The result of this function can be used to generate bindings.
pub fn compose(config: &mulan_config::Config) -> Result<Output, ComposeError> {
    let mut input = Input::read(config).map_err(ComposeError::Read)?;
    let main_locale = {
        input
            .locales
            .remove(&config.main_locale)
            .expect("all locales should've been read when parsing `input`")
    };
    let word_parser = Word::chumsky_parser();
    let ident_parser = Identifier::chumsky_parser(&word_parser);
    let _key_parser = DottedKey::chumsky_parser(&ident_parser);
    let tag_parser = Tag::chumsky_parser(&ident_parser);
    let template_part_parser = TemplatePart::chumsky_parser(&tag_parser);
    let template_parser = Template::chumsky_parser(&template_part_parser);
    self::schemas::transform(
        &input,
        &main_locale,
        &ident_parser,
        &template_parser,
        config,
    )
    .map_err(ComposeError::Transform)
}
