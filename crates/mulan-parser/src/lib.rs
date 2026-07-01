pub use self::schemas::Input; // TODO: remove this export
pub use self::schemas::output::{Key, Namespace, Node, Output, Translations};
pub use self::template::Template;

mod identifier;
mod schemas;
mod template;
