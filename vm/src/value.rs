use std::{
    cell::RefCell,
    ffi::CString,
    fmt::Display,
    ops::{Add, Div, Mul, Sub},
    rc::Rc,
};

use crate::vm_runtime_error;

#[cfg(not(target_arch = "wasm32"))]
use super::jit::typecheck::Type;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Value {
    Int(i32),
    Atom(u32),
    Nil,

    Pair(Rc<(Value, Value)>),
    Array(Rc<RefCell<Vec<Value>>>),
    String(CString),

    Lambda(Rc<(u32, Vec<Value>)>),
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Nil => write!(f, "nil"),
            Self::Int(int) => write!(f, "{}", int),
            Self::Atom(a) => write!(f, "atom({})", a),
            Self::String(s) => write!(f, "{}", s.to_string_lossy()),
            Self::Lambda(_) => write!(f, "lambda"),
            Self::Pair(p) => write!(f, "{{{};{}}}", p.0, p.1),
            Self::Array(a) => write!(f, "{:?}", a.borrow()),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl Into<i64> for Value {
    fn into(self) -> i64 {
        match self {
            Self::Int(i) => i as i64,
            Self::Pair(p) => unsafe { std::mem::transmute(p) },
            _ => unreachable!(),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl From<(Type, i64)> for Value {
    fn from((s, v): (Type, i64)) -> Self {
        match s {
            Type::Int => Value::Int(v as i32),
            Type::Pair => Value::Pair(unsafe { std::mem::transmute(v) }),
            _ => unreachable!(),
        }
    }
}

impl Value {
    pub fn ffi(&self) -> i64 {
        match self {
            Value::Int(i) => *i as i64,
            Value::String(s) => s.as_bytes_with_nul().as_ptr() as i64,
            _ => unimplemented!(),
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn typ(&self) -> Type {
        match self {
            Value::Int(_) => Type::Int,
            Value::Nil => Type::Nil,
            Value::Pair(_) => Type::Pair,
            Value::Lambda(_) => Type::Lambda,
            Value::Atom(_) => Type::Int,
            Value::String(_) => Type::Int,
            Value::Array(_) => Type::Nil,
        }
    }
}

impl Add for Value {
    type Output = Value;
    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Int(i), Value::Int(i2)) => Value::Int(i + i2),
            (s, r) => vm_runtime_error!("Can't add {:?} and {:?}", s, r),
        }
    }
}

impl Sub for Value {
    type Output = Value;
    fn sub(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Int(i), Value::Int(i2)) => Value::Int(i - i2),
            (s, r) => vm_runtime_error!("Can't sub {:?} and {:?}", s, r),
        }
    }
}

impl Mul for Value {
    type Output = Value;
    fn mul(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Int(i), Value::Int(i2)) => Value::Int(i * i2),
            (s, r) => vm_runtime_error!("Can't mul {:?} and {:?}", s, r),
        }
    }
}

impl Div for Value {
    type Output = Value;
    fn div(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Int(i), Value::Int(i2)) => Value::Int(i / i2),
            (s, r) => vm_runtime_error!("Can't mul {:?} and {:?}", s, r),
        }
    }
}
