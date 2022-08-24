use crate::asm::arg::{Register, Size};
use crate::asm::instruction::{ModRM, Opcode, MOD, REX};

use self::arg::Arg;
use self::buffer::Buffer;

pub mod arg;
pub mod buffer;
mod instruction;

fn is_i8(n: i32) -> bool {
    n <= i8::MAX as i32 && n >= i8::MIN as i32
}

pub fn ret(buff: &mut Buffer) {
    buff.push(0xC3);
}

pub fn call(buff: &mut Buffer, arg: Arg) {
    let opcode = Opcode::new(0xE8);

    match arg {
        Arg::Rel(mut rel) => {
            buff.push(opcode.into());
            if rel < 0 {
                rel -= 5;
            }
            buff.push_many(&rel.to_le_bytes());
        }
        Arg::Register(reg, Size::QWord) => {
            buff.push(0xFF);
            let reg: u8 = reg.into();
            buff.push(0b11010_000 | reg);
        }
        arg => unreachable!("{:?}", arg),
    }
}

pub fn jne(buff: &mut Buffer, arg: Arg) {
    let mut opcode = Opcode::new(0x85);

    match arg {
        Arg::Rel(mut rel) => {
            let byte_sized = is_i8(rel);

            if byte_sized {
                opcode.op = 0x75;
            } else {
                buff.push(0x0F);
            }

            buff.push(opcode.into());
            if rel < 0 {
                rel -= 6;
            }

            if byte_sized {
                rel += 4;
                buff.push(rel as u8);
            } else {
                buff.push_many(&rel.to_le_bytes());
            }
        }
        arg => unreachable!("{:?}", arg),
    }
}

pub fn jmp(buff: &mut Buffer, arg: Arg) {
    let mut opcode = Opcode::new(0xEB);

    match arg {
        Arg::Rel(mut rel) => {
            let byte_sized = rel <= i8::MAX as i32;

            if !byte_sized {
                opcode.op = 0xE9;
            }

            buff.push(opcode.into());
            if rel < 0 {
                rel -= 5;
            }

            if byte_sized {
                rel += 3;
                buff.push(rel as u8);
            } else {
                buff.push_many(&rel.to_le_bytes());
            }
        }
        arg => unreachable!("{:?}", arg),
    }
}

macro_rules! push_and_pop {
    ($($name:ident, $opcode:expr, $opcode_memory:expr, $memory:expr, $imm:expr, $imm_byte:expr);+) => {
        $(pub fn $name(buff: &mut Buffer, arg: Arg) {
            let mut opcode = Opcode::new($opcode);
            let mut modrm = ModRM::new();

            match arg {
                Arg::Register(r, _s) => {
                    if r.is_extension() {
                        let mut rex = REX::new();
                        rex.rm_extension();
                        buff.push(rex.into());
                    }

                    let o: u8 = opcode.into();
                    let r: u8 = r.into();
                    buff.push(o | r);
                },
                Arg::Memory(r, offset) => {
                    if r.is_extension() {
                        let mut rex = REX::new();
                        rex.rm_extension();
                        buff.push(rex.into());
                    }

                    opcode.op = $opcode_memory;
                    modrm.reg($memory);
                    modrm.rm(r.into());

                    if offset == 0 {
                        modrm.md(MOD::ZeroByteDisplacement);
                    } else if is_i8(offset) {
                        modrm.md(MOD::OneByteDisplacement);
                    } else {
                        modrm.md(MOD::FourByteDisplacemant);
                    }

                    buff.push(opcode.into());
                    buff.push(modrm.into());

                    if r == Register::RSP {
                        buff.push(0x24);
                    }

                    if offset == 0 {
                    } else if is_i8(offset) {
                        buff.push(offset as u8);
                    } else {
                        buff.push_many(&offset.to_le_bytes());
                    }
                }
                Arg::Imm(imm) => {
                    opcode.op = $imm;

                    if is_i8(imm) {
                        opcode.op = $imm_byte;
                        buff.push(opcode.into());
                        buff.push(imm as u8);
                    }else{
                        buff.push(opcode.into());
                        buff.push_many(&imm.to_le_bytes());
                    }
                }
                arg => unreachable!("{:?}", arg),
            }
        })+
    };
}

push_and_pop!(
    push, 0x50, 0xFF, 6, 0x68, 0x6A;
    pop, 0x58, 0x8F, 0, 0, 0
);

pub fn cmp(buff: &mut Buffer, arg1: Arg, arg2: Arg) {
    let mut opcode = Opcode::new(0x39);
    let mut modrm = ModRM::new();

    match (arg1, arg2) {
        (Arg::Register(Register::RAX, s), Arg::Imm(imm)) => {
            if s == Size::QWord {
                let mut rex = REX::new();

                if s == Size::QWord {
                    rex.qword_operand()
                }

                buff.push(rex.into())
            }

            if is_i8(imm) {
                buff.push(0x3C);
                buff.push(imm as u8);
            } else {
                buff.push(0x3D);
                buff.push_many(&imm.to_le_bytes());
            }
        }
        (Arg::Register(r1, s), Arg::Imm(imm)) => {
            if s == Size::QWord || r1.is_extension() {
                let mut rex = REX::new();

                if s == Size::QWord {
                    rex.qword_operand()
                }
                if r1.is_extension() {
                    rex.rm_extension()
                }

                buff.push(rex.into())
            }

            opcode.op = 0x81;

            modrm.reg(0x7);
            modrm.rm(r1.into());

            buff.push(opcode.into());
            buff.push(modrm.into());
            buff.push_many(&imm.to_le_bytes());
        }
        (Arg::Register(r1, s), Arg::Memory(r2, offset)) => {
            if s == Size::QWord || r1.is_extension() || r2.is_extension() {
                let mut rex = REX::new();

                if s == Size::QWord {
                    rex.qword_operand()
                }
                if r1.is_extension() {
                    rex.reg_extension()
                }
                if r2.is_extension() {
                    rex.rm_extension()
                }

                buff.push(rex.into())
            }

            opcode.memory_to_register();
            buff.push(opcode.into());

            if offset == 0 {
                modrm.md(MOD::ZeroByteDisplacement);
            } else if offset <= i8::MAX as i32 {
                modrm.md(MOD::OneByteDisplacement);
            } else {
                modrm.md(MOD::FourByteDisplacemant);
            }

            modrm.reg(r1.into());
            modrm.rm(r2.into());
            buff.push(modrm.into());

            if r2 == Register::RSP {
                buff.push(0x24);
            }

            if offset == 0 {
            } else if offset <= i8::MAX as i32 {
                buff.push(offset as u8);
            } else {
                buff.push_many(&offset.to_le_bytes());
            }
        }
        (Arg::Register(r1, s1), Arg::Register(r2, s2)) => {
            assert_eq!(s1, s2);

            if s1 == Size::QWord || r1.is_extension() || r2.is_extension() {
                let mut rex = REX::new();

                if s1 == Size::QWord {
                    rex.qword_operand()
                }
                if r1.is_extension() {
                    rex.reg_extension()
                }
                if r2.is_extension() {
                    rex.rm_extension()
                }

                buff.push(rex.into())
            }

            opcode.memory_to_register();

            modrm.md(MOD::Register);
            modrm.reg(r1.into());
            modrm.rm(r2.into());

            buff.push(opcode.into());
            buff.push(modrm.into());
        }

        (arg1, arg2) => unreachable!("{:?}, {:?}", arg1, arg2),
    }
}

pub fn add(buff: &mut Buffer, arg1: Arg, arg2: Arg) {
    let mut opcode = Opcode::new(0x01);
    let mut modrm = ModRM::new();

    match (arg1, arg2) {
        (Arg::Register(Register::RAX, s), Arg::Imm(imm)) => {
            if s == Size::QWord {
                let mut rex = REX::new();

                if s == Size::QWord {
                    rex.qword_operand()
                }

                buff.push(rex.into())
            }

            if is_i8(imm) {
                buff.push(0x04);
                buff.push(imm as u8);
            } else {
                buff.push(0x05);
                buff.push_many(&imm.to_le_bytes());
            }
        }
        (Arg::Register(r1, s), Arg::Imm(imm)) => {
            if s == Size::QWord || r1.is_extension() {
                let mut rex = REX::new();

                if s == Size::QWord {
                    rex.qword_operand()
                }
                if r1.is_extension() {
                    rex.rm_extension()
                }

                buff.push(rex.into())
            }

            let is_byte_sized = is_i8(imm);

            if is_byte_sized {
                opcode.op = 0x80;
            } else {
                opcode.op = 0x81;
            }

            modrm.reg(0x0);
            modrm.rm(r1.into());

            buff.push(opcode.into());
            buff.push(modrm.into());

            if is_byte_sized {
                buff.push(imm as u8);
            } else {
                buff.push_many(&imm.to_le_bytes())
            }
        }
        (Arg::Register(r1, s), Arg::Memory(r2, offset)) => {
            if s == Size::QWord || r1.is_extension() || r2.is_extension() {
                let mut rex = REX::new();

                if s == Size::QWord {
                    rex.qword_operand()
                }
                if r1.is_extension() {
                    rex.reg_extension()
                }
                if r2.is_extension() {
                    rex.rm_extension()
                }

                buff.push(rex.into())
            }

            opcode.memory_to_register();
            buff.push(opcode.into());

            if offset == 0 {
                modrm.md(MOD::ZeroByteDisplacement);
            } else if is_i8(offset) {
                modrm.md(MOD::OneByteDisplacement);
            } else {
                modrm.md(MOD::FourByteDisplacemant);
            }

            modrm.reg(r1.into());
            modrm.rm(r2.into());
            buff.push(modrm.into());

            if r2 == Register::RSP {
                buff.push(0x24);
            }

            if offset == 0 {
            } else if is_i8(offset) {
                buff.push(offset as u8);
            } else {
                buff.push_many(&offset.to_le_bytes());
            }
        }
        (Arg::Register(r1, s1), Arg::Register(r2, s2)) => {
            assert_eq!(s1, s2);

            if s1 == Size::QWord || r1.is_extension() || r2.is_extension() {
                let mut rex = REX::new();

                if s1 == Size::QWord {
                    rex.qword_operand()
                }
                if r1.is_extension() {
                    rex.reg_extension()
                }
                if r2.is_extension() {
                    rex.rm_extension()
                }

                buff.push(rex.into())
            }

            opcode.memory_to_register();

            modrm.md(MOD::Register);
            modrm.reg(r1.into());
            modrm.rm(r2.into());

            buff.push(opcode.into());
            buff.push(modrm.into());
        }

        (arg1, arg2) => unreachable!("{:?}, {:?}", arg1, arg2),
    }
}

pub fn mov(buff: &mut Buffer, arg1: Arg, arg2: Arg) {
    let mut opcode = Opcode::new(0x89);
    let mut modrm = ModRM::new();

    match (arg1, arg2) {
        (Arg::Register(r, Size::QWord), Arg::Imm64(imm)) => {
            let mut rex = REX::new();

            if r.is_extension() {
                rex.rm_extension()
            }
            rex.qword_operand();

            buff.push(rex.into());

            opcode.op = 0xb8;

            buff.push(opcode.into());

            buff.push_many(&imm.to_le_bytes());
        }
        (Arg::Register(r, _s), Arg::Imm(imm)) => {
            if r.is_extension() {
                let mut rex = REX::new();

                if r.is_extension() {
                    rex.rm_extension()
                }

                buff.push(rex.into())
            }

            let byte_sized = imm <= i8::MAX.into();

            if byte_sized {
                opcode.op = 0xb0;
            } else {
                opcode.op = 0xb8;
            }

            let o: u8 = opcode.into();
            let r: u8 = r.into();
            buff.push(o | r);

            if byte_sized {
                buff.push(imm as u8);
            } else {
                buff.push_many(&imm.to_le_bytes());
            }
        }
        (Arg::Register(r1, s1), Arg::Register(r2, s2)) => {
            assert_eq!(s1, s2);

            if s1 == Size::QWord || r1.is_extension() || r2.is_extension() {
                let mut rex = REX::new();

                if s1 == Size::QWord {
                    rex.qword_operand()
                }
                if r1.is_extension() {
                    rex.reg_extension()
                }
                if r2.is_extension() {
                    rex.rm_extension()
                }

                buff.push(rex.into())
            }

            opcode.memory_to_register();

            modrm.md(MOD::Register);
            modrm.reg(r1.into());
            modrm.rm(r2.into());

            buff.push(opcode.into());
            buff.push(modrm.into());
        }
        (arg1, arg2) => unreachable!("{:?}, {:?}", arg1, arg2),
    }
}
