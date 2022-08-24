use std::fmt::Display;

pub struct Shape {
    pub args: Vec<ShapeType>,
}

impl Shape {
    pub fn new(args: Vec<ShapeType>) -> Shape {
        Shape { args }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ShapeType {
    Int,
    Int64,
}

impl Display for ShapeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Int => "int",
            Self::Int64 => "i64",
        };
        write!(f, "{}", s)
    }
}
