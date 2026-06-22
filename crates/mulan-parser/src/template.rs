//! See [`MessageTemplate`].

use compact_str::CompactString;
use smallvec::SmallVec;

use crate::identifier::Identifier;

/// ...
#[derive(Debug, PartialEq, Eq)]
pub struct Parameter {
    name: Identifier,
}

/// ...
#[derive(Debug)]
pub struct Template {
    parts: SmallVec<[TemplatePart; 1]>,
}

/// ...
#[derive(Debug, PartialEq, Eq)]
enum TemplatePart {
    /// ...
    Text(CompactString),

    /// ...
    Placeholder(Parameter),
}

mod parser {
    use chumsky::prelude::*;
    use compact_str::CompactString;
    use smallvec::SmallVec;

    use super::{Parameter, Template, TemplatePart};
    use crate::identifier::Identifier;

    impl TemplatePart {
        #[must_use]
        pub fn chumsky_parser<'src>()
        -> impl Parser<'src, &'src str, Self, extra::Err<Rich<'src, char>>> {
            let text = {
                choice((just("{{").to('{'), just("}}").to('}'), none_of("{}")))
                    .repeated()
                    .at_least(1)
                    .collect::<String>()
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

    impl Template {
        #[must_use]
        pub fn chumsky_parser<'src>()
        -> impl Parser<'src, &'src str, Self, extra::Err<Rich<'src, char>>> {
            TemplatePart::chumsky_parser()
                .repeated()
                .collect()
                .then_ignore(end())
                .map(|parts: Vec<_>| Self {
                    parts: SmallVec::from_vec(parts),
                })
        }
    }
}

#[cfg(test)]
mod tests {
    use chumsky::Parser as _;

    use super::*;

    #[test]
    fn parse() {
        let ident_parser = Identifier::chumsky_parser();
        let msg_parser = Template::chumsky_parser();
        let actual_output = {
            msg_parser
                .parse("aaa{bbb}ccc{{ddd}}eee{{{  fff  }}}ggg{{{{hhh}}}}iii")
                .unwrap()
                .parts
        };
        let expected_output = [
            TemplatePart::Text("aaa".into()),
            TemplatePart::Placeholder(Parameter {
                name: ident_parser.parse("bbb").unwrap(),
            }),
            TemplatePart::Text("ccc{ddd}eee{".into()),
            TemplatePart::Placeholder(Parameter {
                name: ident_parser.parse("fff").unwrap(),
            }),
            TemplatePart::Text("}ggg{{hhh}}iii".into()),
        ];
        assert_eq!(actual_output, expected_output.into());
    }
}
