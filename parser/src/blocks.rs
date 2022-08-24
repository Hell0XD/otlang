use crate::{
    error::expected_exp_or_val, identifier, lambda_, left, match_literal, non_recoverable,
    square_brackets, Identifier,
};

use super::{
    arithetics, brackets, def, defn, do_, if_, map, or, pair, right, value, var, zero_or_more,
    Arithmetics, Parser, Value, __,
};

#[derive(Debug, PartialEq)]
pub enum AritheticsOrCall<'a> {
    Arithmetics(Arithmetics),
    Call(Identifier<'a>),
    ExternCall(Identifier<'a>),
}

#[derive(Debug, PartialEq)]
pub enum Exp<'a> {
    Basic {
        op: AritheticsOrCall<'a>,
        args: Vec<ValOrExp<'a>>,
    },
    If {
        cond: Box<ValOrExp<'a>>,
        if_: Box<ValOrExp<'a>>,
        else_: Box<ValOrExp<'a>>,
    },
    Do {
        blocks: Vec<Block<'a>>,
    },
    Lambda {
        args: Vec<Identifier<'a>>,
        body: Box<ValOrExp<'a>>,
    },
}

#[derive(Debug, PartialEq)]
pub enum ValOrExp<'a> {
    Val(Value<'a>),
    Exp(Exp<'a>),
}

#[derive(Debug, PartialEq)]

pub struct VariableDef<'a> {
    pub name: Identifier<'a>,
    pub v_or_exp: ValOrExp<'a>,
}

#[derive(Debug, PartialEq)]
pub struct FunctionDef<'a> {
    pub name: Identifier<'a>,

    pub types: Vec<(Vec<Value<'a>>, ValOrExp<'a>)>,
}

impl<'a> FunctionDef<'a> {
    pub fn new(
        name: Identifier<'a>,
        types: Vec<(Vec<Value<'a>>, ValOrExp<'a>)>,
    ) -> FunctionDef<'a> {
        FunctionDef { name, types }
    }
}

#[derive(Debug, PartialEq)]
pub enum SymbolType {
    Int,
    Str,
}

#[derive(Debug, PartialEq)]
pub struct Symbol<'a> {
    pub name: Identifier<'a>,
    pub argt: Vec<SymbolType>,
    pub rett: SymbolType,
}

#[derive(Debug, PartialEq)]
pub struct Import<'a> {
    pub path: ValOrExp<'a>,

    pub symbols: Vec<Symbol<'a>>,
}

#[derive(Debug, PartialEq)]
pub enum Block<'a> {
    VariableDef(VariableDef<'a>),
    FunctionDef(FunctionDef<'a>),
    Expression(Exp<'a>),
    Import(Import<'a>),
}

pub fn block<'a>() -> impl Parser<'a, Block<'a>> {
    |ctx| {
        __(or!(
            map(function_def(), |fd| Block::FunctionDef(fd)),
            map(variable_def(), |vd| Block::VariableDef(vd)),
            map(import(), |import| Block::Import(import)),
            map(expression(), |exp| Block::Expression(exp))
        ))
        .parse(ctx)
    }
}

pub fn value_or_expression<'a>() -> impl Parser<'a, ValOrExp<'a>> {
    |ctx| {
        or(
            map(value(), |v| ValOrExp::Val(v)),
            map(expression(), |e| ValOrExp::Exp(e)),
        )
        .parse(ctx)
    }
}

// (do (def x 5) x)
fn do_exp<'a>() -> impl Parser<'a, Vec<Block<'a>>> {
    |ctx| brackets(right(__(do_()), zero_or_more(__(block())))).parse(ctx)
}

// (if cond true false)
fn if_exp<'a>() -> impl Parser<'a, (ValOrExp<'a>, (ValOrExp<'a>, ValOrExp<'a>))> {
    |ctx| {
        brackets(right(
            __(if_()),
            pair(
                __(value_or_expression()),
                pair(__(value_or_expression()), __(value_or_expression())),
            ),
        ))
        .parse(ctx)
    }
}

/*
   (match x
       0 (foo)
       1 (bar)
       > 5 & < 4 (fizz)
       _ (buzz)
   )
*/
// fn match_exp<'a>() -> impl Parser<'a, ()> {
//     |ctx| brackets()
// }

fn lambda_exp<'a>() -> impl Parser<'a, (Vec<Identifier<'a>>, ValOrExp<'a>)> {
    |ctx| {
        brackets(right(
            __(lambda_()),
            pair(
                __(square_brackets(zero_or_more(__(var())))),
                __(value_or_expression()),
            ),
        ))
        .parse(ctx)
    }
}

// (+ 5 5)
fn expression<'a>() -> impl Parser<'a, Exp<'a>> {
    |ctx| {
        or(
            map(lambda_exp(), |(args, body)| Exp::Lambda {
                args,
                body: Box::new(body),
            }),
            or(
                map(do_exp(), |blocks| Exp::Do { blocks }),
                or(
                    map(if_exp(), |(cond, (tr, fl))| Exp::If {
                        cond: Box::new(cond),
                        if_: Box::new(tr),
                        else_: Box::new(fl),
                    }),
                    map(
                        brackets(pair(
                            __(or!(
                                map(arithetics(), |a| AritheticsOrCall::Arithmetics(a)),
                                map(var(), |s| AritheticsOrCall::Call(s)),
                                map(right(match_literal("!"), var()), |s| {
                                    AritheticsOrCall::ExternCall(s)
                                })
                            )),
                            zero_or_more(__(value_or_expression())),
                        )),
                        |(op, args)| Exp::Basic { op, args },
                    ),
                ),
            ),
        )
        .parse(ctx)
    }
}

// int
fn symbol_type<'a>() -> impl Parser<'a, SymbolType> {
    or!(
        map(match_literal("int"), |_| SymbolType::Int),
        map(match_literal("str"), |_| SymbolType::Str)
    )
}

// (defn sum [int int] int)
fn symbol<'a>() -> impl Parser<'a, Symbol<'a>> {
    map(
        brackets(right(
            __(defn()),
            pair(
                __(identifier(|char| char.is_ascii_alphabetic())),
                pair(
                    __(square_brackets(zero_or_more(__(symbol_type())))),
                    __(symbol_type()),
                ),
            ),
        )),
        |(name, (argt, rett))| Symbol { name, argt, rett },
    )
}

// (import "./math.dll"
//      (defn sum (int int) int)
// )
pub fn import<'a>() -> impl Parser<'a, Import<'a>> {
    map(
        brackets(right(
            __(match_literal("import")),
            pair(__(value_or_expression()), zero_or_more(__(symbol()))),
        )),
        |(path, symbols)| Import { path, symbols },
    )
}

// (defn sum
//    (x y) (+ x y)
//    (x) x
// )
pub fn function_def<'a>() -> impl Parser<'a, FunctionDef<'a>> {
    map(
        brackets(right(
            __(defn()),
            non_recoverable(pair(
                __(var()),
                zero_or_more(__(pair(
                    square_brackets(zero_or_more(__(value()))),
                    __(value_or_expression()),
                ))),
            )),
        )),
        |(name, types)| FunctionDef::new(name, types),
    )
}

// (def x 69)
pub fn variable_def<'a>() -> impl Parser<'a, VariableDef<'a>> {
    map(
        right(
            match_literal("("),
            left(
                right(
                    __(def()),
                    non_recoverable(pair(
                        __(var()),
                        __(expected_exp_or_val(value_or_expression())),
                    )),
                ),
                non_recoverable(__(match_literal(")"))),
            ),
        ),
        |(name, val_e)| VariableDef {
            name,
            v_or_exp: val_e,
        },
    )
}
