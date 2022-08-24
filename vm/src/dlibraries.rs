use std::{collections::HashMap, ffi::OsStr};

use dl_loader::DynamicLibrary;

pub struct DynamicLibraries {
    holders: Vec<DynamicLibrary>,

    symbol_table: HashMap<String, *mut u8>,
}

impl DynamicLibraries {
    pub fn new() -> DynamicLibraries {
        DynamicLibraries {
            holders: Vec::new(),
            symbol_table: HashMap::new(),
        }
    }

    pub fn get_symbol(&self, name: &str) -> Option<*mut u8> {
        self.symbol_table.get(name).map(|f| *f)
    }

    pub fn load(&mut self, name: &str, symbols: Vec<String>) {
        let library = DynamicLibrary::open(OsStr::new(name)).unwrap();

        for symbol in symbols {
            let s = unsafe { library.symbol(&symbol).unwrap() };
            self.symbol_table.insert(symbol, s);
        }

        self.holders.push(library);
    }
}
