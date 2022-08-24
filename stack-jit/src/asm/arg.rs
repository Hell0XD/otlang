use crate::{shape::ShapeType, sim::StackVal};

#[derive(Debug, PartialEq)]
pub enum Arg {
    Register(Register, Size),
    Memory(Register, i32),
    Imm(i32),
    Imm64(i64),

    Rel(i32),
}

impl From<(ShapeType, StackVal)> for Arg {
    fn from((shape, val): (ShapeType, StackVal)) -> Self {
        match val {
            StackVal::Register(reg) => Arg::Register(reg, shape.into()),
            StackVal::Stack(offset) => Arg::Memory(Register::RSP, offset),
            StackVal::Imm32(dw) => Arg::Imm(dw),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Register {
    RAX,
    RCX,
    RDX,
    RBX,
    RSP,
    RBP,
    RSI,
    RDI,

    R8,
    R9,
    R10,
    R11,
    R12,
    R13,
    R14,
    R15,
}

impl Into<u8> for Register {
    fn into(self) -> u8 {
        match self {
            Self::R8 => 0,
            Self::R9 => 1,
            Self::R10 => 2,
            Self::R11 => 3,
            Self::R12 => 4,
            Self::R13 => 5,
            Self::R14 => 6,
            Self::R15 => 7,
            _ => self as u8,
        }
    }
}

impl Register {
    pub fn is_extension(&self) -> bool {
        match self {
            Self::R8 => true,
            Self::R9 => true,
            Self::R10 => true,
            Self::R11 => true,
            Self::R12 => true,
            Self::R13 => true,
            Self::R14 => true,
            Self::R15 => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Size {
    DWord,
    QWord,
}

impl Into<Size> for ShapeType {
    fn into(self) -> Size {
        match self {
            ShapeType::Int => Size::DWord,
            ShapeType::Int64 => Size::QWord,
        }
    }
}
