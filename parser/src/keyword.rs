use super::{left, map, match_literal, or, right, Parser, __};

#[derive(Debug, PartialEq)]
pub enum Arithmetics {
    Add,
    Sub,
    Mul,
    Div,
    Pow,

    Eq,
    Nq,
    Lt,
    Gt,
    Ltq,
    Gtq,
}

// Syntax

pub fn square_brackets<'a, A>(parser: impl Parser<'a, A>) -> impl Parser<'a, A> {
    right(match_literal("["), left(parser, __(match_literal("]"))))
}

pub fn brackets<'a, A>(parser: impl Parser<'a, A>) -> impl Parser<'a, A> {
    right(match_literal("("), left(parser, __(match_literal(")"))))
}

pub fn do_<'a>() -> impl Parser<'a, ()> {
    match_literal("do")
}

pub fn if_<'a>() -> impl Parser<'a, ()> {
    or(match_literal("if"), match_literal("?"))
}

pub fn lambda_<'a>() -> impl Parser<'a, ()> {
    or!(match_literal("lambda"), match_literal("λ"))
}

// Setters

pub fn def<'a>() -> impl Parser<'a, ()> {
    match_literal("def")
}

pub fn defn<'a>() -> impl Parser<'a, ()> {
    match_literal("defn")
}

pub fn arithetics<'a>() -> impl Parser<'a, Arithmetics> {
    or!(
        map(match_literal("+"), |_| Arithmetics::Add),
        map(match_literal("-"), |_| Arithmetics::Sub),
        map(match_literal("*"), |_| Arithmetics::Mul),
        map(match_literal("/"), |_| Arithmetics::Div),
        map(match_literal("^"), |_| Arithmetics::Pow),
        map(match_literal("="), |_| Arithmetics::Eq),
        map(match_literal("!="), |_| Arithmetics::Nq),
        map(match_literal("<"), |_| Arithmetics::Lt),
        map(match_literal(">"), |_| Arithmetics::Gt),
        map(match_literal(">="), |_| Arithmetics::Gtq),
        map(match_literal("<="), |_| Arithmetics::Ltq)
    )
}
