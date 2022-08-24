use parser::SymbolType;
use vm_bytecode::Bytecode;

use super::{CompilerCtx, Constant};

enum Type {
    Int,
    String,
}

impl From<SymbolType> for Type {
    fn from(s: SymbolType) -> Self {
        match s {
            SymbolType::Int => Self::Int,
            SymbolType::Str => Self::String,
        }
    }
}

pub fn serialize(ctx: CompilerCtx) -> Vec<u8> {
    let mut bytes: Vec<u8> = Vec::new();

    magic(&mut bytes); // magic bytes
    bytes.extend(
        ctx.functions
            .main_index()
            .expect("Main function is not defined")
            .to_le_bytes(),
    ); // index of start function
    constants(&mut bytes, ctx.constants); // constants
    code(&mut bytes, ctx.functions.get_all()); // code

    let (sigs, sgroups) = ctx.signatures.get_signatures_and_groups();
    signatures(&mut bytes, sigs); // signatures
    groups(&mut bytes, sgroups); // signature groups

    return bytes;
}

fn length(bytes: &mut Vec<u8>) -> impl FnOnce(&mut Vec<u8>) {
    bytes.extend([0, 0, 0, 0]);
    let len = bytes.len();

    move |bytes: &mut Vec<u8>| {
        let size = (bytes.len() - len).to_le_bytes();

        bytes[len - 4] = size[0];
        bytes[len - 3] = size[1];
        bytes[len - 2] = size[2];
        bytes[len - 1] = size[3];
    }
}

fn signatures(bytes: &mut Vec<u8>, sigs: Vec<(&str, u32, Vec<SymbolType>, SymbolType)>) {
    bytes.extend((sigs.len() as u32).to_le_bytes());
    for (name, _, args, ret) in sigs {
        bytes.extend(name.as_bytes());
        bytes.push(0); // ending null character

        bytes.push(args.len() as u8); // argc
        args.into_iter()
            .for_each(|t| bytes.push(Type::from(t) as u8));

        bytes.push(Type::from(ret) as u8);
    }
}

fn groups(bytes: &mut Vec<u8>, groups: Vec<Vec<u32>>) {
    bytes.extend((groups.len() as u32).to_le_bytes());
    for group in groups {
        bytes.extend((group.len() as u32).to_le_bytes());
        group
            .into_iter()
            .for_each(|i| bytes.extend(i.to_le_bytes()));
    }
}

fn code(bytes: &mut Vec<u8>, code: Vec<(u32, Vec<Bytecode>, u8, u8)>) {
    bytes.extend((code.len() as u32).to_le_bytes());
    for (_, f, argc, localc) in code {
        bytes.push(argc);
        bytes.push(localc - argc);
        let end_function_length = length(bytes);

        f.into_iter().for_each(|b| b.encode(bytes));

        end_function_length(bytes);
    }
}

fn magic(bytes: &mut Vec<u8>) {
    bytes.extend(['S' as u8, 'P' as u8, 'S' as u8, 'L' as u8]);
}

fn constants(bytes: &mut Vec<u8>, cons: Vec<Constant>) {
    bytes.extend((cons.len() as u32).to_le_bytes());

    for con in cons {
        match con {
            Constant::String(s) => {
                bytes.push(Type::String as u8);
                bytes.extend(s.as_bytes());
                bytes.push(0); // ending null character
            }
            Constant::Int(dw) => {
                bytes.push(Type::Int as u8);
                bytes.extend(dw.to_le_bytes());
            }
        }
    }
}
