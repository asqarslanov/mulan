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

use self::identifier::{Identifier, Word};
use self::schemas::input::Input;
pub use self::schemas::input::RawKey;
pub use self::schemas::output::{Key, Namespace, Node, Output, Subkey, Translations};
pub use self::template::{Parameter, Template, TemplatePart};
use crate::errors::{ComposeError, LocaleNotFoundError, TransformError};

mod chumsky_parse;
pub mod errors;
mod identifier;
mod schemas;
mod template;

/// Does all the heavy-lifting (locating locales, parsing, transforming).
/// The result of this function can be used to generate bindings.
pub fn compose(config: &mulan_config::Config) -> Result<Output, ComposeError> {
    let mut input = Input::read(config).map_err(ComposeError::Read)?;
    let main_locale = input.locales.remove(&config.main_locale).ok_or({
        ComposeError::Transform(TransformError::LocaleNotFound(LocaleNotFoundError {
            locale: config.main_locale,
        }))
    })?;
    let word_parser = Word::chumsky_parser();
    let ident_parser = Identifier::chumsky_parser(&word_parser);
    let subkey_parser = Subkey::chumsky_parser(&ident_parser);
    let param_parser = Parameter::chumsky_parser(&ident_parser);
    let template_part_parser = TemplatePart::chumsky_parser(&param_parser);
    let template_parser = Template::chumsky_parser(&template_part_parser);
    self::schemas::transform(
        &input,
        &main_locale,
        &subkey_parser,
        &template_parser,
        config,
    )
    .map_err(ComposeError::Transform)
}
