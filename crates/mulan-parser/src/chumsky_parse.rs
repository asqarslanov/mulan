//! Simplifies the interaction with [`mod@chumsky`].
//!
//! See [`ChumskyParser`].

use chumsky::prelude::*;
use mitsein::iter1::IteratorExt as _;

use crate::errors::{ChumskyAllErrors, ChumskySingleError};

/// A trait alias to a Chumsky parser with rich error reporting.
///
/// Provides [`ChumskyParser::mulan_parse`] that conviniently wraps parse
/// results.
#[expect(clippy::disallowed_types, reason = "to make `Parser::parse` available")]
pub trait ChumskyParser<'src, Out>:
    Parser<'src, &'src str, Out, extra::Err<Rich<'src, char>>>
{
    /// A replacement for `.parse(_)` that converts its [`chumsky::ParseResult`]
    /// to a [`std::result::Result`] with a custom [`ChumskyAllErrors`] error
    /// type.
    fn mulan_parse(&self, source: &'src str) -> Result<Out, ChumskyAllErrors> {
        #[expect(
            clippy::disallowed_methods,
            reason = "this is the reason `parse` is disallowed in favor of `mulan_parse`"
        )]
        self.parse(source)
            .into_result()
            .map_err(|errors| ChumskyAllErrors {
                source: source.into(),
                errors: errors
                    .into_iter()
                    .try_into_iter1()
                    .expect("if parsing failed, there should at least be one error")
                    .map(|err| ChumskySingleError {
                        message: err.to_string(),
                        span: err.span().into_range().into(),
                    })
                    .collect1(),
            })
    }
}

#[expect(
    clippy::disallowed_types,
    reason = "this is the reason `Parser` is disallowed in favor of `ChumskyParser`"
)]
impl<'src, Out, T: Parser<'src, &'src str, Out, extra::Err<Rich<'src, char>>>>
    ChumskyParser<'src, Out> for T
{
}
