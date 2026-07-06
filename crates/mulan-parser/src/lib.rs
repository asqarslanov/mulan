//! ...

use self::schemas::input::Input;
pub use self::schemas::output::{Key, Namespace, Node, Output, Subkey, Translations};
pub use self::template::{Parameter, Template, TemplatePart};

mod chumsky_parse;
mod identifier;
mod schemas;
mod template;

/// ...
pub fn read_and_parse(
    config: &mulan_config::Config,
) -> Result<Output, self::schemas::TransformError> {
    let input = Input::read().unwrap();
    let word_parser = crate::identifier::Word::chumsky_parser();
    let ident_parser = crate::identifier::Identifier::chumsky_parser(&word_parser);
    let subkey_parser = Subkey::chumsky_parser(&ident_parser);
    let param_parser = Parameter::chumsky_parser(&ident_parser);
    let template_part_parser = TemplatePart::chumsky_parser(&param_parser);
    let template_parser = Template::chumsky_parser(&template_part_parser);
    self::schemas::transform(&input, &subkey_parser, &template_parser, config)
}
