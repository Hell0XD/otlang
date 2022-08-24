use std::collections::HashMap;

#[derive(Debug)]
pub struct Atoms<'a> {
    atoms: HashMap<&'a str, u32>,
    atomc: u32,
}

impl<'a> Atoms<'a> {
    pub fn new() -> Atoms<'a> {
        Atoms {
            atoms: HashMap::new(),
            atomc: 0,
        }
    }

    pub fn translate(&mut self, name: &'a str) -> u32 {
        self.atoms.get(name).map(|&x| x).unwrap_or_else(|| {
            let c = self.atomc;
            self.atoms.insert(name, c);
            self.atomc += 1;
            return c;
        })
    }
}
