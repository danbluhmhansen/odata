//! `odata` is an [OData (Open Data Protocol)](https://www.odata.org/) query options parser.
//!
//! Parsing is done using the [`OData`] struct.
//!
//! # Syntax support
//!
//! ## Identifiers
//!
//! - [x] identifier - see [`path::Ident`].
//! - [x] count segment - `.../$count` see [`path::Segment`].
//! - [x] paths - `foo/bar/baz` see [`path::Path`].
//! - [ ] keys - `foo(1)/bar`.
//! - [ ] references - `.../$ref`.
//!
//! ## Literal data values
//!
//! - [x] null - `null`.
//! - [x] booleans - `true` | `false`.
//! - [x] numbers - using [rust_decimal](https://crates.io/crates/rust_decimal) if the feature is enabled (default), otherwise `f64`.
//! - [x] text - `'text'` or `''text''`(non-standard specification).
//!   - [ ] escaping `'`.
//! - [ ] UUIDs.
//! - [x] dates/timestamps - using [jiff](https://crates.io/crates/jiff) if the feature is enabled (default), otherwise they have to be parsed as text.
//! - [ ] durations.
//! - [ ] time of day.
//! - [ ] geo literals.
//! - [x] lists - `('one','two','three')` | `['one','two','three']`.
//!
//! ## Query options
//!
//! - [ ] parameter aliases.
//! - [ ] case-insensitive keywords.
//!
//! ### Count
//!
//! - [x] with value - `$count=false`.
//! - [x] implicit - `$count`(true).
//!
//! ### Skip & Top
//!
//! - [x] offsets - `$skip=10&top=5`.
//! - [x] cursors - `$skip='foo'&top='bar'`(non-standard specification).
//!
//! ### Compute
//!
//! - [ ] aliases - `$compute=price mult qty as total_price`.
//!
//! ### Select
//!
//! - [x] identifiers - `$filter=one,two,three`.
//! - [x] wildcard - `$filter=*`.
//!
//! ### Orderby
//!
//! - [x] unary operators - `$orderby=one asc,two desc`.
//! - [x] implicit - `$orderby=one,two`.
//!
//! ### Search
//!
//! - [x] text, no tokens parsed just a raw string - `$search=Inventore vel aut dolorem velit sit repellat.`.
//!
//! ### Filter
//!
//! - [x] comparison operators - `$filter=address/city eq 'Redmond'`.
//! - [x] logical operators - `$filter=price le 200 and price gt 3.5`.
//! - [x] arithmetic operators - `$filter=price add 5 gt 10`.
//! - [x] precedence grouping - `$filter=(price sub 5) gt 10`.
//! - [x] functions - `$filter=contains(company_name,'freds')`.
//! - [x] negation - `$filter=not endswith(description,'milk')`.
//! - [ ] any - `$filter=orders/any(o:o/price gt 100)`.
//!
//! ### Expand
//!
//! - [x] identifiers - `$expand=one,two,three`.
//! - [x] recursion - `$expand=orders($filter=amount gt 100)`.
//! - [ ] levels - `$expand=manager/reports($levels=4)`.

#![forbid(unsafe_code)]

use std::fmt::Display;

use chumsky::{Parser, error::Rich, extra, input::ValueInput, prelude::*, span::SimpleSpan};
use derive_more::From;

use crate::{
    count::Count,
    expand::Expand,
    filter::Filter,
    orderby::Orderby,
    pagination::{Skip, Top},
    search::Search,
    select::Select,
    token::{Ctrl, Token, TokenParser},
};

pub mod count;
pub mod expand;
pub mod filter;
pub mod orderby;
pub mod pagination;
pub mod path;
pub mod search;
pub mod select;
pub mod token;

/// Represents [OData system query options](https://docs.oasis-open.org/odata/odata/v4.01/odata-v4.01-part1-protocol.html#sec_SystemQueryOptions).
///
/// Parsing is done using the `OData::try_from("")` function.
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
#[derive(Debug, Clone, PartialEq, PartialOrd, Default, From)]
#[from((Count, Expand<'src>, Filter<'src>, Orderby<'src>, Search<'src>, Select<'src>, Skip<'src>, Top<'src>))]
#[from((
    Count,
    Option<Expand<'src>>,
    Option<Filter<'src>>,
    Option<Orderby<'src>>,
    Option<Search<'src>>,
    Option<Select<'src>>,
    Option<Skip<'src>>,
    Option<Top<'src>>,
))]
pub struct OData<'src> {
    pub count: Count,
    pub expand: Option<Expand<'src>>,
    pub filter: Option<Filter<'src>>,
    pub orderby: Option<Orderby<'src>>,
    pub search: Option<Search<'src>>,
    pub select: Option<Select<'src>>,
    pub skip: Option<Skip<'src>>,
    pub top: Option<Top<'src>>,
}

impl<'tokens, 'src: 'tokens> TokenParser<'tokens, 'src> for OData<'src> {
    fn parser<I: ValueInput<'tokens, Token = Token<'src>, Span = SimpleSpan>>()
    -> impl Parser<'tokens, I, Self, extra::Err<Rich<'tokens, Token<'src>>>> + Clone {
        ODataItem::parser()
            .separated_by(just(Token::Ctrl(Ctrl::Delim)))
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
                        ODataItem::Count(c) => {
                            count.replace(c);
                        }
                        ODataItem::Expand(e) => {
                            expand.replace(e);
                        }
                        ODataItem::Filter(f) => {
                            filter.replace(f);
                        }
                        ODataItem::Orderby(o) => {
                            orderby.replace(o);
                        }
                        ODataItem::Search(s) => {
                            search.replace(s);
                        }
                        ODataItem::Select(s) => {
                            select.replace(s);
                        }
                        ODataItem::Skip(s) => {
                            skip.replace(s);
                        }
                        ODataItem::Top(t) => {
                            top.replace(t);
                        }
                    }
                }

                let count = count.unwrap_or_default();
                Self {
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
    }
}

/// Used to parse a [`&str`] into an [`OData`] struct.
///
/// The error type is a formatted [`String`] intended to by returned directly.
///
/// # Examples
///
/// ```
/// use odata::OData;
/// assert_eq!(
///     Err("Error: found 'str' expected '*', identifiers, '&', or end of input
///    ╭─[ <unknown>:1:9 ]
///  1 │$select='str'
///    │          ╰─── found 'str' expected '*', identifiers, '&', or end of input
/// ".into()),
///     OData::try_from("$select='str'"),
/// )
/// ```
impl<'src> TryFrom<&'src str> for OData<'src> {
    type Error = String;
    fn try_from(value: &'src str) -> Result<Self, Self::Error> {
        Self::try_from_str(value)
    }
}

impl Display for OData<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut first = true;

        if *self.count {
            first = false;
            write!(f, "{}", self.count)?;
        }

        if let Some(expand) = &self.expand {
            if !first {
                write!(f, "&")?;
            }
            first = false;
            write!(f, "{expand}")?;
        }

        if let Some(filter) = &self.filter {
            if !first {
                write!(f, "&")?;
            }
            first = false;
            write!(f, "{filter}")?;
        }

        if let Some(orderby) = &self.orderby {
            if !first {
                write!(f, "&")?;
            }
            first = false;
            write!(f, "{orderby}")?;
        }

        if let Some(search) = &self.search {
            if !first {
                write!(f, "&")?;
            }
            first = false;
            write!(f, "{search}")?;
        }

        if let Some(select) = &self.select {
            if !first {
                write!(f, "&")?;
            }
            first = false;
            write!(f, "{select}")?;
        }

        if let Some(skip) = &self.skip {
            if !first {
                write!(f, "&")?;
            }
            first = false;
            write!(f, "{skip}")?;
        }

        if let Some(top) = &self.top {
            if !first {
                write!(f, "&")?;
            }
            write!(f, "{top}")?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
enum ODataItem<'src> {
    Count(Count),
    Expand(Expand<'src>),
    Filter(Filter<'src>),
    Orderby(Orderby<'src>),
    Search(Search<'src>),
    Select(Select<'src>),
    Skip(Skip<'src>),
    Top(Top<'src>),
}

impl<'tokens, 'src: 'tokens> ODataItem<'src> {
    fn parser<I: ValueInput<'tokens, Token = Token<'src>, Span = SimpleSpan>>()
    -> impl Parser<'tokens, I, Self, extra::Err<Rich<'tokens, Token<'src>>>> + Clone {
        choice((
            Count::parser().map(Self::Count),
            Expand::parser().map(Self::Expand),
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
    fn empty() {
        assert_eq!(Ok(OData::default()), "".try_into());
    }

    #[test]
    fn select() {
        assert_eq!(
            Ok(OData {
                select: Some(vec![Ident("address").into(), Ident("orders").into()].into()),
                ..Default::default()
            }),
            "$select=address,orders".try_into(),
        );
    }

    #[test]
    fn full() {
        assert_eq!(
            Ok(OData {
                count: true.into(),
                expand: Some(
                    vec![(
                        Ident("rel").into(),
                        OData {
                            count: true.into(),
                            select: Some(vec![Ident("sub").into()].into()),
                            ..Default::default()
                        }
                    )]
                    .into()
                ),
                filter: Some(Filter::from((
                    Filter::Path(Ident("name").into()),
                    BinOp::Eq,
                    Filter::Str("milk")
                ))),
                orderby: Some(
                    vec![
                        ("one", Direction::Asc),
                        ("two", Direction::Asc),
                        ("three", Direction::Desc),
                    ]
                    .into()
                ),
                search: Some(Search("In sapiente atque eum molestias eos.")),
                select: Some(
                    vec![
                        Ident("one").into(),
                        Ident("two").into(),
                        Ident("three").into()
                    ]
                    .into()
                ),
                skip: Some(Skip::Offset(20)),
                top: Some(Top::Offset(10))
            }),
            concat!(
                "$count&",
                "$expand=rel($count,select=sub)&",
                "$filter=name eq 'milk'&",
                "$orderby=one,two asc,three desc&",
                "$search=In sapiente atque eum molestias eos.&",
                "$select=one,two,three&",
                "$skip=20&",
                "$top=10&",
            )
            .try_into(),
        );
    }

    #[cfg(not(feature = "jiff"))]
    #[test]
    fn complex() {
        assert_eq!(
            Ok(OData {
                count: true.into(),
                expand: Some(
                    vec![(
                        Ident("Order_Details").into(),
                        OData {
                            select: Some(Select::Paths(vec![
                                Ident("ProductID").into(),
                                Ident("UnitPrice").into(),
                                Ident("Quantity").into(),
                                Ident("Discount").into(),
                            ])),
                            ..Default::default()
                        }
                    )]
                    .into()
                ),
                filter: Some(Filter::from((
                    Filter::from((
                        Filter::from((
                            Filter::from((
                                Filter::from((
                                    Filter::from(Ident("OrderDate")),
                                    BinOp::Ge,
                                    Filter::Str("2023-01-01"),
                                )),
                                BinOp::And,
                                Filter::from((
                                    Filter::from(Ident("OrderDate")),
                                    BinOp::Le,
                                    Filter::Str("2023-12-31"),
                                )),
                            )),
                            BinOp::And,
                            Filter::from((
                                Filter::from(Ident("ShipCountry")),
                                BinOp::Eq,
                                Filter::Str("USA"),
                            )),
                        )),
                        BinOp::And,
                        Filter::from((
                            Filter::from((
                                Filter::from(Ident("ShipCity")),
                                BinOp::Eq,
                                Filter::Str("Seattle"),
                            )),
                            BinOp::Or,
                            Filter::from((
                                Filter::from(Ident("ShipCity")),
                                BinOp::Eq,
                                Filter::Str("New York"),
                            )),
                        )),
                    )),
                    BinOp::And,
                    Filter::from((
                        Filter::from((
                            Filter::from(Ident("CustomerID")),
                            BinOp::Eq,
                            Filter::Str("ALFKI"),
                        )),
                        BinOp::Or,
                        Filter::from((
                            Filter::from(Ident("CustomerID")),
                            BinOp::Eq,
                            Filter::Str("ANATR"),
                        )),
                    )),
                ))),
                orderby: Some(
                    vec![("OrderDate", Direction::Desc), ("ShipCity", Direction::Asc),].into()
                ),
                search: None,
                select: Some(Select::Paths(vec![
                    Ident("OrderID").into(),
                    Ident("OrderDate").into(),
                    Ident("ShipName").into(),
                    Ident("ShipCity").into(),
                    Ident("ShipCountry").into(),
                    Ident("CustomerID").into(),
                ])),
                skip: Some(Skip::Offset(5)),
                top: Some(Top::Offset(10))
            }),
            concat!(
                "$filter=OrderDate ge '2023-01-01' and OrderDate le '2023-12-31' and ",
                "ShipCountry eq 'USA' and ",
                "(ShipCity eq 'Seattle' or ShipCity eq 'New York') and ",
                "(CustomerID eq 'ALFKI' or CustomerID eq 'ANATR')&",
                "$orderby=OrderDate desc, ShipCity asc&",
                "$select=OrderID, OrderDate, ShipName, ShipCity, ShipCountry, CustomerID&",
                "$expand=Order_Details($select=ProductID, UnitPrice, Quantity, Discount)&",
                "$count=true&",
                "$top=10&",
                "$skip=5&",
            )
            .try_into(),
        );
    }

    #[cfg(feature = "jiff")]
    #[test]
    fn complex() {
        assert_eq!(
            Ok(OData {
                count: true.into(),
                expand: Some(
                    vec![(
                        Ident("Order_Details").into(),
                        OData {
                            select: Some(Select::Paths(vec![
                                Ident("ProductID").into(),
                                Ident("UnitPrice").into(),
                                Ident("Quantity").into(),
                                Ident("Discount").into(),
                            ])),
                            ..Default::default()
                        }
                    )]
                    .into()
                ),
                filter: Some(Filter::from((
                    Filter::from((
                        Filter::from((
                            Filter::from((
                                Filter::from((
                                    Filter::from(Ident("OrderDate")),
                                    BinOp::Ge,
                                    Filter::Date(
                                        jiff::civil::Date::new(2023, 1, 1).unwrap().into()
                                    ),
                                )),
                                BinOp::And,
                                Filter::from((
                                    Filter::from(Ident("OrderDate")),
                                    BinOp::Le,
                                    Filter::Date(
                                        jiff::civil::Date::new(2023, 12, 31).unwrap().into()
                                    ),
                                )),
                            )),
                            BinOp::And,
                            Filter::from((
                                Filter::from(Ident("ShipCountry")),
                                BinOp::Eq,
                                Filter::Str("USA"),
                            )),
                        )),
                        BinOp::And,
                        Filter::from((
                            Filter::from((
                                Filter::from(Ident("ShipCity")),
                                BinOp::Eq,
                                Filter::Str("Seattle"),
                            )),
                            BinOp::Or,
                            Filter::from((
                                Filter::from(Ident("ShipCity")),
                                BinOp::Eq,
                                Filter::Str("New York"),
                            )),
                        )),
                    )),
                    BinOp::And,
                    Filter::from((
                        Filter::from((
                            Filter::from(Ident("CustomerID")),
                            BinOp::Eq,
                            Filter::Str("ALFKI"),
                        )),
                        BinOp::Or,
                        Filter::from((
                            Filter::from(Ident("CustomerID")),
                            BinOp::Eq,
                            Filter::Str("ANATR"),
                        )),
                    )),
                ))),
                orderby: Some(
                    vec![("OrderDate", Direction::Desc), ("ShipCity", Direction::Asc),].into()
                ),
                search: None,
                select: Some(Select::Paths(vec![
                    Ident("OrderID").into(),
                    Ident("OrderDate").into(),
                    Ident("ShipName").into(),
                    Ident("ShipCity").into(),
                    Ident("ShipCountry").into(),
                    Ident("CustomerID").into(),
                ])),
                skip: Some(Skip::Offset(5)),
                top: Some(Top::Offset(10))
            }),
            concat!(
                "$filter=OrderDate ge 2023-01-01 and OrderDate le 2023-12-31 and ",
                "ShipCountry eq 'USA' and ",
                "(ShipCity eq 'Seattle' or ShipCity eq 'New York') and ",
                "(CustomerID eq 'ALFKI' or CustomerID eq 'ANATR')&",
                "$orderby=OrderDate desc, ShipCity asc&",
                "$select=OrderID, OrderDate, ShipName, ShipCity, ShipCountry, CustomerID&",
                "$expand=Order_Details($select=ProductID, UnitPrice, Quantity, Discount)&",
                "$count=true&",
                "$top=10&",
                "$skip=5&",
            )
            .try_into(),
        );
    }
}
