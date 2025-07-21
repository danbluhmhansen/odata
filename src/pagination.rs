use std::fmt::Display;

use chumsky::{Parser, input::ValueInput, prelude::*};

use crate::token::{Keyword, Token, TokenParser};

/// The [OData $skip query option](https://docs.oasis-open.org/odata/odata/v4.01/odata-v4.01-part1-protocol.html#sec_SystemQueryOptionskip).
///
/// # Examples
///
/// ```
/// use odata::{OData, pagination::Skip};
/// assert_eq!(
///     Ok(OData {
///         skip: Some(Skip::Offset(5)),
///         ..Default::default()
///     }),
///     "$skip=5".try_into(),
/// );
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Skip<'src> {
    Offset(u128),
    Cursor(&'src str),
}

impl<'tokens, 'src: 'tokens> TokenParser<'tokens, 'src> for Skip<'src> {
    fn parser<I: ValueInput<'tokens, Token = Token<'src>, Span = SimpleSpan>>()
    -> impl Parser<'tokens, I, Self, extra::Err<Rich<'tokens, Token<'src>>>> + Clone {
        just(Token::Kw(Keyword::Skip)).ignore_then(
            select! { Token::Offset(offset) => offset }
                .map(Self::Offset)
                .labelled("offset")
                .or(select! { Token::Str(str) => str }
                    .map(Self::Cursor)
                    .labelled("cursor")),
        )
    }
}

impl<'src> TryFrom<&'src str> for Skip<'src> {
    type Error = String;
    fn try_from(value: &'src str) -> Result<Self, Self::Error> {
        Self::try_from_str(value)
    }
}

impl Display for Skip<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "$skip=")?;
        match self {
            Self::Offset(o) => write!(f, "{o}"),
            Self::Cursor(c) => write!(f, "{c}"),
        }
    }
}

/// The [OData $top query option](https://docs.oasis-open.org/odata/odata/v4.01/odata-v4.01-part1-protocol.html#sec_SystemQueryOptiontop).
///
/// # Examples
///
/// ```
/// use odata::{OData, pagination::Top};
/// assert_eq!(
///     Ok(OData {
///         top: Some(Top::Offset(5)),
///         ..Default::default()
///     }),
///     "$top=5".try_into(),
/// );
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Top<'src> {
    Offset(u128),
    Cursor(&'src str),
}

impl<'tokens, 'src: 'tokens> TokenParser<'tokens, 'src> for Top<'src> {
    fn parser<I: ValueInput<'tokens, Token = Token<'src>, Span = SimpleSpan>>()
    -> impl Parser<'tokens, I, Self, extra::Err<Rich<'tokens, Token<'src>>>> + Clone {
        just(Token::Kw(Keyword::Top)).ignore_then(
            select! { Token::Offset(offset) => offset }
                .map(Self::Offset)
                .labelled("offset")
                .or(select! { Token::Str(str) => str }
                    .map(Self::Cursor)
                    .labelled("cursor")),
        )
    }
}

impl<'src> TryFrom<&'src str> for Top<'src> {
    type Error = String;
    fn try_from(value: &'src str) -> Result<Self, Self::Error> {
        Self::try_from_str(value)
    }
}

impl Display for Top<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "$skip=")?;
        match self {
            Self::Offset(o) => write!(f, "{o}"),
            Self::Cursor(c) => write!(f, "{c}"),
        }
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
                Ok(Skip::Offset(42)),
                Ok(Skip::Cursor("jc8mUc")),
                Ok(Top::Offset(42)),
                Ok(Top::Cursor("jc8mUc")),
            ),
            (
                "$skip=42".try_into(),
                "$skip='jc8mUc'".try_into(),
                "$top=42".try_into(),
                "$top='jc8mUc'".try_into(),
            )
        );
    }

    #[test]
    fn invalid() {
        insta::assert_snapshot!(
            Skip::try_from("").unwrap_err()
                + "\n"
                + &Skip::try_from("$skip=text").unwrap_err()
                + "\n"
                + &Top::try_from("").unwrap_err()
                + "\n"
                + &Top::try_from("$top=text").unwrap_err()
        );
    }
}
