//! ...

use trait_set::trait_set;

trait_set! {
    /// ...
    pub trait ChumskyParser<'src, T> =
        chumsky::Parser<'src, &'src str, T, chumsky::extra::Err<chumsky::error::Rich<'src, char>>>;
}
