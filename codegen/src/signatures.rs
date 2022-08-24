use std::collections::HashMap;

use parser::SymbolType;

#[derive(Debug)]
pub struct Signatures<'a> {
    pub signatures: HashMap<(&'a str, u8), (u32, Vec<SymbolType>, SymbolType)>, // index
    signaturec: u32,

    groups: Vec<Vec<u32>>,
}

impl<'a> Signatures<'a> {
    pub fn new() -> Signatures<'a> {
        Signatures {
            signatures: HashMap::new(),
            signaturec: 0,

            groups: Vec::new(),
        }
    }

    pub fn get_signatures_and_groups(
        self,
    ) -> (
        Vec<(&'a str, u32, Vec<SymbolType>, SymbolType)>,
        Vec<Vec<u32>>,
    ) {
        let sig = self.signatures;

        let mut tmp = sig
            .into_iter()
            .map(|((name, _argc), (index, args, ret))| (name, index, args, ret))
            .collect::<Vec<(&'a str, u32, Vec<SymbolType>, SymbolType)>>();
        tmp.sort_by(|(_, x, _, _), (_, y, _, _)| x.cmp(&y));

        return (tmp, self.groups);
    }

    pub fn new_group(&mut self) -> u32 {
        self.groups.push(Vec::new());
        return (self.groups.len() - 1) as u32;
    }

    pub fn define(&mut self, name: &'a str, args: Vec<SymbolType>, rett: SymbolType) {
        self.signatures
            .insert((name, args.len() as u8), (self.signaturec, args, rett));

        self.groups.last_mut().unwrap().push(self.signaturec);

        self.signaturec += 1;
    }

    pub fn get(&self, name: &'a str, argc: u8) -> Option<u32> {
        self.signatures.get(&(name, argc)).map(|x| x.0)
    }
}
