use std::collections::HashMap;

use error::Error;
use parser::{
    AritheticsOrCall, Arithmetics, Block, Exp, FunctionDef, Identifier, Import, Symbol, ValOrExp,
    Value, VariableDef,
};

use crate::{
    error::CompilerError, fanalysis::FunctionAnalysis, signatures::Signatures, util::escape_string,
    variables::Variable,
};
use vm_bytecode::Bytecode;

use crate::{atoms::Atoms, functions::Functions, variables::Variables};

macro_rules! ignore {
    ($i:expr) => {{
        $i;
        unreachable!();
    }};
}

macro_rules! raise_compiler_error {
    ($($t: tt)+) => {
        {
            println!("\x1b[31m\x1b[1mCOMPILER ERROR:\x1b[0m {}", format!($($t)+));
            std::process::exit(0);
        }
    };
}

#[derive(Debug, PartialEq, Clone)]
pub enum Constant {
    Int(i32),
    String(String),
}

#[derive(Debug)]
pub struct Label<'a> {
    name: Identifier<'a>,
    arity: u8,
    index: usize,
}

#[derive(Debug)]
pub struct CompilerCtx<'a> {
    vars: Variables<'a>,

    pub constants: Vec<Constant>,
    pub functions: Functions,
    pub signatures: Signatures<'a>,

    fanalysis: FunctionAnalysis,

    current_function: Vec<Bytecode>,

    atoms: Atoms<'a>,

    labels: Vec<((String, u8), Label<'a>)>,
    current_labels: Vec<Label<'a>>,

    lambdac: u32,
}

impl<'a> CompilerCtx<'a> {
    pub fn new(fanalysis: FunctionAnalysis) -> CompilerCtx<'a> {
        CompilerCtx {
            vars: Variables::new(),
            constants: Vec::new(),
            functions: Functions::new(),
            signatures: Signatures::new(),

            fanalysis,

            current_function: Vec::new(),

            atoms: Atoms::new(),

            labels: Vec::new(),
            current_labels: Vec::new(),

            lambdac: 0,
        }
    }

    pub fn get_functions(&self) -> impl Iterator<Item = (&String, &u8, &u32, &u8, &Vec<Bytecode>)> {
        self.functions.get_functions()
    }

    pub fn swap_variables(&mut self, new_vars: &mut Variables<'a>) {
        std::mem::swap(&mut self.vars, new_vars);
    }
    pub fn swap_func(&mut self, new_func: &mut Vec<Bytecode>) {
        std::mem::swap(&mut self.current_function, new_func);
    }
    pub fn swap_labels(&mut self, new_labels: &mut Vec<Label<'a>>) {
        std::mem::swap(&mut self.current_labels, new_labels);
    }

    pub fn fill_labels(&mut self) {
        std::mem::take(&mut self.labels)
            .into_iter()
            .for_each(|((fname, fargc), label)| {
                if let Some(f) = self.functions.get(label.name.val, label.arity) {
                    let code = self.functions.get_code(fname, fargc).unwrap();
                    match code[label.index] {
                        Bytecode::Call(_) => code[label.index] = Bytecode::Call(f),
                        Bytecode::CallRet(_) => code[label.index] = Bytecode::CallRet(f),
                        // Can't be reached because label can't refer to non-call bytecode
                        _ => unreachable!(),
                    }
                } else {
                    ignore!(CompilerError::new(
                        format!("Unknown function `{}`", label.name.val),
                        &label.name.file,
                        label.name.line,
                        label.name.start,
                    )
                    .display())
                }
            });
    }

    pub fn lambdac_increase(&mut self) -> u32 {
        let curr = self.lambdac;
        self.lambdac += 1;
        return curr;
    }

    pub fn flush_func(&mut self, name: impl Into<String>, argc: u8, localc: u8) {
        let mut f = Vec::new();
        std::mem::swap(&mut self.current_function, &mut f);

        let name = name.into();
        self.functions.define(name.clone(), f, argc, localc);
        self.labels.extend(
            self.current_labels
                .drain(..)
                .map(|label| ((name.clone(), argc), label)),
        );
    }

    pub fn flush_func_with_locals(&mut self, name: impl Into<String>, argc: u8) {
        let mut f = Vec::new();
        std::mem::swap(&mut self.current_function, &mut f);

        let name = name.into();
        self.functions
            .define(name.clone(), f, argc, self.vars.reset());

        self.labels.extend(
            self.current_labels
                .drain(..)
                .map(|label| ((name.clone(), argc), label)),
        );
    }
}

pub fn block<'a>(ctx: &mut CompilerCtx<'a>, bl: Block<'a>, is_tail: bool) {
    match bl {
        Block::Import(Import { path, symbols }) => {
            val_or_expression(ctx, path, false);

            let group = ctx.signatures.new_group();

            for Symbol { name, argt, rett } in symbols {
                ctx.signatures.define(name.val, argt, rett);
            }

            ctx.current_function.push(Bytecode::Import(group));
        }
        Block::VariableDef(VariableDef { name, v_or_exp }) => {
            val_or_expression(ctx, v_or_exp, false);
            ctx.current_function.push(Bytecode::LocalSet(
                ctx.vars.define_variable(name.val).unwrap_or_else(|| {
                    ignore!(CompilerError::new(
                        format!("Variable with name `{}` already exists", name.val),
                        name.file,
                        name.line,
                        name.start,
                    )
                    .display())
                }),
            ));
        }
        Block::FunctionDef(FunctionDef { name, types }) => {
            let mut saved_vars = Variables::new();
            let mut saved_func = Vec::new();
            let mut saved_labels = Vec::new();
            ctx.swap_variables(&mut saved_vars);
            ctx.swap_func(&mut saved_func);
            ctx.swap_labels(&mut saved_labels);

            // Own group by
            let mut groups: HashMap<usize, Vec<(Vec<Value>, ValOrExp)>> = HashMap::new();
            types
                .into_iter()
                .for_each(|(args, body)| groups.entry(args.len()).or_default().push((args, body)));

            for (argc, group) in groups {
                let mut highest_local = 0;
                for (args, body) in group {
                    let labelc = resolve_args(ctx, args);
                    val_or_expression(ctx, body, true);
                    match *ctx.current_function.last().unwrap() {
                        Bytecode::Call(dw) => {
                            ctx.current_function.pop();
                            ctx.current_function.push(Bytecode::CallRet(dw));
                        }
                        _ => ctx.current_function.push(Bytecode::Ret),
                    }
                    (0..labelc).for_each(|_| {
                        ctx.current_function.push(Bytecode::Else);
                        ctx.current_function.push(Bytecode::End);
                    });
                    let lc = ctx.vars.reset();
                    if highest_local < lc {
                        highest_local = lc;
                    }
                }
                ctx.flush_func(name.val, argc as u8, highest_local);
            }

            ctx.swap_variables(&mut saved_vars);
            ctx.swap_func(&mut saved_func);
            ctx.swap_labels(&mut saved_labels);
        }
        Block::Expression(exp) => {
            expression(ctx, exp, is_tail);
            ctx.current_function.push(Bytecode::Remove);
        }
    }
}

fn resolve_args<'a>(ctx: &mut CompilerCtx<'a>, args: Vec<Value<'a>>) -> u32 {
    let mut labelc: u32 = 0;
    args.into_iter().for_each(|arg| match arg {
        Value::Var(name) => {
            ctx.vars.define_variable(name.val).unwrap_or_else(|| {
                ignore!(CompilerError::new(
                    format!("Argument `{}` already exists", name.val),
                    name.file,
                    name.line,
                    name.start,
                )
                .display())
            });
        }
        val => {
            val_or_expression(ctx, ValOrExp::Val(val), false);
            ctx.current_function
                .push(Bytecode::LocalGet(ctx.vars.define_empty()));
            ctx.current_function.push(Bytecode::IfEq);
            labelc += 1;
        }
    });
    return labelc;
}

fn val_or_expression<'a>(ctx: &mut CompilerCtx<'a>, v_exp: ValOrExp<'a>, is_tail: bool) {
    match v_exp {
        ValOrExp::Val(v) => match v {
            Value::Var(name) => ctx.current_function.push(
                match ctx.vars.get(name.val).unwrap_or_else(|| {
                    ignore!(CompilerError::new(
                        format!("Variable with name `{}` does not exist", name.val),
                        name.file,
                        name.line,
                        name.start,
                    )
                    .display())
                }) {
                    Variable::Local(l) => Bytecode::LocalGet(l),
                    Variable::Array(a) => Bytecode::LocalArrayGet(0, a),
                },
            ),
            Value::Int(dw) => {
                let local = ctx
                    .constants
                    .iter()
                    .position(|cons| cons == &Constant::Int(dw))
                    .map(|p| p as u8)
                    .unwrap_or_else(|| {
                        ctx.constants.push(Constant::Int(dw));
                        (ctx.constants.len() - 1) as u8
                    });
                ctx.current_function.push(Bytecode::ConstantGet(local));
            }
            Value::Null => ctx.current_function.push(Bytecode::ConstantNil),
            Value::String(s) => {
                ctx.constants.push(Constant::String(escape_string(s.val)));
                ctx.current_function
                    .push(Bytecode::ConstantGet((ctx.constants.len() - 1) as u8))
            }
            Value::Atom(atom) => {
                ctx.current_function
                    .push(Bytecode::Atom(ctx.atoms.translate(atom.val)));
            }
        },
        ValOrExp::Exp(exp) => expression(ctx, exp, is_tail),
    }
}

fn expression<'a>(ctx: &mut CompilerCtx<'a>, exp: Exp<'a>, is_tail: bool) {
    match exp {
        Exp::Basic { op, args } => match op {
            AritheticsOrCall::ExternCall(name) => {
                let argc = args.len() as u8;
                args.into_iter()
                    .for_each(|arg| val_or_expression(ctx, arg, false));

                ctx.current_function.push(Bytecode::CallDynamic(
                    ctx.signatures
                        .get(name.val, argc)
                        .expect(&format!("Unknown dynamic library call `{}`", name.val)),
                ));
            }
            AritheticsOrCall::Arithmetics(a) => arithetics(ctx, a, args),
            AritheticsOrCall::Call(name) => {
                let argc = args.len() as u8;
                args.into_iter()
                    .for_each(|arg| val_or_expression(ctx, arg, false));

                match name.val {
                    "pair" => {
                        if argc != 2 {
                            CompilerError::new(
                                format!("pair/2 expects 2 arguments, found {}", argc),
                                name.file,
                                name.line,
                                name.start,
                            )
                            .display();
                        }
                        ctx.current_function.push(Bytecode::NewPair);
                        return;
                    }
                    "left" => {
                        if argc != 1 {
                            CompilerError::new(
                                format!("left/1 expects 1 argument, found {}", argc),
                                name.file,
                                name.line,
                                name.start,
                            )
                            .display();
                        }
                        ctx.current_function.push(Bytecode::PairLeft);
                        return;
                    }
                    "right" => {
                        if argc != 1 {
                            CompilerError::new(
                                format!("right/1 expects 1 argument, found {}", argc),
                                name.file,
                                name.line,
                                name.start,
                            )
                            .display();
                        }
                        ctx.current_function.push(Bytecode::PairRight);
                        return;
                    }
                    "print" => {
                        if argc != 1 {
                            CompilerError::new(
                                format!("print/1 expects 1 argument, found {}", argc),
                                name.file,
                                name.line,
                                name.start,
                            )
                            .display();
                        }
                        ctx.current_function.push(Bytecode::Print);
                        return;
                    }
                    "new" => {
                        if argc != 1 {
                            CompilerError::new(
                                format!("new/1 expects 1 argument, found {}", argc),
                                name.file,
                                name.line,
                                name.start,
                            )
                            .display();
                        }
                        ctx.current_function.push(Bytecode::NewArrayDyn);
                        return;
                    }
                    "get" => {
                        if argc != 2 {
                            CompilerError::new(
                                format!("get/2 expects 2 arguments, found {}", argc),
                                name.file,
                                name.line,
                                name.start,
                            )
                            .display();
                        }
                        ctx.current_function.push(Bytecode::ArrayGet);
                        return;
                    }
                    "set" => {
                        if argc != 3 {
                            CompilerError::new(
                                format!("set/3 expects 3 arguments, found {}", argc),
                                name.file,
                                name.line,
                                name.start,
                            )
                            .display();
                        }
                        ctx.current_function.push(Bytecode::ArraySet);
                        return;
                    }
                    "len" => {
                        if argc != 1 {
                            CompilerError::new(
                                format!("len/1 expects 1 argument, found {}", argc),
                                name.file,
                                name.line,
                                name.start,
                            )
                            .display();
                        }
                        ctx.current_function.push(Bytecode::ArrayLen);
                        return;
                    }
                    "strlen" => {
                        if argc != 1 {
                            CompilerError::new(
                                format!("strlen/1 expects 1 argument, found {}", argc),
                                name.file,
                                name.line,
                                name.start,
                            )
                            .display();
                        }
                        ctx.current_function.push(Bytecode::StringLen);
                        return;
                    }
                    _ if !ctx.fanalysis.exists(name.val) => {
                        if let Some(var) = ctx.vars.get(name.val) {
                            match var {
                                Variable::Local(l) => {
                                    ctx.current_function.push(Bytecode::LocalGet(l))
                                }
                                Variable::Array(a) => {
                                    ctx.current_function.push(Bytecode::LocalArrayGet(0, a))
                                }
                            }

                            ctx.current_function.push(Bytecode::CallLambda);
                        } else {
                            CompilerError::new(
                                format!("Function with name {} does not exists", name.val),
                                name.file,
                                name.line,
                                name.start,
                            )
                            .display();
                        }
                    }
                    _ => {
                        let index = ctx.functions.get(name.val, argc).unwrap_or_else(|| {
                            ctx.current_labels.push(Label {
                                name,
                                arity: argc,
                                index: ctx.current_function.len(),
                            });
                            0
                        });

                        ctx.current_function.push(if is_tail {
                            Bytecode::CallRet(index)
                        } else {
                            Bytecode::Call(index)
                        });
                    }
                }
            }
        },
        Exp::If { cond, if_, else_ } => {
            match *cond {
                ValOrExp::Exp(Exp::Basic {
                    op: AritheticsOrCall::Arithmetics(Arithmetics::Eq),
                    args,
                }) => {
                    let mut args = args.into_iter();
                    val_or_expression(ctx, args.next().unwrap(), false);
                    val_or_expression(ctx, args.next().unwrap(), false);
                    ctx.current_function.push(Bytecode::IfEq);
                }
                cond => {
                    val_or_expression(ctx, cond, false);
                    ctx.current_function.push(Bytecode::If);
                }
            }
            ctx.vars.scope();
            val_or_expression(ctx, *if_, true);
            ctx.current_function.push(Bytecode::Else);
            ctx.vars.end_scope();
            ctx.vars.scope();
            val_or_expression(ctx, *else_, true);
            ctx.current_function.push(Bytecode::End);
            ctx.vars.end_scope();
        }
        Exp::Do { mut blocks } => {
            ctx.vars.scope();
            let last = blocks.pop().expect("Atleast one block thingy");
            blocks.into_iter().for_each(|bl| block(ctx, bl, false));
            block(ctx, last, is_tail);
            match ctx.current_function.pop() {
                None | Some(Bytecode::Remove) => (),
                Some(bc) => ctx.current_function.push(bc),
            }
            ctx.vars.end_scope();
        }
        Exp::Lambda { args, body } => {
            let mut saved_vars = Variables::new_lambda();
            let mut saved_func = Vec::new();
            let mut saved_labels = Vec::new();
            ctx.swap_variables(&mut saved_vars);
            ctx.swap_func(&mut saved_func);
            ctx.swap_labels(&mut saved_labels);

            let argc = args.len() as u8;

            resolve_args(ctx, args.into_iter().map(|s| Value::Var(s)).collect());
            val_or_expression(ctx, *body, true);
            ctx.current_function.push(Bytecode::Ret);

            let localc = ctx.vars.reset();

            let name = format!("@lambda{}", ctx.lambdac_increase());
            ctx.flush_func(&name, argc, localc);

            ctx.swap_func(&mut saved_func);
            ctx.swap_labels(&mut saved_labels);

            let ctx_vars = ctx.vars.lambda_context_variables();
            let ctx_varc = ctx_vars.len() as u8;

            ctx.swap_variables(&mut saved_vars);

            for name in ctx_vars {
                ctx.current_function.push(
                    match ctx.vars.get(name).unwrap_or_else(|| {
                        raise_compiler_error!("Unknown variable in lambda `{}`", name)
                    }) {
                        Variable::Local(l) => Bytecode::LocalGet(l),
                        Variable::Array(a) => Bytecode::LocalArrayGet(0, a),
                    },
                );
            }

            ctx.current_function.push(Bytecode::NewLambda(
                ctx.functions.get(name, argc).unwrap(),
                ctx_varc,
            ));
        }
    }
}

fn arithetics<'a>(ctx: &mut CompilerCtx<'a>, a: Arithmetics, args: Vec<ValOrExp<'a>>) {
    let mut args = args.into_iter();
    val_or_expression(
        ctx,
        args.next()
            .unwrap_or_else(|| raise_compiler_error!("Arithetics expects at least 1 argument")),
        false,
    );

    args.for_each(|arg| {
        val_or_expression(ctx, arg, false);
        ctx.current_function.push(match a {
            Arithmetics::Add => Bytecode::Add,
            Arithmetics::Sub => Bytecode::Sub,
            Arithmetics::Div => Bytecode::Div,
            Arithmetics::Mul => Bytecode::Mul,
            Arithmetics::Pow => todo!(),

            Arithmetics::Eq => Bytecode::Eq,
            Arithmetics::Nq => todo!(),
            Arithmetics::Lt => Bytecode::Lt,
            Arithmetics::Gt => Bytecode::Gt,
            Arithmetics::Ltq => Bytecode::Lte,
            Arithmetics::Gtq => Bytecode::Gte,
        });
    });
}
