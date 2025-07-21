use std::fmt::Display;

use chumsky::{Parser, input::ValueInput, prelude::*};
use derive_more::Deref;

use crate::token::{Token, TokenParser};

/// The [OData $search query option](https://docs.oasis-open.org/odata/odata/v4.01/odata-v4.01-part1-protocol.html#sec_SystemQueryOptionsearch).
///
/// # Examples
///
/// ```
/// use odata::{OData, search::Search};
/// assert_eq!(
///     Ok(OData {
///         search: Some(Search("bike")),
///         ..Default::default()
///     }),
///     "$search=bike".try_into(),
/// );
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Deref)]
pub struct Search<'src>(pub &'src str);

impl<'tokens, 'src: 'tokens> TokenParser<'tokens, 'src> for Search<'src> {
    fn parser<I: ValueInput<'tokens, Token = Token<'src>, Span = SimpleSpan>>()
    -> impl Parser<'tokens, I, Self, extra::Err<Rich<'tokens, Token<'src>>>> + Clone {
        select! { Token::Search(s) => Self(s) }.labelled("'search'")
    }
}

impl<'src> TryFrom<&'src str> for Search<'src> {
    type Error = String;
    fn try_from(value: &'src str) -> Result<Self, Self::Error> {
        Self::try_from_str(value)
    }
}

impl Display for Search<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "$search={}", self.0)
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
                Ok(Search("In sapiente atque eum molestias eos.")),
                Ok(Search("Quasi est & consequatur sed eligendi.")),
                Ok(Search("Animi non 'saepe' sed laudantium."))
            ),
            (
                "$search=In sapiente atque eum molestias eos.".try_into(),
                "$search='Quasi est & consequatur sed eligendi.'".try_into(),
                "$search=''Animi non 'saepe' sed laudantium.''".try_into(),
            ),
        );
    }
}
