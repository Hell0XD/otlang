use std::collections::HashSet;

use parser::{Block, FunctionDef};

#[derive(Debug)]
pub struct FunctionAnalysis {
    functions: HashSet<String>,
}

impl FunctionAnalysis {
    pub fn new() -> FunctionAnalysis {
        FunctionAnalysis {
            functions: HashSet::new(),
        }
    }

    pub fn add_function(&mut self, name: &str) {
        self.functions.insert(name.to_owned());
    }

    pub fn exists(&self, name: &str) -> bool {
        self.functions.contains(name)
    }
}

pub fn analyse(blocks: &[Block], fanalysis: &mut FunctionAnalysis) {
    for block in blocks {
        match block {
            Block::FunctionDef(FunctionDef { name, types: _ }) => fanalysis.add_function(name.val),
            _ => (),
        }
    }
}
