use crate::Identifier;

use super::{identifier, left, map, match_literal, maybe, or, pair, right, Parser};

#[derive(Debug, PartialEq)]
pub enum Value<'a> {
    Int(i32),
    String(Identifier<'a>),
    Atom(Identifier<'a>),
    Var(Identifier<'a>),
    Null,
}

fn number<'a>() -> impl Parser<'a, i32> {
    map(identifier(|char| char.is_numeric()), |s| {
        s.val.parse().unwrap()
    })
}

pub fn string<'a>() -> impl Parser<'a, Identifier<'a>> {
    right(
        match_literal("\""),
        left(identifier(|char| char != '"'), match_literal("\"")),
    )
}

const HEX_CHARS: [char; 12] = ['a', 'b', 'c', 'd', 'e', 'f', 'A', 'B', 'C', 'D', 'E', 'F'];

fn hex<'a>() -> impl Parser<'a, i32> {
    map(
        right(
            match_literal("0x"),
            identifier(|char| char.is_numeric() || HEX_CHARS.contains(&char)),
        ),
        |s| i32::from_str_radix(s.val, 16).unwrap(),
    )
}

fn binary<'a>() -> impl Parser<'a, i32> {
    map(
        right(
            match_literal("0b"),
            identifier(|char| char == '1' || char == '0'),
        ),
        |s| i32::from_str_radix(s.val, 2).unwrap(),
    )
}

fn char<'a>() -> impl Parser<'a, i32> {
    map(
        right(
            match_literal("'"),
            left(identifier(|ch| ch != '\''), match_literal("'")),
        ),
        |s| s.val.chars().next().unwrap() as i32,
    )
}

fn int<'a>() -> impl Parser<'a, i32> {
    map(
        pair(
            maybe(map(match_literal("-"), |_| -1), || 1),
            or!(hex(), binary(), number(), char()),
        ),
        |(negation, n)| n * negation,
    )
}

fn atom<'a>() -> impl Parser<'a, Identifier<'a>> {
    right(match_literal(":"), var())
}

fn null<'a>() -> impl Parser<'a, ()> {
    or(match_literal("null"), match_literal("!"))
}

pub fn var<'a>() -> impl Parser<'a, Identifier<'a>> {
    identifier(|c| c.is_ascii_alphabetic() || c == '_')
}

pub fn value<'a>() -> impl Parser<'a, Value<'a>> {
    or!(
        map(null(), |_| Value::Null),
        map(string(), |s| Value::String(s)),
        map(int(), |n| Value::Int(n)),
        map(atom(), |a| Value::Atom(a)),
        map(var(), |s| Value::Var(s))
    )
}
