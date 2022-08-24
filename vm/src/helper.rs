use vm_bytecode::Bytecode;

pub trait IsIf {
    fn is_if(&self) -> bool;
}

impl IsIf for Bytecode {
    fn is_if(&self) -> bool {
        use Bytecode::*;
        match self {
            If | IfGt | IfEq | IfLt | IfGte | IfLte => true,
            _ => false,
        }
    }
}
