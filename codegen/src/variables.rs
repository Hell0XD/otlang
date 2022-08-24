use std::collections::HashMap;

#[derive(Debug)]
pub struct Variables<'a> {
    vars: HashMap<(&'a str, u8), u8>,

    scope_counter: u8,
    local_counter: u8,

    highest_local: u8,

    lambda: bool,
    lambda_variables: Vec<&'a str>,
}

pub enum Variable {
    Local(u8),
    Array(u32),
}

impl<'a> Variables<'a> {
    pub fn new() -> Variables<'a> {
        Variables {
            vars: HashMap::new(),
            local_counter: 0,
            scope_counter: 0,

            highest_local: 0,
            lambda: false,
            lambda_variables: Vec::with_capacity(0),
        }
    }
    pub fn new_lambda() -> Variables<'a> {
        Variables {
            vars: HashMap::new(),
            local_counter: 1,
            scope_counter: 0,

            highest_local: 1,
            lambda: true,
            lambda_variables: Vec::new(),
        }
    }

    pub fn lambda_context_variables(&mut self) -> Vec<&'a str> {
        self.lambda_variables.drain(..).collect()
    }

    pub fn reset(&mut self) -> u8 {
        self.vars.clear();
        self.local_counter = 0;
        self.scope_counter = 0;
        let highest = self.highest_local;
        self.highest_local = 0;
        return highest;
    }

    pub fn scope(&mut self) {
        self.scope_counter += 1;
    }

    pub fn end_scope(&mut self) {
        let c = self.scope_counter;
        let mut removed = 0;
        self.vars.retain(|k, _| {
            if k.1 != c {
                true
            } else {
                removed += 1;
                false
            }
        });

        self.local_counter -= removed;
        self.scope_counter -= 1;
    }

    pub fn get(&mut self, name: &'a str) -> Option<Variable> {
        self.vars
            .get(&(name, self.scope_counter))
            .or_else(|| {
                let mut c = self.scope_counter;
                while c != 0 {
                    c -= 1;
                    let var = self.vars.get(&(name, c));
                    if var.is_some() {
                        return var;
                    }
                }
                None
            })
            .map(|&x| Variable::Local(x))
            .or_else(|| {
                if !self.lambda {
                    return None;
                }
                let index = self
                    .lambda_variables
                    .iter()
                    .position(|&var| var == name)
                    .map(|x| x as u32)
                    .unwrap_or_else(|| {
                        let tmp = self.lambda_variables.len() as u32;
                        self.lambda_variables.push(name);
                        tmp
                    });

                Some(Variable::Array(index))
            })
    }

    pub fn define_empty(&mut self) -> u8 {
        let local = self.local_counter;
        self.local_counter += 1;

        if self.local_counter > self.highest_local {
            self.highest_local = self.local_counter;
        }
        return local;
    }

    pub fn define_variable(&mut self, name: &'a str) -> Option<u8> {
        let local = self.local_counter;
        if self
            .vars
            .insert((name, self.scope_counter), local)
            .is_some()
        {
            return None;
        };
        self.local_counter += 1;

        if self.local_counter > self.highest_local {
            self.highest_local = self.local_counter;
        }

        return Some(local);
    }
}
