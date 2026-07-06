//! ...

use self::identifier::{Identifier, Word};
use self::schemas::TransformError;
use self::schemas::input::Input;
pub use self::schemas::output::{Key, Namespace, Node, Output, Subkey, Translations};
pub use self::template::{Parameter, Template, TemplatePart};

mod chumsky_parse;
mod identifier;
mod schemas;
mod template;

/// ...
pub fn read_and_parse(config: &mulan_config::Config) -> Result<Output, TransformError> {
    let mut input = Input::read().unwrap();
    let default_locale = {
        input
            .locales
            .remove(&config.default_locale)
            .ok_or(TransformError::LocaleNotFound(config.default_locale))?
    };
    let word_parser = Word::chumsky_parser();
    let ident_parser = Identifier::chumsky_parser(&word_parser);
    let subkey_parser = Subkey::chumsky_parser(&ident_parser);
    let param_parser = Parameter::chumsky_parser(&ident_parser);
    let template_part_parser = TemplatePart::chumsky_parser(&param_parser);
    let template_parser = Template::chumsky_parser(&template_part_parser);
    self::schemas::transform(
        &input,
        &default_locale,
        &subkey_parser,
        &template_parser,
        config,
    )
}
