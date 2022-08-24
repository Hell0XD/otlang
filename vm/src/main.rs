use std::env::args;

use vm::deserialize;
use vm::VM;

fn main() {
    let mut args = args().skip(1);
    let path = args.next().unwrap();
    let program = std::fs::read(path).unwrap();

    let should_jit = if let Some(str) = args.next() {
        if str == "--jit" {
            true
        } else {
            false
        }
    } else {
        false
    };

    let debug_jit = if let Some(str) = args.next() {
        if str == "--debug-jit" {
            true
        } else {
            false
        }
    } else {
        false
    };

    let (start, constants, functions, signatures, groups) = deserialize(&program).unwrap();

    VM::new(
        constants,
        functions,
        signatures,
        groups,
        |s| print!("{}", s),
        should_jit,
        debug_jit,
    )
    .run(start, vec![]);
}
