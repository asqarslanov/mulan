//! See [`Identifier`].

use smallvec::SmallVec;

use self::word::Word;

mod word;

/// ...
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Identifier {
    words: SmallVec<[Word; 2]>,
}

mod parser {
    use chumsky::prelude::*;
    use compact_str::CompactString;

    use super::*;

    impl Identifier {
        /// Parses identifiers `like-this1`. Reject identifiers
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
                .repeated()
                .at_least(1)
                .to_slice()
                .separated_by(just(delimiter))
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
    #[case("e", &["e"])]
    #[case("foo", &["foo"])]
    #[case("lorem-ipsum", &["lorem", "ipsum"])]
    #[case("i18n", &["i18n"])]
    #[case("r2-d2", &["r2", "d2"])]
    #[case(
        "v3ry-l0n9-str1n9-of-l3tt3rs-and-d1g1ts",
        &["v3ry", "l0n9", "str1n9", "of", "l3tt3rs", "and", "d1g1ts"],
    )]
    fn parse_kebab_valid(#[case] input: &str, #[case] expected_output: &[&str]) {
        let parser = Identifier::chumsky_parser_kebab();
        let actual_output = parser.parse(input).unwrap();
        let expected_output = Identifier {
            words: {
                expected_output
                    .iter()
                    .map(|it| Word {
                        inner: CompactString::new(it),
                    })
                    .collect()
            },
        };
        assert_eq!(actual_output, expected_output)
    }
}
