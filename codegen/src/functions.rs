use std::collections::HashMap;

use vm_bytecode::Bytecode;

#[derive(Debug)]
pub struct Functions {
    funcs: HashMap<(String, u8), (u32, u8, Vec<Bytecode>)>,
    fc: u32,
    main_function: Option<u32>,
}

impl Functions {
    pub fn new() -> Functions {
        Functions {
            funcs: HashMap::new(),
            fc: 0,
            main_function: None,
        }
    }

    pub fn get_functions(&self) -> impl Iterator<Item = (&String, &u8, &u32, &u8, &Vec<Bytecode>)> {
        self.funcs
            .iter()
            .map(|((name, argc), (index, localc, b))| (name, argc, index, localc, b))
    }

    pub fn main_index(&self) -> Option<u32> {
        self.main_function
    }

    pub fn get_all(self) -> Vec<(u32, Vec<Bytecode>, u8, u8)> {
        let mut vec = self
            .funcs
            .into_iter()
            .map(|((_, argc), (index, localc, b))| (index, b, argc, localc))
            .collect::<Vec<(u32, Vec<Bytecode>, u8, u8)>>();
        vec.sort_by_key(|(index, _, _, _)| *index);

        vec
    }

    pub fn get_code(&mut self, name: impl Into<String>, argc: u8) -> Option<&mut Vec<Bytecode>> {
        self.funcs.get_mut(&(name.into(), argc)).map(|(_, _, b)| b)
    }

    pub fn get(&mut self, name: impl Into<String>, argc: u8) -> Option<u32> {
        self.funcs.get(&(name.into(), argc)).map(|(i, _, _)| *i)
    }

    pub fn define(
        &mut self,
        name: impl Into<String>,
        code: Vec<Bytecode>,
        argc: u8,
        localc: u8,
    ) -> u32 {
        let name: String = name.into();
        if name == "@main" {
            self.main_function = Some(self.fc);
        }
        self.funcs.insert((name, argc), (self.fc, localc, code));
        let fc = self.fc;
        self.fc += 1;
        return fc;
    }
}
