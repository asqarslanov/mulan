pub use self::schemas::Input; // TODO: remove this export
pub use self::schemas::output::{Key, Namespace, Node, Output, Subkey, Translations};
pub use self::template::{Parameter, Template, TemplatePart};

mod chumsky_parse;
mod identifier;
mod schemas;
mod template;
