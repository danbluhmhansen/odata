use std::fmt::Display;

use chumsky::{Parser, input::ValueInput, prelude::*};
use derive_more::{Display, From, IntoIterator};

use crate::token::{Ctrl, Keyword, Token, TokenParser, UnOp};

/// The [OData $orderby query option](https://docs.oasis-open.org/odata/odata/v4.01/odata-v4.01-part1-protocol.html#sec_SystemQueryOptionorderby).
///
/// # Examples
///
/// ```
/// use odata::{OData, orderby::Direction};
/// assert_eq!(
///     Ok(OData {
///         orderby: Some(vec![("release_date", Direction::Asc), ("rating", Direction::Desc)].into()),
///         ..Default::default()
///     }),
///     "$orderby=release_date asc, rating desc".try_into(),
/// );
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, From, IntoIterator)]
pub struct Orderby<'src>(#[into_iterator(owned, ref, ref_mut)] Vec<(&'src str, Direction)>);

impl<'tokens, 'src: 'tokens> TokenParser<'tokens, 'src> for Orderby<'src> {
    fn parser<I: ValueInput<'tokens, Token = Token<'src>, Span = SimpleSpan>>()
    -> impl Parser<'tokens, I, Self, extra::Err<Rich<'tokens, Token<'src>>>> + Clone {
        just(Token::Kw(Keyword::Orderby)).ignore_then(
            select! { Token::Ident(ident) => ident }
                .then(Direction::parser())
                .separated_by(just(Token::Ctrl(Ctrl::Comma)))
                .collect()
                .map(Self)
                .labelled("identifiers with direction"),
        )
    }
}

impl<'src> TryFrom<&'src str> for Orderby<'src> {
    type Error = String;
    fn try_from(value: &'src str) -> Result<Self, Self::Error> {
        Self::try_from_str(value)
    }
}

impl Display for Orderby<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut first = true;
        write!(f, "$orderby=")?;
        for (ident, dir) in &self.0 {
            if !first {
                write!(f, ",")?;
            }
            first = false;
            write!(f, "{ident} {dir}")?;
        }
        Ok(())
    }
}

/// [`Orderby`] unary direction operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Display)]
#[display(rename_all = "lowercase")]
pub enum Direction {
    #[default]
    Asc,
    Desc,
}

impl<'tokens, 'src: 'tokens> TokenParser<'tokens, 'src> for Direction {
    fn parser<I: ValueInput<'tokens, Token = Token<'src>, Span = SimpleSpan>>()
    -> impl Parser<'tokens, I, Self, extra::Err<Rich<'tokens, Token<'src>>>> + Clone {
        choice((
            select! { Token::UnOp(UnOp::Asc) => Direction::Asc },
            select! { Token::UnOp(UnOp::Desc) => Direction::Desc },
        ))
        .or_not()
        .map(|dir| dir.unwrap_or_default())
    }
}

impl<'src> TryFrom<&'src str> for Direction {
    type Error = String;
    fn try_from(value: &'src str) -> Result<Self, Self::Error> {
        Self::try_from_str(value)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn valid() {
        assert_eq!(
            (Ok(Orderby(vec![
                ("one", Direction::Asc),
                ("two", Direction::Asc),
                ("three", Direction::Desc)
            ])),),
            ("$orderby=one,two asc,three desc".try_into(),)
        );
    }

    #[test]
    fn invalid() {
        insta::assert_snapshot!(
            Orderby::try_from("").unwrap_err()
                + "\n"
                + &Orderby::try_from("$orderby='str'").unwrap_err()
        );
    }
}
