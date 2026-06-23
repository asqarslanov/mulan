//! See [`Identifier`].

use compact_str::CompactString;
use smallvec::SmallVec;

/// A generic name that can be converted to an
/// [identifier](https://en.wikipedia.org/wiki/Identifier_(computer_languages))
/// in any major programming language.
///
/// Has a relatively strict lexical form: e.g., ASCII-only, no whitespace,
/// every word starts with a Latin letter, etc.
///
/// This type serves as the underlying representation of
/// [`Parameter`](crate::template::Parameter) or a message path segment(key).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Identifier {
    words: SmallVec<[Word; 2]>,
}

/// A part of an [`Identifier`](crate::identifier::Identifier).
///
/// For example, the identifier `student-bs23-id006` consists of three
/// [`Word`]s: `student`, `bs23`, and `id006`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Word {
    pub(super) inner: CompactString,
}

mod parser {
    use chumsky::prelude::*;
    use compact_str::CompactString;

    use super::{Identifier, Word};

    impl Identifier {
        #[must_use]
        pub fn chumsky_parser<'src>()
        -> impl Parser<'src, &'src str, Self, extra::Err<Rich<'src, char>>> {
            Word::chumsky_parser()
                .map(|part| part.inner)
                .separated_by(just('-'))
                .at_least(1)
                .collect()
                .map(|raw_words: Vec<_>| Self {
                    words: raw_words.into_iter().map(|inner| Word { inner }).collect(),
                })
        }
    }

    impl Word {
        #[must_use]
        pub fn chumsky_parser<'src>()
        -> impl Parser<'src, &'src str, Self, extra::Err<Rich<'src, char>>> {
            let letter = one_of('a'..='z').labelled("small latin letter");
            let consecutive_letters = letter.clone().repeated().at_least(1);
            let consecutive_digits = text::digits(10);
            let letter_digit_mix = choice((consecutive_letters, consecutive_digits)).repeated();
            let word = letter.then(letter_digit_mix);
            word.to_slice().map(|it: &str| Self {
                inner: CompactString::new(it),
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
    fn parse(#[case] input: &str, #[case] expected_output: Option<&[&str]>) {
        let parser = Identifier::chumsky_parser();
        let actual_output = parser.parse(input).into_output();
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

    #[rstest]
    #[case("w", Ok)]
    #[case("manylettersofthelatinalphabet", Ok)]
    #[case("manylettersofthelatinalphabet2", Ok)]
    #[case("manydigits281348214378912347892", Ok)]
    #[case("onedigit1", Ok)]
    #[case("digitin1themiddle", Ok)]
    #[case("fn", Ok)]
    #[case("def", Ok)]
    #[case("gen", Ok)]
    #[case("for", Ok)]
    #[case("in", Ok)]
    #[case("if", Ok)]
    #[case("while", Ok)]
    #[case("not", Ok)]
    #[case("let", Ok)]
    #[case("", Err)]
    #[case(" ", Err)]
    #[case("spaceattheend ", Err)]
    #[case(" spaceatthebeginning", Err)]
    #[case("1", Err)]
    #[case("123", Err)]
    #[case("1digitatthebeginning", Err)]
    #[case("ALLCAPS", Err)]
    #[case("oneCapital", Err)]
    #[case("W", Err)]
    #[case("apostrophe'", Err)]
    #[case("diacriticñ", Err)]
    #[case("cyrillicж", Err)]
    #[case("dash-", Err)]
    #[case("underscore_", Err)]
    #[case("colon:", Err)]
    #[case("semi;", Err)]
    #[case("at@", Err)]
    #[case("amp&", Err)]
    #[case("star*", Err)]
    #[case("hash#", Err)]
    #[case("slash/", Err)]
    #[case("dot.", Err)]
    #[case("newline\n", Err)]
    fn parse_word(#[case] input: &str, #[case] expected_output: fn(()) -> Result<(), ()>) {
        let parser = Word::chumsky_parser();
        let parse_result = parser.parse(input).into_result();
        assert_eq!(parse_result.is_ok(), expected_output(()).is_ok());
        if let Ok(parsed_word) = parse_result {
            let input_word = Word {
                inner: CompactString::new(input),
            };
            assert_eq!(input_word, parsed_word);
        }
    }
}
