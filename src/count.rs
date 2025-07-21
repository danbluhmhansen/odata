use std::fmt::Display;

use chumsky::{Parser, input::ValueInput, prelude::*};
use derive_more::{Deref, From};

use crate::token::{Keyword, Token, TokenParser};

/// The [OData $count query option](https://docs.oasis-open.org/odata/odata/v4.01/odata-v4.01-part1-protocol.html#sec_SystemQueryOptioncount).
///
/// # Examples
///
/// ```
/// use odata::OData;
/// assert_eq!(
///     Ok(OData {
///         count: true.into(),
///         ..Default::default()
///     }),
///     "$count".try_into(),
/// );
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Default, Deref, From)]
pub struct Count(bool);

impl<'tokens, 'src: 'tokens> TokenParser<'tokens, 'src> for Count {
    fn parser<I: ValueInput<'tokens, Token = Token<'src>, Span = SimpleSpan>>()
    -> impl Parser<'tokens, I, Self, extra::Err<Rich<'tokens, Token<'src>>>> + Clone {
        just(Token::Kw(Keyword::Count)).ignore_then(
            select! { Token::Bool(bool) => bool }
                .or_not()
                .map(|b| b.unwrap_or(true))
                .map(Self)
                .labelled("true, false"),
        )
    }
}

impl<'src> TryFrom<&'src str> for Count {
    type Error = String;
    fn try_from(value: &'src str) -> Result<Self, Self::Error> {
        Self::try_from_str(value)
    }
}

impl Display for Count {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "$count={}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn valid() {
        assert_eq!(
            (
                Ok(Count(true)),
                Ok(Count(true)),
                Ok(Count(true)),
                Ok(Count(false)),
            ),
            (
                "$count".try_into(),
                "$count=".try_into(),
                "$count=true".try_into(),
                "$count=false".try_into(),
            )
        );
    }

    #[test]
    fn invalid() {
        insta::assert_snapshot!(
            Count::try_from("").unwrap_err() + "\n" + &Count::try_from("$count=no").unwrap_err()
        );
    }
}
