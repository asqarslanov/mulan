//! See [`MessageTemplate`].

use compact_str::CompactString;
use smallvec::SmallVec;

use crate::identifier::Identifier;

/// ...
#[derive(Debug)]
pub struct Parameter {
    name: Identifier,
}

/// ...
#[derive(Debug)]
pub struct Template {
    parts: SmallVec<[TemplatePart; 1]>,
}

/// ...
#[derive(Debug)]
enum TemplatePart {
    /// ...
    Text(CompactString),

    /// ...
    Placeholder(Parameter),
}

mod parser {
    use chumsky::prelude::*;
    use compact_str::CompactString;

    use super::{Parameter, TemplatePart};
    use crate::identifier::Identifier;

    impl TemplatePart {
        #[must_use]
        pub fn chumsky_parser<'src>()
        -> impl Parser<'src, &'src str, Self, extra::Err<Rich<'src, char>>> {
            let text = {
                choice((just("{{").to('{'), just("}}").to('}'), none_of("{}")))
                    .repeated()
                    .at_least(1)
                    .to_slice()
                    .map(|it| Self::Text(CompactString::new(it)))
            };
            let expr = {
                Identifier::chumsky_parser()
                    .padded()
                    .delimited_by(just('{'), just('}'))
                    .map(|name| Self::Placeholder(Parameter { name }))
            };
            choice((text, expr))
        }
    }
}
