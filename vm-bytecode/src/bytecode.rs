#[derive(Debug, Clone, Copy)]
pub enum Bytecode {
    LocalGet(u8),
    LocalSet(u8),

    ConstantGet(u8),

    Add,
    Sub,
    Mul,
    Div,

    Eq,
    Lt,
    Gt,
    Lte,
    Gte,

    If,
    Else,
    End,

    Call(u32),
    CallRet(u32),

    Ret,

    Remove,

    ConstantNil,

    CallLambda,

    Nop,
    Atom(u32),
    Print,

    NewPair,
    NewArray(u32),      // size of the array / number for items to pop
    NewLambda(u32, u8), // index of function, number of context values to take from stack
    NewArrayDyn,

    PairLeft,
    PairRight,
    ArrayGet,
    LocalArrayGet(u8, u32),
    ArraySet,
    ArrayLen,
    StringLen,

    IfEq,
    IfLt,
    IfGt,
    IfLte,
    IfGte,

    Import(u32),      // index of signatures
    CallDynamic(u32), // index of signature
}

impl Bytecode {
    pub fn encode(self, bytes: &mut Vec<u8>) {
        match self {
            Bytecode::LocalGet(b) => bytes.extend([0, b]),
            Bytecode::LocalSet(b) => bytes.extend([1, b]),
            Bytecode::ConstantGet(b) => bytes.extend([2, b]),
            Bytecode::Add => bytes.push(3),
            Bytecode::Sub => bytes.push(4),
            Bytecode::Mul => bytes.push(5),
            Bytecode::Div => bytes.push(6),
            Bytecode::Eq => bytes.push(7),
            Bytecode::Lt => bytes.push(8),
            Bytecode::Gt => bytes.push(9),
            Bytecode::Lte => bytes.push(10),
            Bytecode::Gte => bytes.push(11),
            Bytecode::If => bytes.push(12),
            Bytecode::Else => bytes.push(13),
            Bytecode::End => bytes.push(14),
            Bytecode::Call(dw) => {
                bytes.push(15);
                bytes.extend(dw.to_le_bytes())
            }
            Bytecode::CallRet(dw) => {
                bytes.push(16);
                bytes.extend(dw.to_le_bytes())
            }
            Bytecode::Ret => bytes.push(17),
            Bytecode::Remove => bytes.push(18),
            Bytecode::ConstantNil => bytes.push(19),
            Bytecode::CallLambda => bytes.push(20),

            Bytecode::Nop => bytes.push(22),
            Bytecode::Atom(dw) => {
                bytes.push(23);
                bytes.extend(dw.to_le_bytes());
            }
            Bytecode::Print => bytes.push(24),
            Bytecode::NewPair => bytes.push(25),
            Bytecode::NewArray(dw) => {
                bytes.push(26);
                bytes.extend(dw.to_le_bytes())
            }
            Bytecode::NewLambda(dw, b) => {
                bytes.push(27);
                bytes.extend(dw.to_le_bytes());
                bytes.push(b)
            }
            Bytecode::NewArrayDyn => bytes.push(28),

            Bytecode::PairLeft => bytes.push(30),
            Bytecode::PairRight => bytes.push(31),
            Bytecode::ArrayGet => bytes.push(32),
            Bytecode::LocalArrayGet(l, dw) => {
                bytes.push(33);
                bytes.push(l);
                bytes.extend(dw.to_le_bytes());
            }
            Bytecode::ArraySet => bytes.push(34),
            Bytecode::ArrayLen => bytes.push(35),
            Bytecode::StringLen => bytes.push(36),

            Bytecode::IfEq => bytes.push(40),
            Bytecode::IfLt => bytes.push(41),
            Bytecode::IfGt => bytes.push(42),
            Bytecode::IfLte => bytes.push(43),
            Bytecode::IfGte => bytes.push(44),

            Bytecode::Import(dw) => {
                bytes.push(50);
                bytes.extend(dw.to_le_bytes());
            }
            Bytecode::CallDynamic(dw) => {
                bytes.push(51);
                bytes.extend(dw.to_le_bytes())
            }
        }
    }

    pub fn decode(bytes: &[u8], len: u32) -> Vec<Bytecode> {
        let mut vec = Vec::with_capacity(len as usize);
        let mut i = 0;

        while i < len as usize {
            let bytecode = match bytes[i] {
                0 => {
                    i += 1;
                    Bytecode::LocalGet(bytes[i])
                }
                1 => {
                    i += 1;
                    Bytecode::LocalSet(bytes[i])
                }
                2 => {
                    i += 1;
                    Bytecode::ConstantGet(bytes[i])
                }
                3 => Bytecode::Add,
                4 => Bytecode::Sub,
                5 => Bytecode::Mul,
                6 => Bytecode::Div,
                7 => Bytecode::Eq,
                8 => Bytecode::Lt,
                9 => Bytecode::Gt,
                10 => Bytecode::Lte,
                11 => Bytecode::Gte,
                12 => Bytecode::If,
                13 => Bytecode::Else,
                14 => Bytecode::End,
                15 => {
                    let dw = u32::from_le_bytes([
                        bytes[i + 1],
                        bytes[i + 2],
                        bytes[i + 3],
                        bytes[i + 4],
                    ]);
                    i += 4;
                    Bytecode::Call(dw)
                }
                16 => {
                    let dw = u32::from_le_bytes([
                        bytes[i + 1],
                        bytes[i + 2],
                        bytes[i + 3],
                        bytes[i + 4],
                    ]);
                    i += 4;
                    Bytecode::CallRet(dw)
                }
                17 => Bytecode::Ret,
                18 => Bytecode::Remove,
                19 => Bytecode::ConstantNil,
                20 => Bytecode::CallLambda,
                22 => Bytecode::Nop,
                23 => {
                    let dw = u32::from_le_bytes([
                        bytes[i + 1],
                        bytes[i + 2],
                        bytes[i + 3],
                        bytes[i + 4],
                    ]);
                    i += 4;
                    Bytecode::Atom(dw)
                }
                24 => Bytecode::Print,
                25 => Bytecode::NewPair,
                26 => {
                    let dw = u32::from_le_bytes([
                        bytes[i + 1],
                        bytes[i + 2],
                        bytes[i + 3],
                        bytes[i + 4],
                    ]);
                    i += 4;
                    Bytecode::NewArray(dw)
                }
                27 => {
                    let dw = u32::from_le_bytes([
                        bytes[i + 1],
                        bytes[i + 2],
                        bytes[i + 3],
                        bytes[i + 4],
                    ]);
                    let b = bytes[i + 5];
                    i += 5;
                    Bytecode::NewLambda(dw, b)
                }
                28 => Bytecode::NewArrayDyn,

                30 => Bytecode::PairLeft,
                31 => Bytecode::PairRight,
                32 => Bytecode::ArrayGet,
                33 => {
                    let l = bytes[i + 1];
                    let dw = u32::from_le_bytes([
                        bytes[i + 2],
                        bytes[i + 3],
                        bytes[i + 4],
                        bytes[i + 5],
                    ]);
                    i += 5;
                    Bytecode::LocalArrayGet(l, dw)
                }
                34 => Bytecode::ArraySet,
                35 => Bytecode::ArrayLen,
                36 => Bytecode::StringLen,

                40 => Bytecode::IfEq,
                41 => Bytecode::IfLt,
                42 => Bytecode::IfGt,
                43 => Bytecode::IfLte,
                44 => Bytecode::IfGte,

                50 => {
                    let dw = u32::from_le_bytes([
                        bytes[i + 1],
                        bytes[i + 2],
                        bytes[i + 3],
                        bytes[i + 4],
                    ]);
                    i += 4;
                    Bytecode::Import(dw)
                }
                51 => {
                    let dw = u32::from_le_bytes([
                        bytes[i + 1],
                        bytes[i + 2],
                        bytes[i + 3],
                        bytes[i + 4],
                    ]);
                    i += 4;
                    Bytecode::CallDynamic(dw)
                }
                _ => unreachable!(),
            };
            vec.push(bytecode);
            i += 1;
        }

        return vec;
    }
}
