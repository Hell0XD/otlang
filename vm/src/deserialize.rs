use std::ffi::CString;

use vm_bytecode::Bytecode;

use super::value::Value;

pub fn deserialize(
    bytes: &[u8],
) -> Result<
    (
        u32,
        Vec<Value>,
        Vec<(u8, u8, Vec<Bytecode>)>,
        Vec<(String, Vec<Type>, Type)>,
        Vec<Vec<u32>>,
    ),
    &'static str,
> {
    let (b, bool) = is_magic(bytes);
    let bytes = b;
    if !bool {
        return Err("Magic bytes are missing");
    }

    let (b, start_index) = decode_u32(bytes);
    let bytes = b;

    let (b, constants) = decode_constans(bytes);
    let bytes = b;

    let (b, functions) = decode_functions(bytes);
    let bytes = b;

    let (b, signatures) = decode_signatures(bytes);
    let bytes = b;

    let (_, groups) = decode_groups(bytes);

    Ok((start_index, constants, functions, signatures, groups))
}

fn decode_groups(bytes: &[u8]) -> (&[u8], Vec<Vec<u32>>) {
    let (b, groupc) = decode_u32(bytes);
    let mut bytes = b;

    let mut groups = Vec::with_capacity(groupc as usize);

    for _ in 0..groupc {
        let (b, sigc) = decode_u32(bytes);
        bytes = b;

        groups.push(
            (0..sigc)
                .map(|_| {
                    let (b, sig) = decode_u32(bytes);
                    bytes = b;
                    sig
                })
                .collect(),
        );
    }

    return (bytes, groups);
}

#[allow(unused)]
#[derive(Debug)]
pub enum Type {
    Int,
    String,
}

fn decode_signatures(bytes: &[u8]) -> (&[u8], Vec<(String, Vec<Type>, Type)>) {
    let (b, sigc) = decode_u32(bytes);
    let mut bytes = b;

    let mut sigs = Vec::with_capacity(sigc as usize);

    for _ in 0..sigc {
        let (b, name) = decode_string(bytes);
        bytes = b;

        let argc = bytes[0];
        bytes = &bytes[1..];

        let types: Vec<Type> = (0..argc)
            .map(|_| {
                let t = bytes[0];
                bytes = &bytes[1..];
                unsafe { std::mem::transmute(t) }
            })
            .collect();

        let ret = unsafe { std::mem::transmute(bytes[0]) };
        bytes = &bytes[1..];

        sigs.push((name, types, ret));
    }

    return (bytes, sigs);
}

fn decode_functions(bytes: &[u8]) -> (&[u8], Vec<(u8, u8, Vec<Bytecode>)>) {
    let (b, func_count) = decode_u32(bytes);
    let mut bytes = b;

    let mut vec = Vec::with_capacity(func_count as usize);

    for _ in 0..func_count {
        let argc = bytes[0];
        bytes = &bytes[1..];
        let localc = bytes[0];
        bytes = &bytes[1..];

        let (b, f_len) = decode_u32(bytes);
        bytes = b;

        let code = Bytecode::decode(bytes, f_len);
        bytes = &bytes[f_len as usize..];

        vec.push((argc, localc, code));
    }

    (bytes, vec)
}

fn decode_constans(bytes: &[u8]) -> (&[u8], Vec<Value>) {
    let (b, cons_count) = decode_u32(bytes);
    let mut bytes = b;

    let mut vec = Vec::with_capacity(cons_count as usize);

    for _ in 0..cons_count {
        match bytes[0] {
            0 => {
                bytes = &bytes[1..];

                let (b, dw) = decode_i32(bytes);
                bytes = b;

                vec.push(Value::Int(dw));
            }
            1 => {
                bytes = &bytes[1..];

                let (b, s) = decode_string(bytes);
                bytes = b;

                vec.push(Value::String(CString::new(s).unwrap()));
            }
            _ => unreachable!(),
        }
    }

    (bytes, vec)
}

fn is_magic(bytes: &[u8]) -> (&[u8], bool) {
    (
        &bytes[4..],
        bytes[0] == 'S' as u8
            && bytes[1] == 'P' as u8
            && bytes[2] == 'S' as u8
            && bytes[3] == 'L' as u8,
    )
}

fn decode_u32(bytes: &[u8]) -> (&[u8], u32) {
    (
        &bytes[4..],
        u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
    )
}

fn decode_i32(bytes: &[u8]) -> (&[u8], i32) {
    (
        &bytes[4..],
        i32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
    )
}

fn decode_string(bytes: &[u8]) -> (&[u8], String) {
    let mut i = 0;
    while bytes[i] != 0 {
        i += 1;
    }

    return (
        &bytes[i + 1..],
        String::from_utf8(bytes[0..i].to_vec()).unwrap(),
    );
}
