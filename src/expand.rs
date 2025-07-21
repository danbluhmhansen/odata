use std::fmt::Display;

use chumsky::{Parser, input::ValueInput, prelude::*};
use derive_more::{From, IntoIterator};

use crate::{
    OData,
    count::Count,
    filter::Filter,
    orderby::Orderby,
    pagination::{Skip, Top},
    path::Path,
    search::Search,
    select::Select,
    token::{Ctrl, Keyword, Token, TokenParser},
};

/// The [OData $expand query option](https://docs.oasis-open.org/odata/odata/v4.01/odata-v4.01-part1-protocol.html#sec_SystemQueryOptionexpand).
///
/// # Examples
///
/// ```
/// use odata::{OData, path::Ident};
/// assert_eq!(
///     Ok(OData {
///         expand: Some(vec![(Ident("orders").into(), OData::default())].into()),
///         ..Default::default()
///     }),
///     "$expand=orders".try_into(),
/// );
/// ```
#[derive(Debug, Clone, PartialEq, PartialOrd, From, IntoIterator)]
pub struct Expand<'src>(#[into_iterator(owned, ref, ref_mut)] Vec<(Path<'src>, OData<'src>)>);

impl<'tokens, 'src: 'tokens> TokenParser<'tokens, 'src> for Expand<'src> {
    fn parser<I: ValueInput<'tokens, Token = Token<'src>, Span = SimpleSpan>>()
    -> impl Parser<'tokens, I, Self, extra::Err<Rich<'tokens, Token<'src>>>> + Clone {
        recursive(|exp| {
            just(Token::Kw(Keyword::Expand)).ignore_then(
                Path::parser()
                    .then(
                        ExpandItem::parser(exp)
                            .separated_by(just(Token::Ctrl(Ctrl::Comma)))
                            .allow_trailing()
                            .collect::<Vec<_>>()
                            .map(|items| {
                                let (
                                    mut count,
                                    mut expand,
                                    mut filter,
                                    mut orderby,
                                    mut search,
                                    mut select,
                                    mut skip,
                                    mut top,
                                ) = (None, None, None, None, None, None, None, None);

                                for i in items {
                                    match i {
                                        ExpandItem::Count(c) => {
                                            count.replace(c);
                                        }
                                        ExpandItem::Expand(e) => {
                                            expand.replace(e);
                                        }
                                        ExpandItem::Filter(f) => {
                                            filter.replace(f);
                                        }
                                        ExpandItem::Orderby(o) => {
                                            orderby.replace(o);
                                        }
                                        ExpandItem::Search(s) => {
                                            search.replace(s);
                                        }
                                        ExpandItem::Select(s) => {
                                            select.replace(s);
                                        }
                                        ExpandItem::Skip(s) => {
                                            skip.replace(s);
                                        }
                                        ExpandItem::Top(t) => {
                                            top.replace(t);
                                        }
                                    }
                                }

                                let count = count.unwrap_or_default();
                                OData {
                                    count,
                                    expand,
                                    filter,
                                    orderby,
                                    search,
                                    select,
                                    skip,
                                    top,
                                }
                            })
                            .delimited_by(
                                just(Token::Ctrl(Ctrl::LParen)),
                                just(Token::Ctrl(Ctrl::RParen)),
                            )
                            .or_not()
                            .map(|o| o.unwrap_or_default()),
                    )
                    .separated_by(just(Token::Ctrl(Ctrl::Semi)))
                    .collect()
                    .map(Self),
            )
        })
    }
}

impl<'src> TryFrom<&'src str> for Expand<'src> {
    type Error = String;
    fn try_from(value: &'src str) -> Result<Self, Self::Error> {
        Self::try_from_str(value)
    }
}

impl Display for Expand<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut first = true;
        write!(f, "$expand=")?;
        for (ident, odata) in &self.0 {
            if !first {
                write!(f, ",")?;
            }
            first = false;
            write!(f, "{ident}")?;
            if *odata != OData::default() {
                write!(f, "({odata})")?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
enum ExpandItem<'src> {
    Count(Count),
    Expand(Expand<'src>),
    Filter(Filter<'src>),
    Orderby(Orderby<'src>),
    Search(Search<'src>),
    Select(Select<'src>),
    Skip(Skip<'src>),
    Top(Top<'src>),
}

impl<'tokens, 'src: 'tokens> ExpandItem<'src> {
    fn parser<I: ValueInput<'tokens, Token = Token<'src>, Span = SimpleSpan>>(
        expand_parser: impl Parser<'tokens, I, Expand<'src>, extra::Err<Rich<'tokens, Token<'src>>>>
        + Clone,
    ) -> impl Parser<'tokens, I, Self, extra::Err<Rich<'tokens, Token<'src>>>> + Clone {
        choice((
            Count::parser().map(Self::Count),
            expand_parser.map(Self::Expand),
            Filter::parser().map(Self::Filter),
            Orderby::parser().map(Self::Orderby),
            Search::parser().map(Self::Search),
            Select::parser().map(Self::Select),
            Skip::parser().map(Self::Skip),
            Top::parser().map(Self::Top),
        ))
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use crate::{orderby::Direction, path::Ident, token::BinOp};

    use super::*;

    #[test]
    fn ident() {
        assert_eq!(
            Ok(Expand(vec![(Ident("one").into(), OData::default())])),
            "$expand=one".try_into()
        );
    }

    #[test]
    fn count() {
        assert_eq!(
            Ok(Expand(vec![(
                Ident("one").into(),
                OData {
                    count: true.into(),
                    ..Default::default()
                }
            )])),
            "$expand=one($count)".try_into()
        );
    }

    #[test]
    fn expand() {
        assert_eq!(
            Ok(Expand(vec![(
                Ident("one").into(),
                OData {
                    expand: Some(Expand(vec![(Ident("two").into(), OData::default())])),
                    ..Default::default()
                }
            )])),
            "$expand=one($expand=two)".try_into()
        );
    }

    #[test]
    fn filter() {
        assert_eq!(
            Ok(Expand(vec![(
                Ident("one").into(),
                OData {
                    filter: Some(Filter::from((
                        Filter::from(Ident("name")),
                        BinOp::Eq,
                        Filter::Str("milk")
                    ))),
                    ..Default::default()
                }
            )])),
            "$expand=one($filter=name eq 'milk')".try_into()
        );
    }

    #[test]
    fn orderby() {
        assert_eq!(
            Ok(Expand(vec![(
                Ident("one").into(),
                OData {
                    orderby: Some(
                        vec![
                            ("two", Direction::Asc),
                            ("three", Direction::Asc),
                            ("four", Direction::Desc)
                        ]
                        .into()
                    ),
                    ..Default::default()
                }
            )])),
            "$expand=one($orderby=two,three asc,four desc)".try_into()
        );
    }

    #[test]
    fn select() {
        assert_eq!(
            Ok(Expand(vec![(
                Ident("one").into(),
                OData {
                    select: Some(Select::Paths(vec![
                        Ident("two").into(),
                        Ident("three").into(),
                        Ident("four").into(),
                    ])),
                    ..Default::default()
                }
            )])),
            "$expand=one($select=two,three,four)".try_into()
        );
    }
}
