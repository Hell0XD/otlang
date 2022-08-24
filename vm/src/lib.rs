mod deserialize;
mod error;

#[cfg(not(target_arch = "wasm32"))]
mod dlibraries;
#[cfg(not(target_arch = "wasm32"))]
mod jit;

mod value;
mod vm;

mod helper;

pub use deserialize::*;
pub use vm::*;

pub fn run_bytecode(bc: &[u8], console: impl Fn(&str)) {
    let (start, constants, functions, signatures, groups) = deserialize(bc).unwrap();

    VM::new(
        constants, functions, signatures, groups, console, false, false,
    )
    .run(start, vec![]);
}
