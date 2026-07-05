//! ...

use chumsky::prelude::*;
use trait_set::trait_set;

trait_set! {
    /// ...
    pub trait ChumskyParser<'src, T> = Parser<'src, &'src str, T, extra::Err<Rich<'src, char>>>;
}
