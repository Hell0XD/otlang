use stack_jit::ShapeType;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Type {
    Int,
    Nil,
    Pair,
    Lambda,
}

impl Into<ShapeType> for Type {
    fn into(self) -> ShapeType {
        match self {
            Type::Int => ShapeType::Int,
            _ => unreachable!(),
        }
    }
}

pub struct TypeChecker {
    stack: Vec<Type>,
    locals: Vec<Type>,
}

impl TypeChecker {
    pub fn new() -> TypeChecker {
        TypeChecker {
            stack: Vec::new(),
            locals: Vec::new(),
        }
    }

    pub fn load_arguments(&mut self, args: &[Type]) {
        self.locals.extend(args);
    }

    pub fn get(&self, index: u8) -> Option<Type> {
        self.locals.get(index as usize).map(|&t| t)
    }

    pub fn push(&mut self, t: Type) {
        self.stack.push(t);
    }

    pub fn pop(&mut self) -> Option<Type> {
        self.stack.pop()
    }
}
