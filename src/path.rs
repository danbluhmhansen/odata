use std::fmt::Display;

use chumsky::{Parser, input::ValueInput, prelude::*};
use derive_more::{Display, From, IntoIterator};

use crate::token::{Ctrl, Keyword, Token, TokenParser};

/// A collection of OData identifiers.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, From, IntoIterator)]
pub struct Path<'src>(#[into_iterator(owned, ref, ref_mut)] Vec<Segment<'src>>);

impl<'tokens, 'src: 'tokens> TokenParser<'tokens, 'src> for Path<'src> {
    fn parser<I: ValueInput<'tokens, Token = Token<'src>, Span = SimpleSpan>>()
    -> impl Parser<'tokens, I, Self, extra::Err<Rich<'tokens, Token<'src>>>> + Clone {
        Segment::parser()
            .separated_by(just(Token::Ctrl(Ctrl::Slash)))
            .collect()
            .map(Self)
    }
}

impl<'src> TryFrom<&'src str> for Path<'src> {
    type Error = String;
    fn try_from(value: &'src str) -> Result<Self, Self::Error> {
        Self::try_from_str(value)
    }
}

impl Display for Path<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut first = true;
        for segment in self {
            if !first {
                write!(f, "/")?;
            }
            first = false;
            write!(f, "{segment}")?;
        }
        Ok(())
    }
}

impl<'src> From<Segment<'src>> for Path<'src> {
    fn from(value: Segment<'src>) -> Self {
        Self(vec![value])
    }
}

impl<'src> From<Ident<'src>> for Path<'src> {
    fn from(value: Ident<'src>) -> Self {
        Self(vec![value.into()])
    }
}

// An OData identifier, or the `.../$count` segment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, From, Display)]
pub enum Segment<'src> {
    Ident(Ident<'src>),
    Count,
}

impl<'tokens, 'src: 'tokens> TokenParser<'tokens, 'src> for Segment<'src> {
    fn parser<I: ValueInput<'tokens, Token = Token<'src>, Span = SimpleSpan>>()
    -> impl Parser<'tokens, I, Self, extra::Err<Rich<'tokens, Token<'src>>>> + Clone {
        choice((
            Ident::parser().map(Self::Ident),
            just(Token::Kw(Keyword::Count)).to(Self::Count),
        ))
    }
}

impl<'src> TryFrom<&'src str> for Segment<'src> {
    type Error = String;
    fn try_from(value: &'src str) -> Result<Self, Self::Error> {
        Self::try_from_str(value)
    }
}

// An OData identifier.
//
// Must start with an alphabetical char or `_`, and then contain only alphanumeric chars or `_`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Display)]
pub struct Ident<'src>(pub &'src str);

impl<'tokens, 'src: 'tokens> TokenParser<'tokens, 'src> for Ident<'src> {
    fn parser<I: ValueInput<'tokens, Token = Token<'src>, Span = SimpleSpan>>()
    -> impl Parser<'tokens, I, Self, extra::Err<Rich<'tokens, Token<'src>>>> + Clone {
        select! { Token::Ident(i) => Self(i) }
    }
}

impl<'src> TryFrom<&'src str> for Ident<'src> {
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
            (Ok(Path(vec![
                Segment::Ident(Ident("one")),
                Segment::Ident(Ident("two")),
                Segment::Ident(Ident("three")),
                Segment::Count,
            ])),),
            ("one/two/three/$count".try_into(),)
        );
    }
}
