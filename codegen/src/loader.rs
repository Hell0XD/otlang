use std::collections::HashMap;

use error::Error;
use parser::{parse, Block, FileCxt, Parser};

use crate::compiler;
use crate::fanalysis::{analyse, FunctionAnalysis};
use crate::{serialize::serialize, CompilerCtx};

pub fn load_files(root: &str, debug: bool) -> Vec<u8> {
    let dir = std::fs::read_dir(root).expect("cant read directory");

    let res = dir
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.is_file())
        .filter(|path| path.extension().unwrap_or_default() == "ot")
        .map(|path| {
            (
                std::fs::read_to_string(&path).unwrap(),
                path.to_str().unwrap().to_owned(),
            )
        })
        .collect::<Vec<_>>();

    let mut modules: HashMap<&str, Vec<Block>> = res
        .iter()
        .map(|(file, path)| {
            parse()
                .parse(FileCxt::new(file, path))
                .map_err(|e| {
                    e.display();
                    unreachable!()
                })
                .unwrap()
        })
        .map(|(ctx, blocks)| (ctx.module, blocks))
        .collect();

    let mut fanalysis = FunctionAnalysis::new();
    modules
        .iter()
        .for_each(|(_, blocks)| analyse(blocks, &mut fanalysis));

    let mut ctx = CompilerCtx::new(fanalysis);

    let main = modules.remove("main").expect("Main module is missing");

    for block in main {
        compiler::block(&mut ctx, block, false);
    }
    ctx.flush_func_with_locals("@main", 0);

    modules
        .into_values()
        .try_for_each(|blocks| {
            blocks
                .into_iter()
                .map(|block| match &block {
                    Block::FunctionDef(_) => Some(block),
                    _ => None,
                })
                .try_for_each(|block| Some(compiler::block(&mut ctx, block?, false)))
        })
        .expect("Only main module can have top-level expressions");

    ctx.fill_labels();

    if debug {
        for (name, argc, index, localc, bytecode) in ctx.get_functions() {
            println!("{} - {}/{}[{}]", index, name, argc, localc - argc);
            bytecode.iter().for_each(|bc| println!("\t{:?}", bc));
        }
    }

    return serialize(ctx);
}
