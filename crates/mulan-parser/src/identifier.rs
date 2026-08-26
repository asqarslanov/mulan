//! See [`Identifier`].

use std::convert::identity;

use compact_str::format_compact;
use mitsein::compact_string1::{CompactString1, CompactString1Ext as _};
use mitsein::small_vec1::SmallVec1;
use mulan_config::Case;

/// A name that can be converted to an
/// [identifier](https://en.wikipedia.org/wiki/Identifier_(computer_languages))
/// in any major programming language.
///
/// Has a relatively strict lexical form: e.g., ASCII-only, no whitespace,
/// every word starts with a Latin letter, etc.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Identifier {
    words: SmallVec1<[Word; 2]>,
}

impl Identifier {
    /// Formats this identifier as a message template parameter (e.g., `{foo}`).
    pub fn parameter_preview(&self, config: &mulan_config::Config) -> CompactString1 {
        let preview = format_compact!("{{{}}}", self.to_compact_string1(config.key_case));
        CompactString1::try_from(preview).expect("non-empty")
    }

    pub fn to_compact_string1(&self, case: Case) -> CompactString1 {
        use Case as C;
        match case {
            C::Kebab => self.to_kebab_case(),
            C::Pascal => self.to_pascal_case(),
            C::Snake => self.to_snake_case(),
        }
    }

    /// Converts this identifier to a `kebab-case` string
    /// (e.g., `lorem02-ipsum67`).
    fn to_kebab_case(&self) -> CompactString1 {
        self.words
            .iter1()
            .map(|word| &word.inner)
            .join_compact1("-")
    }

    /// Converts this identifier to a `PascalCase` string
    /// (e.g., `Lorem02Ipsum67`).
    fn to_pascal_case(&self) -> CompactString1 {
        self.words
            .iter1()
            .map(|word| {
                word.inner
                    .chars1()
                    .map_first_and_then(|first| first.to_ascii_uppercase(), identity)
                    .collect1::<CompactString1>()
            })
            .join_compact1("")
    }

    /// Converts this identifier to a `snake_case` string
    /// (e.g., `lorem02_ipsum67`).
    fn to_snake_case(&self) -> CompactString1 {
        self.words
            .iter1()
            .map(|word| &word.inner)
            .join_compact1("_")
    }
}

/// A part of an [`Identifier`].
///
/// For example, the identifier `student-bs23-id006` consists of three
/// [`Word`]s: `student`, `bs23`, and `id006`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Word {
    pub(super) inner: CompactString1,
}

/// Defines parsers with [`mod@chumsky`].
mod parser {
    use chumsky::prelude::*;
    use mitsein::iter1::IteratorExt as _;
    use smallvec::SmallVec;

    use super::{Identifier, Word};
    use crate::chumsky_parse::ChumskyParser;

    impl Identifier {
        #[must_use]
        pub fn chumsky_parser<'src>(
            word_parser: &impl ChumskyParser<'src, Word>,
        ) -> impl ChumskyParser<'src, Self> {
            word_parser
                .map(|part| part.inner)
                .separated_by(just('-'))
                .at_least(1)
                .collect::<SmallVec<[_; 2]>>()
                .map(|raw_words| Self {
                    words: raw_words
                        .into_iter()
                        .try_into_iter1()
                        .expect(".at_least(1)")
                        .map(|inner| Word { inner })
                        .collect1(),
                })
        }
    }

    impl Word {
        #[must_use]
        pub fn chumsky_parser<'src>() -> impl ChumskyParser<'src, Self> {
            let letter = one_of('a'..='z').labelled("small latin letter");
            let consecutive_letters = letter.clone().repeated().at_least(1);
            let consecutive_digits = text::digits(10);
            let letter_digit_mix = choice((consecutive_letters, consecutive_digits)).repeated();
            let word = letter.then(letter_digit_mix);
            word.to_slice().map(|it: &str| Self {
                inner: it.try_into().expect(".at_least(1)"),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use mitsein::iter1::IteratorExt as _;
    use rstest::rstest;

    use super::*;
    use crate::chumsky_parse::ChumskyParser as _;

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
    #[case("fn", Some(["fn"].as_slice()))]
    #[case("def", Some(["def"].as_slice()))]
    #[case("gen", Some(["gen"].as_slice()))]
    #[case("for", Some(["for"].as_slice()))]
    #[case("in", Some(["in"].as_slice()))]
    #[case("if", Some(["if"].as_slice()))]
    #[case("while", Some(["while"].as_slice()))]
    #[case("not", Some(["not"].as_slice()))]
    #[case("let", Some(["let"].as_slice()))]
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
    #[case("a.a", None)]
    #[case("a b", None)]
    #[case("ALLCAPS", None)]
    #[case("oneCapital", None)]
    #[case("apostrophe'", None)]
    #[case("diacriticñ", None)]
    #[case("cyrillicж", None)]
    #[case("dash-", None)]
    #[case("underscore_", None)]
    #[case("colon:", None)]
    #[case("semi;", None)]
    #[case("at@", None)]
    #[case("amp&", None)]
    #[case("star*", None)]
    #[case("hash#", None)]
    #[case("slash/", None)]
    #[case("dot.", None)]
    #[case("newline\n", None)]
    fn parse(#[case] input: &str, #[case] expected_output: Option<&[&str]>) {
        let word_parser = Word::chumsky_parser();
        let parser = Identifier::chumsky_parser(&word_parser);
        let actual_output = parser.mulan_parse(input).ok();
        let expected_output = expected_output.map(|raw_words| {
            let words = {
                raw_words
                    .iter()
                    .try_into_iter1()
                    .unwrap()
                    .map(|&it| Word {
                        inner: it.try_into().unwrap(),
                    })
                    .collect1()
            };
            Identifier { words }
        });
        assert_eq!(actual_output, expected_output);
    }

    #[rstest]
    #[case("e", "e")]
    #[case("foo", "foo")]
    #[case("lorem-ipsum", "lorem-ipsum")]
    #[case("aa-aa-aa", "aa-aa-aa")]
    #[case("i18n", "i18n")]
    #[case("r2-d2", "r2-d2")]
    #[case(
        "v3ry-l0n9-str1n9-of-l3tt3rs-and-d1g1ts",
        "v3ry-l0n9-str1n9-of-l3tt3rs-and-d1g1ts"
    )]
    fn to_kebab_case(#[case] input: &str, #[case] expected_output: &str) {
        let word_parser = Word::chumsky_parser();
        let parser = Identifier::chumsky_parser(&word_parser);
        let identifier = parser.mulan_parse(input).unwrap();
        let actual_output = identifier.to_kebab_case();
        assert_eq!(actual_output, expected_output);
    }

    #[rstest]
    #[case("e", "E")]
    #[case("foo", "Foo")]
    #[case("lorem-ipsum", "LoremIpsum")]
    #[case("aa-aa-aa", "AaAaAa")]
    #[case("i18n", "I18n")]
    #[case("r2-d2", "R2D2")]
    #[case(
        "v3ry-l0n9-str1n9-of-l3tt3rs-and-d1g1ts",
        "V3ryL0n9Str1n9OfL3tt3rsAndD1g1ts"
    )]
    fn to_pascal_case(#[case] input: &str, #[case] expected_output: &str) {
        let word_parser = Word::chumsky_parser();
        let parser = Identifier::chumsky_parser(&word_parser);
        let identifier = parser.mulan_parse(input).unwrap();
        let actual_output = identifier.to_pascal_case();
        assert_eq!(actual_output, expected_output);
    }

    #[rstest]
    #[case("e", "e")]
    #[case("foo", "foo")]
    #[case("lorem-ipsum", "lorem_ipsum")]
    #[case("aa-aa-aa", "aa_aa_aa")]
    #[case("i18n", "i18n")]
    #[case("r2-d2", "r2_d2")]
    #[case(
        "v3ry-l0n9-str1n9-of-l3tt3rs-and-d1g1ts",
        "v3ry_l0n9_str1n9_of_l3tt3rs_and_d1g1ts"
    )]
    fn to_snake_case(#[case] input: &str, #[case] expected_output: &str) {
        let word_parser = Word::chumsky_parser();
        let parser = Identifier::chumsky_parser(&word_parser);
        let identifier = parser.mulan_parse(input).unwrap();
        let actual_output = identifier.to_snake_case();
        assert_eq!(actual_output, expected_output);
    }
}
