use std::fmt::Display;

use chumsky::{Parser, input::ValueInput, prelude::*};
use derive_more::From;

use crate::{
    path::Path,
    token::{Ctrl, Keyword, Token, TokenParser},
};

/// The [OData $select query option](https://docs.oasis-open.org/odata/odata/v4.01/odata-v4.01-part1-protocol.html#sec_SystemQueryOptionselect).
///
/// # Examples
///
/// ```
/// use odata::{OData, path::Ident};
/// assert_eq!(
///     Ok(OData {
///         select: Some(vec![Ident("address").into(), Ident("orders").into()].into()),
///         ..Default::default()
///     }),
///     "$select=address,orders".try_into(),
/// );
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, From)]
pub enum Select<'src> {
    Paths(Vec<Path<'src>>),
    Wildcard,
}

impl<'tokens, 'src: 'tokens> TokenParser<'tokens, 'src> for Select<'src> {
    fn parser<I: ValueInput<'tokens, Token = Token<'src>, Span = SimpleSpan>>()
    -> impl Parser<'tokens, I, Self, extra::Err<Rich<'tokens, Token<'src>>>> + Clone {
        just(Token::Kw(Keyword::Select)).ignore_then(
            just(Token::Ctrl(Ctrl::Wildcard))
                .to(Self::Wildcard)
                .or(Path::parser()
                    .separated_by(just(Token::Ctrl(Ctrl::Comma)))
                    .collect()
                    .map(Self::Paths)
                    .labelled("identifiers")),
        )
    }
}

impl<'src> TryFrom<&'src str> for Select<'src> {
    type Error = String;
    fn try_from(value: &'src str) -> Result<Self, Self::Error> {
        Self::try_from_str(value)
    }
}

impl Display for Select<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "$select=")?;
        match self {
            Select::Paths(idents) => {
                let mut first = true;
                for ident in idents {
                    if !first {
                        write!(f, ",")?;
                    }
                    first = false;
                    write!(f, "{ident}")?;
                }
            }
            Select::Wildcard => write!(f, "*")?,
        };
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use crate::path::Ident;

    use super::*;

    #[test]
    fn valid() {
        assert_eq!(
            (
                Ok(Select::Wildcard),
                Ok(Select::Paths(vec![
                    Ident("one").into(),
                    Ident("two").into(),
                    Ident("three").into()
                ])),
                Ok(Select::Paths(vec![
                    Ident("one").into(),
                    Ident("two").into(),
                    Ident("three").into()
                ])),
            ),
            (
                "$select=*".try_into(),
                "$select=one,two,three".try_into(),
                "$select= one,two , three ".try_into(),
            )
        );
    }

    #[test]
    fn invalid() {
        insta::assert_snapshot!(
            Select::try_from("").unwrap_err()
                + "\n"
                + &Select::try_from("$select='str'").unwrap_err()
        );
    }
}
