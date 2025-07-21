use std::fmt::Display;

use chumsky::{Parser, input::ValueInput, prelude::*};
use derive_more::From;

#[cfg(feature = "jiff")]
use crate::token::Date;
use crate::{
    path::{Ident, Path},
    token::{BinOp, Ctrl, Keyword, Token, TokenParser, UnOp},
};

/// The [OData $filter query option](https://docs.oasis-open.org/odata/odata/v4.01/odata-v4.01-part1-protocol.html#sec_SystemQueryOptionfilter).
///
/// # Examples
///
/// ```
/// use odata::{OData, filter::Filter, path::Ident, token::BinOp};
/// assert_eq!(
///     Ok(OData {
///         filter: Some(Filter::from((
///             Filter::Path(Ident("price").into()),
///             BinOp::Lt,
///             Filter::from(10),
///         ))),
///         ..Default::default()
///     }),
///     "$filter=price lt 10.00".try_into(),
/// );
/// ```
#[derive(Debug, Clone, PartialEq, PartialOrd, From)]
pub enum Filter<'src> {
    #[from(())]
    Null,
    Bool(bool),
    #[cfg(not(feature = "rust_decimal"))]
    #[from(i8, i16, i32, u8, u16, u32)]
    Num(f64),
    #[cfg(feature = "rust_decimal")]
    #[from(i8, i16, i32, u8, u16, u32)]
    Num(rust_decimal::Decimal),
    #[from(skip)]
    Str(&'src str),
    #[cfg(feature = "jiff")]
    Date(Date),
    List(Vec<Self>),
    #[from(Path<'src>)]
    Path(Path<'src>),
    #[from((&'src str, Vec<Self>))]
    Fn {
        name: &'src str,
        params: Vec<Self>,
    },
    Not(Box<Self>),
    #[from((Self, BinOp, Self))]
    Bin(Box<Self>, BinOp, Box<Self>),
}

impl<'src> Filter<'src> {
    fn fold_bin(lhs: Self, (op, rhs): (BinOp, Self)) -> Self {
        Self::Bin(lhs.into(), op, rhs.into())
    }
}

impl<'tokens, 'src: 'tokens> TokenParser<'tokens, 'src> for Filter<'src> {
    fn parser<I: ValueInput<'tokens, Token = Token<'src>, Span = SimpleSpan>>()
    -> impl Parser<'tokens, I, Self, extra::Err<Rich<'tokens, Token<'src>>>> + Clone {
        just(Token::Kw(Keyword::Filter)).ignore_then(recursive(|expr| {
            let atom = choice((
                expr.clone().delimited_by(
                    just(Token::Ctrl(Ctrl::LParen)),
                    just(Token::Ctrl(Ctrl::RParen)),
                ),
                select! { Token::Ident(i) => i }
                    .then(
                        expr.clone()
                            .separated_by(just(Token::Ctrl(Ctrl::Comma)))
                            .collect()
                            .delimited_by(
                                just(Token::Ctrl(Ctrl::LParen)),
                                just(Token::Ctrl(Ctrl::RParen)),
                            ),
                    )
                    .map(|(name, params)| Self::Fn { name, params }),
                just(Token::Null).to(Self::Null),
                select! { Token::Bool(b) => b }.map(Self::Bool),
                #[cfg(not(feature = "rust_decimal"))]
                select! { Token::Offset(n) => n }.map(|n| Self::Num(n as f64)),
                #[cfg(feature = "rust_decimal")]
                select! { Token::Offset(n) => n }.map(|n| Self::Num(n.into())),
                select! { Token::Num(n) => n }.map(Self::Num),
                select! { Token::Str(s) => s }.map(Self::Str),
                #[cfg(feature = "jiff")]
                select! { Token::Date(d) => d }.map(Self::Date),
                expr.clone()
                    .separated_by(just(Token::Ctrl(Ctrl::Comma)))
                    .collect()
                    .map(Self::List)
                    .delimited_by(
                        just(Token::Ctrl(Ctrl::LParen)),
                        just(Token::Ctrl(Ctrl::RParen)),
                    )
                    .boxed(),
                expr.clone()
                    .separated_by(just(Token::Ctrl(Ctrl::Comma)))
                    .collect()
                    .map(Self::List)
                    .delimited_by(
                        just(Token::Ctrl(Ctrl::LBracket)),
                        just(Token::Ctrl(Ctrl::RBracket)),
                    )
                    .boxed(),
                Path::parser().map(Self::Path),
                expr,
            ));

            let has_op = just(Token::BinOp(BinOp::Has)).to(BinOp::Has);
            let has_chain = atom
                .clone()
                .foldl(has_op.then(atom).repeated(), Self::fold_bin)
                .boxed();

            let in_op = just(Token::BinOp(BinOp::In)).to(BinOp::In);
            let in_chain = has_chain
                .clone()
                .foldl(in_op.then(has_chain).repeated(), Self::fold_bin)
                .boxed();

            let not_op = just(Token::UnOp(UnOp::Not));
            let not_chain = not_op
                .repeated()
                .foldr(in_chain, |_, e| Self::Not(e.into()))
                .boxed();

            let mul_op = choice((
                just(Token::BinOp(BinOp::Mul)).to(BinOp::Mul),
                just(Token::BinOp(BinOp::Div)).to(BinOp::Div),
                just(Token::BinOp(BinOp::Divby)).to(BinOp::Divby),
                just(Token::BinOp(BinOp::Mod)).to(BinOp::Mod),
            ));
            let mul_chain = not_chain
                .clone()
                .foldl(mul_op.then(not_chain).repeated(), Self::fold_bin)
                .boxed();

            let add_op = choice((
                just(Token::BinOp(BinOp::Add)).to(BinOp::Add),
                just(Token::BinOp(BinOp::Sub)).to(BinOp::Sub),
            ));
            let add_chain = mul_chain
                .clone()
                .foldl(add_op.then(mul_chain).repeated(), Self::fold_bin)
                .boxed();

            let rel_op = choice((
                just(Token::BinOp(BinOp::Gt)).to(BinOp::Gt),
                just(Token::BinOp(BinOp::Ge)).to(BinOp::Ge),
                just(Token::BinOp(BinOp::Lt)).to(BinOp::Lt),
                just(Token::BinOp(BinOp::Le)).to(BinOp::Le),
            ));
            let rel_chain = add_chain
                .clone()
                .foldl(rel_op.then(add_chain).repeated(), Self::fold_bin)
                .boxed();

            let cmp_op = choice((
                just(Token::BinOp(BinOp::Eq)).to(BinOp::Eq),
                just(Token::BinOp(BinOp::Ne)).to(BinOp::Ne),
            ));
            let cmp_chain = rel_chain
                .clone()
                .foldl(cmp_op.then(rel_chain).repeated(), Self::fold_bin)
                .boxed();

            let and_op = just(Token::BinOp(BinOp::And)).to(BinOp::And);
            let and_chain = cmp_chain
                .clone()
                .foldl(and_op.then(cmp_chain).repeated(), Self::fold_bin)
                .boxed();

            let or_op = just(Token::BinOp(BinOp::Or)).to(BinOp::Or);
            and_chain
                .clone()
                .foldl(or_op.then(and_chain).repeated(), Self::fold_bin)
                .boxed()
        }))
    }
}

impl<'src> TryFrom<&'src str> for Filter<'src> {
    type Error = String;
    fn try_from(value: &'src str) -> Result<Self, Self::Error> {
        Self::try_from_str(value)
    }
}

impl Display for Filter<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "$filter=")?;
        match self {
            Filter::Null => write!(f, "null"),
            Filter::Bool(b) => write!(f, "{b}"),
            Filter::Num(n) => write!(f, "{n}"),
            Filter::Str(s) => write!(f, "'{s}'"),
            #[cfg(feature = "jiff")]
            Filter::Date(d) => write!(f, "{d}"),
            Filter::List(l) => {
                let mut first = true;
                for i in l {
                    if !first {
                        write!(f, ",")?;
                    }
                    first = false;
                    write!(f, "{i}")?;
                }
                Ok(())
            }
            Filter::Path(i) => write!(f, "{i}"),
            Filter::Fn { name, params } => {
                let mut first = true;
                write!(f, "{name}(")?;
                for param in params {
                    if !first {
                        write!(f, ",")?;
                    }
                    first = false;
                    write!(f, "{param}")?;
                }
                write!(f, ")")
            }
            Filter::Not(filter) => write!(f, "not {filter}"),
            Filter::Bin(lhs, op, rhs) => write!(f, "{lhs} {op} {rhs}"),
        }
    }
}

impl<'src> From<Ident<'src>> for Filter<'src> {
    fn from(value: Ident<'src>) -> Self {
        Self::Path(value.into())
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use crate::path::{Ident, Segment};

    use super::*;

    #[test]
    fn eq_str() {
        assert_eq!(
            Ok(Filter::from((
                Filter::from(Ident("name")),
                BinOp::Eq,
                Filter::Str("milk")
            ))),
            "$filter=name eq 'milk'".try_into()
        );
    }

    #[test]
    fn gt_num() {
        assert_eq!(
            Ok(Filter::from((
                Filter::from(Ident("price")),
                BinOp::Gt,
                Filter::from(42)
            ))),
            "$filter=price gt 42".try_into()
        );
    }

    #[test]
    fn and() {
        assert_eq!(
            Ok(Filter::from((
                Filter::from((Filter::from(Ident("name")), BinOp::Eq, Filter::Str("milk"))),
                BinOp::And,
                Filter::from((
                    Filter::from(Ident("price")),
                    BinOp::Lt,
                    #[cfg(not(feature = "rust_decimal"))]
                    Filter::Num(2.55),
                    #[cfg(feature = "rust_decimal")]
                    Filter::Num(rust_decimal::Decimal::new(255, 2)),
                )),
            ))),
            "$filter=name eq 'milk' and price lt 2.55".try_into()
        );
    }

    #[test]
    fn endswith() {
        assert_eq!(
            Ok(Filter::Not(
                Filter::from((
                    "endswith",
                    vec![Filter::Path(Ident("name").into()), Filter::Str("ilk")]
                ))
                .into()
            )),
            "$filter=not endswith(name,'ilk')".try_into()
        );
    }

    #[test]
    fn group() {
        assert_eq!(
            Ok(Filter::from((
                Filter::from((
                    Filter::from((Filter::from(4), BinOp::Add, Filter::from(5))),
                    BinOp::Mod,
                    Filter::from((Filter::from(4), BinOp::Sub, Filter::from(1))),
                )),
                BinOp::Eq,
                Filter::from(0),
            ))),
            "$filter=(4 add 5) mod (4 sub 1) eq 0".try_into()
        );
    }

    #[test]
    fn concat() {
        assert_eq!(
            Ok(Filter::from((
                Filter::from((
                    "concat",
                    vec![
                        Filter::from((
                            "concat",
                            vec![Filter::Path(Ident("city").into()), Filter::Str(", ")]
                        )),
                        Filter::Path(Ident("country").into())
                    ]
                )),
                BinOp::Eq,
                Filter::Str("Berlin, Germany"),
            ))),
            "$filter=concat(concat(city,', '), country) eq 'Berlin, Germany'".try_into()
        );
    }

    #[test]
    fn r#in() {
        assert_eq!(
            Ok(Filter::from((
                Filter::Path(Ident("name").into()),
                BinOp::In,
                Filter::List(vec![Filter::Str("milk"), Filter::Str("cheese")]),
            ))),
            "$filter=name in ('milk', 'cheese')".try_into()
        );
    }

    #[test]
    fn count() {
        assert_eq!(
            Ok(Filter::from((
                Filter::from(Path::from(vec![
                    Segment::Ident(Ident("products")),
                    Segment::Count
                ])),
                BinOp::Gt,
                Filter::from(0),
            ))),
            "$filter=products/$count gt 0".try_into(),
        );
    }
}
