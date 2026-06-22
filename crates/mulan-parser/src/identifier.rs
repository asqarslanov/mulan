//! See [`Identifier`].

use smallvec::SmallVec;

use self::word::Word;

mod word;

/// A name that can be converted to an
/// [identifier](https://en.wikipedia.org/wiki/Identifier_(computer_languages)) in
/// any major programming language. Can be used as a message path segment (key)
/// or as a parameter placeholder (variable name).
///
/// Has a relatively strict lexical form: e.g., ASCII-only, no whitespace, every
/// word starts with a Latin letter, etc.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Identifier {
    words: SmallVec<[Word; 2]>,
}

mod parser {
    use chumsky::prelude::*;
    use compact_str::CompactString;

    use super::{Identifier, Word};

    impl Identifier {
        /// Parses identifiers `like-this1`. Rejects identifiers
        /// `with_underscores` or `CapitalLetters`.
        #[must_use]
        pub fn chumsky_parser_kebab<'src>()
        -> impl Parser<'src, &'src str, Self, extra::Err<Rich<'src, char>>> {
            Self::chumsky_parser('-')
        }

        fn chumsky_parser<'src>(
            delimiter: char,
        ) -> impl Parser<'src, &'src str, Self, extra::Err<Rich<'src, char>>> {
            Word::chumsky_parser()
                .map(|part| part.inner)
                .to_slice()
                .separated_by(just(delimiter))
                .at_least(1)
                .collect()
                .map(|them: Vec<_>| {
                    let words = {
                        them.into_iter()
                            .map(|it: &str| Word {
                                inner: CompactString::new(it),
                            })
                            .collect()
                    };
                    Self { words }
                })
        }
    }
}

#[cfg(test)]
mod tests {
    use chumsky::Parser as _;
    use compact_str::CompactString;
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("e", Some(["e"].as_slice()))]
    #[case("foo", Some(["foo"].as_slice()))]
    #[case("lorem-ipsum", Some(["lorem", "ipsum"].as_slice()))]
    #[case("aa-aa-aa", Some(["aa", "aa", "aa"].as_slice()))]
    #[case("i18n", Some(["i18n"].as_slice()))]
    #[case("r2-d2", Some(["r2", "d2"].as_slice()))]
    #[case(
        "v3ry-l0n9-str1n9-of-l3tt3rs-and-d1g1ts",
        Some(["v3ry", "l0n9", "str1n9", "of", "l3tt3rs", "and", "d1g1ts"].as_slice()),
    )]
    #[case("", None)]
    #[case(" ", None)]
    #[case("7", None)]
    #[case("7up", None)]
    #[case("r-2-d-2", None)]
    #[case("a_b", None)]
    #[case(" a-b", None)]
    #[case("a-b ", None)]
    #[case("aa-", None)]
    #[case("-aa", None)]
    #[case("a-A", None)]
    #[case("A", None)]
    #[case("AAA", None)]
    #[case("aAa", None)]
    #[case("a.a", None)]
    #[case("a b", None)]
    fn parse_kebab(#[case] input: &str, #[case] expected_output: Option<&[&str]>) {
        let parser = Identifier::chumsky_parser_kebab();
        let actual_output = parser.parse(input).into_result().ok();
        let expected_output = expected_output.map(|raw_words| {
            let words = {
                raw_words
                    .iter()
                    .map(|it| Word {
                        inner: CompactString::new(it),
                    })
                    .collect()
            };
            Identifier { words }
        });
        assert_eq!(actual_output, expected_output);
    }
}
