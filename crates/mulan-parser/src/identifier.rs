//! See [`Identifier`].

use smallvec::SmallVec;

use self::word::Word;

mod word;

/// ...
///
/// ## Valid Examples
///
/// `e`, `foo`, `lorem-ipsum`, `i18n`, `r2-d2`,
/// `v3ry-l0n9-str1n9-of-l3tt3rs-and-d1g1ts`.
///
/// ## Invalid Examples
///
/// - `` (empty string)
/// - `my-5-cents` ()
/// - ...
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Identifier {
    words: SmallVec<[Word; 2]>,
}

mod parser {
    use chumsky::prelude::*;
    use compact_str::CompactString;

    use super::*;

    impl Identifier {
        /// ...
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
