use stack_jit::Bytecode as Bc;
use stack_jit::*;

use crate::value::Value;
use vm_bytecode::Bytecode;

use self::typecheck::{Type, TypeChecker};

pub mod typecheck;

pub struct JITHandle {
    jit: JIT,

    fstats: Vec<FunctionShape>,
}

#[derive(Debug, Clone)]
pub struct FunctionShape {
    callc: u8,

    pub args: Option<Vec<Type>>,
    pub ret: Type,
}

impl JITHandle {
    pub fn new(fc: usize) -> JITHandle {
        JITHandle {
            jit: JIT::new(),

            fstats: vec![
                FunctionShape {
                    callc: 0,

                    args: None,
                    ret: Type::Nil
                };
                fc
            ],
        }
    }

    pub fn write_stats(&mut self, index: usize, args: Vec<Type>, ret: Type) {
        let shape = &mut self.fstats[index];
        if shape.args.is_none() || shape.ret != ret || shape.args.as_ref() != Some(&args) {
            shape.callc = 0;
            shape.args = Some(args);
            shape.ret = ret;
        }
        shape.callc += 1;
    }

    pub fn get_shape(&self, index: usize) -> FunctionShape {
        self.fstats[index].clone()
    }

    pub fn is_function_hot(&self, index: usize, hotc: u8) -> bool {
        self.fstats[index].callc >= hotc
    }

    pub fn compile(
        &mut self,
        index: usize,
        args: Vec<Type>,
        localc: u8,
        ret: Type,
        original: &[Bytecode],
        constants: &[Value],
        debug_jit: bool,
    ) {
        translate(args, localc, ret, original, constants).map(|f| {
            if debug_jit {
                println!("{:?}", f);
            }
            compile(&mut self.jit, index as u32, f).unwrap()
        });
    }

    pub fn function_exist(&self, index: u32) -> bool {
        self.jit.function_exist(index)
    }

    pub fn call_function(&self, index: u32, args: &[i64]) -> Option<i64> {
        self.jit.call(index, args)
    }
}

pub fn translate(
    args: Vec<Type>,
    _localc: u8,
    ret: Type,
    original: &[Bytecode],
    constants: &[Value],
) -> Option<Function> {
    let mut code: Vec<Bc> = Vec::with_capacity(original.len());

    let mut typecheck = TypeChecker::new();
    typecheck.load_arguments(&args);

    for bytecode in original {
        let b = match bytecode {
            &Bytecode::LocalGet(i) => {
                typecheck.push(typecheck.get(i).unwrap());
                Bc::LocalGet(i)
            }
            Bytecode::LocalSet(_) => return None,
            &Bytecode::ConstantGet(i) => match constants[i as usize] {
                Value::Int(i) => Bc::ConstI32(i),
                _ => return None,
            },
            Bytecode::Add => {
                let first = typecheck.pop().unwrap();
                let second = typecheck.pop().unwrap();

                if first != Type::Int || second != Type::Int {
                    return None;
                }

                typecheck.push(first);
                Bc::Add
            }
            Bytecode::Sub => todo!(),
            Bytecode::Mul => todo!(),
            Bytecode::Div => todo!(),
            Bytecode::Eq => todo!(),
            Bytecode::Lt => todo!(),
            Bytecode::Gt => todo!(),
            Bytecode::Lte => todo!(),
            Bytecode::Gte => todo!(),
            Bytecode::If => Bc::IfExp,
            Bytecode::Else => Bc::Else,
            Bytecode::End => Bc::EndIf,
            Bytecode::Ret => Bc::Ret,
            _ => return None,
        };
        code.push(b);
    }

    Some(Function::new(
        args.into_iter().map(|arg| arg.into()).collect(),
        vec![],
        ret.into(),
        code,
    ))
}
