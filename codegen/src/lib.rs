mod atoms;
pub mod cli;
pub mod compiler;
mod error;
pub mod fanalysis;
mod functions;
pub mod loader;
pub mod serialize;
mod signatures;
mod util;
mod variables;

use ::error::Error;
use fanalysis::{analyse, FunctionAnalysis};
pub use parser;

use compiler::{CompilerCtx, Constant};
use parser::{parse, FileCxt, Parser};
use serialize::serialize;

pub fn compile_file(contents: &str, error_handle: impl Fn(&str)) -> Vec<u8> {
    let ast = match parse()
        .parse(FileCxt::new(contents, "main.ot"))
        .map_err(|e| e.get_formated_with_file(contents))
    {
        Ok((_, ast)) => ast,
        Err(e) => {
            error_handle(&e);
            return Vec::with_capacity(0);
        }
    };

    let mut fanalysis = FunctionAnalysis::new();
    analyse(&ast, &mut fanalysis);

    let mut ctx = CompilerCtx::new(fanalysis);

    for block in ast {
        compiler::block(&mut ctx, block, false);
    }

    ctx.flush_func_with_locals("@main", 0);
    ctx.fill_labels();

    return serialize(ctx);
}
