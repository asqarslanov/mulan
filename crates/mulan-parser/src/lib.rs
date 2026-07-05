use trait_set::trait_set;

pub use self::schemas::Input; // TODO: remove this export
pub use self::schemas::output::{Key, Namespace, Node, Output, Translations};
pub use self::template::Template;

mod identifier;
mod schemas;
mod template;

trait_set! {
    /// ...
    trait ChumskyParser<T> = for<'src> chumsky::Parser<
            'src,
            &'src str,
            T,
            chumsky::extra::Err<chumsky::error::Rich<'src, char>>,
        >;
}
