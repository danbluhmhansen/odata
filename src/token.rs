use ariadne::{Color, Label, Report, ReportKind, Source};
use chumsky::{
    Parser,
    error::Rich,
    extra,
    input::{Input, Stream, ValueInput},
    span::SimpleSpan,
};
use derive_more::Display;
#[cfg(feature = "jiff")]
use derive_more::From;
use logos::Logos;

const CONFIG: ariadne::Config = ariadne::Config::new()
    .with_color(false)
    .with_compact(true)
    .with_index_type(ariadne::IndexType::Byte);

#[derive(Debug, Logos, Clone, Copy, PartialEq, PartialOrd, Display)]
#[logos(skip r"\s+")]
#[logos(subpattern str = r"'[^']*'")]
#[logos(subpattern str2 = r"''(?:[^']|'[^'])*''")]
#[logos(subpattern date = r"\d{4}-(?:0[1-9]|1[0-2])-(?:0[1-9]|[12]\d|3[01])")]
#[logos(subpattern ts =
    r"(?&date)T(?:[01]\d|2[0-3]):(?:[0-5]\d):(?:[0-5]\d)(?:\.\d{1,9})?(?:Z|[+-](?:0[0-9]|1[0-4]):[0-5][0-9])"
)]
pub(crate) enum Token<'src> {
    #[display("<error>")]
    Error,
    #[regex(r"\$?compute=", |_| Keyword::Compute)]
    #[regex(r"\$?count=?", |_| Keyword::Count)]
    #[regex(r"\$?expand=", |_| Keyword::Expand)]
    #[regex(r"\$?filter=", |_| Keyword::Filter)]
    #[regex(r"\$?orderby=", |_| Keyword::Orderby)]
    #[regex(r"\$?select=", |_| Keyword::Select)]
    #[regex(r"\$?skip=", |_| Keyword::Skip)]
    #[regex(r"\$?top=", |_| Keyword::Top)]
    Kw(Keyword),
    #[regex(r"\$?search=[^&]+", |l| {
        let s = l.slice();
        s.strip_prefix('$').unwrap_or(s).strip_prefix("search=").unwrap_or(s)
    })]
    #[regex(r"\$?search=(?&str)", |l| {
        let s = l.slice();
        s.strip_prefix('$').unwrap_or(s).strip_prefix("search=").unwrap_or(s).trim_matches('\'')
    })]
    #[regex(r"\$?search=(?&str2)", |l| {
        let s = l.slice();
        s.strip_prefix('$').unwrap_or(s).strip_prefix("search=").unwrap_or(s).trim_matches('\'').trim_matches('\'')
    })]
    Search(&'src str),
    #[regex(r"[A-Za-z_][A-Za-z0-9_]*")]
    Ident(&'src str),
    #[token("null")]
    Null,
    #[token("false", |_| false)]
    #[token("true", |_| true)]
    Bool(bool),
    #[regex(r"\d+", |l| l.slice().parse::<u128>().unwrap(), priority = 3)]
    Offset(u128),
    #[cfg(not(feature = "rust_decimal"))]
    #[regex(r"[+-]?([0-9]*[.])?[0-9]+", |l| l.slice().parse::<f64>().unwrap())]
    Num(f64),
    #[cfg(feature = "rust_decimal")]
    #[regex(r"[+-]?([0-9]*[.])?[0-9]+", |l| l.slice().parse::<rust_decimal::Decimal>().unwrap())]
    Num(rust_decimal::Decimal),
    #[regex(r"(?&str)", |l| l.slice().trim_matches('\''))]
    #[regex(r"(?&str2)", |l| l.slice().trim_matches('\'').trim_matches('\''))]
    Str(&'src str),
    #[cfg(feature = "jiff")]
    #[regex(r"(?&date)", |l| Date::Date(l.slice().parse::<jiff::civil::Date>().unwrap()))]
    #[regex(r"(?&ts)", |l| Date::Timestamp(l.slice().parse::<jiff::Timestamp>().unwrap()))]
    Date(Date),
    #[token("not ", |_| UnOp::Not)]
    #[token(" asc", |_| UnOp::Asc)]
    #[token(" desc", |_| UnOp::Desc)]
    UnOp(UnOp),
    #[token(" has ", |_| BinOp::Has)]
    #[token(" in ", |_| BinOp::In)]
    #[token(" mul ", |_| BinOp::Mul)]
    #[token(" div ", |_| BinOp::Div)]
    #[token(" divby ", |_| BinOp::Divby)]
    #[token(" mod ", |_| BinOp::Mod)]
    #[token(" add ", |_| BinOp::Add)]
    #[token(" sub ", |_| BinOp::Sub)]
    #[token(" gt ", |_| BinOp::Gt)]
    #[token(" ge ", |_| BinOp::Ge)]
    #[token(" lt ", |_| BinOp::Lt)]
    #[token(" le ", |_| BinOp::Le)]
    #[token(" eq ", |_| BinOp::Eq)]
    #[token(" ne ", |_| BinOp::Ne)]
    #[token(" and ", |_| BinOp::And)]
    #[token(" or ", |_| BinOp::Or)]
    BinOp(BinOp),
    #[token("&", |_| Ctrl::Delim)]
    #[token("(", |_| Ctrl::LParen)]
    #[token(")", |_| Ctrl::RParen)]
    #[token("[", |_| Ctrl::LBracket)]
    #[token("]", |_| Ctrl::RBracket)]
    #[token(",", |_| Ctrl::Comma)]
    #[token("*", |_| Ctrl::Wildcard)]
    #[token("/", |_| Ctrl::Slash)]
    #[token(";", |_| Ctrl::Semi)]
    Ctrl(Ctrl),
}

impl<'src> Token<'src> {
    fn errors_to_string(source: &'src str, errors: Vec<Rich<'_, Self>>) -> String {
        let mut buffer = Vec::new();
        for err in errors {
            Report::build(ReportKind::Error, ((), err.span().into_range()))
                .with_config(CONFIG)
                // .with_code(3)
                .with_message(err.to_string())
                .with_label(
                    Label::new(((), err.span().into_range()))
                        .with_message(err.reason().to_string())
                        .with_color(Color::Red),
                )
                .finish()
                .write(Source::from(source), &mut buffer)
                .unwrap();
        }
        String::from_utf8_lossy(&buffer).into_owned()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Display)]
#[display(rename_all = "lowercase")]
pub(crate) enum Keyword {
    Compute,
    Count,
    Expand,
    Filter,
    Orderby,
    Select,
    Skip,
    Top,
}

#[cfg(feature = "jiff")]
#[derive(Debug, Logos, Clone, Copy, PartialEq, PartialOrd, Display, From)]
pub enum Date {
    Date(jiff::civil::Date),
    Timestamp(jiff::Timestamp),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Display)]
#[display(rename_all = "lowercase")]
pub(crate) enum UnOp {
    Not,
    Asc,
    Desc,
}

/// [OData $filter operations](https://docs.oasis-open.org/odata/odata/v4.01/odata-v4.01-part1-protocol.html#sec_BuiltinFilterOperations).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Display)]
#[display(rename_all = "lowercase")]
pub enum BinOp {
    Has,

    In,

    Mul,
    Div,
    Divby,
    Mod,

    Add,
    Sub,

    Gt,
    Ge,
    Lt,
    Le,

    Eq,
    Ne,

    And,

    Or,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Display)]
pub(crate) enum Ctrl {
    #[display("&")]
    Delim,
    #[display("(")]
    LParen,
    #[display(")")]
    RParen,
    #[display("[")]
    LBracket,
    #[display("]")]
    RBracket,
    #[display(",")]
    Comma,
    #[display("*")]
    Wildcard,
    #[display("/")]
    Slash,
    #[display(";")]
    Semi,
}

pub(crate) trait TokenParser<'tokens, 'src: 'tokens>: Sized {
    fn parser<I: ValueInput<'tokens, Token = Token<'src>, Span = SimpleSpan>>()
    -> impl Parser<'tokens, I, Self, extra::Err<Rich<'tokens, Token<'src>>>> + Clone;

    fn try_from_str(value: &'src str) -> Result<Self, String> {
        let tokens = Token::lexer(value).spanned().map(|(t, s)| match t {
            Ok(t) => (t, s.into()),
            Err(()) => (Token::Error, s.into()),
        });
        let tokens =
            Stream::from_iter(tokens).map((0..value.len()).into(), |(t, s): (_, _)| (t, s));
        Self::parser()
            .parse(tokens)
            .into_result()
            .map_err(|e| Token::errors_to_string(value, e))
    }
}
