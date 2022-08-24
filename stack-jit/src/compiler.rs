use std::collections::HashMap;

use crate::{
    asm::{
        self,
        arg::{Arg, Register, Size},
        buffer::Buffer,
    },
    bytecode::Bytecode,
    function::Function,
    shape::ShapeType,
    sim::{Simulation, StackVal},
};

use emem::ExecutableMemory;

const EXECUTABLE_MEMORY_SIZE: usize = 0x400;

pub struct JIT {
    mem: ExecutableMemory,
    ptr: *mut u8,

    /// args, argc, function address
    loader: extern "sysv64" fn(*const i64, u32, u64) -> i64,

    extern_functions: Vec<(u64, Vec<ShapeType>, ShapeType)>,

    functions: HashMap<u32, (Vec<ShapeType>, ShapeType, *const u8)>,
}

impl JIT {
    pub fn new() -> JIT {
        let mem = ExecutableMemory::new(EXECUTABLE_MEMORY_SIZE).unwrap();
        let mut jit = JIT {
            ptr: mem.data(),
            loader: unsafe { std::mem::transmute::<_, _>(mem.data()) },

            mem,

            extern_functions: Vec::new(),
            functions: HashMap::new(),
        };
        jit.create_loader();
        jit
    }

    pub fn add_extern_function(&mut self, addr: u64, args: Vec<ShapeType>, ret: ShapeType) {
        self.extern_functions.push((addr, args, ret));
    }

    fn create_loader(&mut self) {
        self.flush(&LOADER_BYTES);
    }

    pub fn print_machine_code(&self) {
        let mut start = unsafe { self.mem.data().offset(LOADER_BYTES.len() as isize) };
        let end = self.ptr;

        print!("| ");
        while start != end {
            unsafe {
                print!("{:02X} ", *start);
                start = start.offset(1);
            }
        }
        println!("|");
    }

    fn flush(&mut self, bytes: &[u8]) {
        unsafe {
            std::ptr::copy(bytes as *const _ as *const u8, self.ptr, bytes.len());
            self.ptr = self.ptr.offset(bytes.len() as isize);
        }
    }

    fn flush_at(at: *mut u8, bytes: &[u8]) {
        unsafe {
            std::ptr::copy(bytes as *const _ as *const u8, at, bytes.len());
        }
    }

    pub fn cast_0(&mut self, index: u32) -> Option<extern "sysv64" fn() -> i64> {
        self.functions
            .get(&index)
            .map(|f| unsafe { std::mem::transmute::<_, _>(f.2) })
    }

    pub fn cast_1(&mut self, index: u32) -> Option<extern "sysv64" fn(i64) -> i64> {
        self.functions
            .get(&index)
            .map(|f| unsafe { std::mem::transmute::<_, _>(f.2) })
    }

    pub fn cast_2(&mut self, index: u32) -> Option<extern "sysv64" fn(i64, i64) -> i64> {
        self.functions
            .get(&index)
            .map(|f| unsafe { std::mem::transmute::<_, _>(f.2) })
    }

    /// uses loader to dynamically load args from slice into registers and into the stack
    pub fn call(&self, index: u32, args: &[i64]) -> Option<i64> {
        let (shape_args, _, f) = self.functions.get(&index)?;

        assert_eq!(shape_args.len(), args.len());
        Some((self.loader)(
            args as *const _ as *const i64,
            args.len() as u32,
            (*f) as u64,
        ))
    }

    pub fn function_exist(&self, index: u32) -> bool {
        self.functions.get(&index).is_some()
    }
}

fn check_types(t1: ShapeType, t2: ShapeType) -> Result<(), ()> {
    if t1 != t2 {
        Err(())
    } else {
        Ok(())
    }
}

pub fn compile(jit: &mut JIT, index: u32, function: Function) -> Result<(), ()> {
    #[cfg(test)]
    println!("{:?}", function);

    let mut sim = Simulation::new();

    sim.load_args(&function.args);
    sim.reserve_locals(&function.locals);

    jit.functions
        .insert(index, (function.args, function.ret, jit.ptr));

    let mut buff = Buffer::new();
    let mut if_labels: Vec<(Option<Register>, Simulation, Box<dyn FnOnce(*const u8)>)> = vec![];

    for bytecode in function.code {
        match bytecode {
            Bytecode::ConstI32(dw) => sim.push((ShapeType::Int, StackVal::Imm32(dw))),
            Bytecode::LocalGet(l) => sim.local_get(l),
            Bytecode::LocalSet(l) => {
                let (typ, from, to) = sim.local_set(l);
                asm::mov(&mut buff, Arg::from((typ, to)), Arg::from((typ, from)));
                jit.flush(buff.get());
            }

            Bytecode::IfExpEq => {
                let left = sim.pop();
                let right = sim.pop();

                asm::cmp(&mut buff, left.into(), right.into());
                jit.flush(buff.get());

                asm::jne(&mut buff, Arg::Rel(0x420));
                jit.flush(buff.get());

                let saved_ptr = jit.ptr;
                if_labels.push((
                    Some(Register::RAX),
                    sim.clone(),
                    Box::new(move |current_ptr: *const u8| {
                        let distance: i32 = current_ptr as i32 - saved_ptr as i32;
                        JIT::flush_at(unsafe { saved_ptr.offset(-4) }, &distance.to_le_bytes());
                    }),
                ));
            }

            Bytecode::IfExp => {
                let cond = sim.pop();

                asm::cmp(&mut buff, cond.into(), Arg::Imm(0x1));
                jit.flush(buff.get());

                asm::jne(&mut buff, Arg::Rel(0x420));
                jit.flush(buff.get());

                let saved_ptr = jit.ptr;
                if_labels.push((
                    Some(Register::RAX),
                    sim.clone(),
                    Box::new(move |current_ptr: *const u8| {
                        let distance: i32 = current_ptr as i32 - saved_ptr as i32;
                        JIT::flush_at(unsafe { saved_ptr.offset(-4) }, &distance.to_le_bytes());
                    }),
                ));
            }

            Bytecode::If => {
                let cond = sim.pop();

                asm::cmp(&mut buff, cond.into(), Arg::Imm(0x1));
                jit.flush(buff.get());

                asm::jne(&mut buff, Arg::Rel(0x420));
                jit.flush(buff.get());

                let saved_ptr = jit.ptr;
                if_labels.push((
                    None,
                    sim.clone(),
                    Box::new(move |current_ptr: *const u8| {
                        let distance: i32 = current_ptr as i32 - saved_ptr as i32;
                        JIT::flush_at(unsafe { saved_ptr.offset(-4) }, &distance.to_le_bytes());
                    }),
                ));
            }

            Bytecode::Else => {
                let (mut r, mut label_sim, label) = if_labels.pop().unwrap();

                if r.is_some() {
                    sim.to_writable(0, |typ, from, to| {
                        asm::mov(&mut buff, Arg::from((typ, to)), Arg::from((typ, from)))
                    });
                    jit.flush(buff.get());
                    r = Some(match sim.pop().1 {
                        StackVal::Register(r) => r,
                        _ => unreachable!(),
                    });
                }

                asm::jmp(&mut buff, Arg::Rel(0x420));
                jit.flush(buff.get());

                label(jit.ptr);
                sim.swap(&mut label_sim);

                let saved_ptr = jit.ptr;
                if_labels.push((
                    r,
                    label_sim,
                    Box::new(move |current_ptr: *const u8| {
                        let distance = current_ptr as i32 - saved_ptr as i32;
                        JIT::flush_at(unsafe { saved_ptr.offset(-4) }, &distance.to_le_bytes());
                    }),
                ));
            }
            Bytecode::EndIf => {
                let (r, _, label) = if_labels.pop().unwrap();
                if r.is_some() {
                    sim.to_writable(0, |typ, from, to| {
                        asm::mov(&mut buff, Arg::from((typ, to)), Arg::from((typ, from)))
                    });
                    jit.flush(buff.get());
                }

                label(jit.ptr);
            }

            Bytecode::Add => {
                sim.to_writable(1, |typ, from, to| {
                    asm::mov(&mut buff, Arg::from((typ, to)), Arg::from((typ, from)))
                });
                jit.flush(buff.get());

                let second = sim.pop();
                let first = sim.pop();
                sim.push(first);

                check_types(first.0, second.0)?;

                asm::add(&mut buff, Arg::from(first), Arg::from(second));
                jit.flush(buff.get());
            }
            Bytecode::CallExtern(i) => {
                let (addr, arg_types, ret_type) = jit.extern_functions[i as usize].clone();

                let mut next = StackVal::Register(Register::RDI);
                let mut arg_gen = move || {
                    let v = next;
                    next = match next {
                        StackVal::Register(Register::RDI) => StackVal::Register(Register::RSI),
                        StackVal::Register(Register::RSI) => StackVal::Register(Register::RDX),
                        StackVal::Register(Register::RDX) => StackVal::Register(Register::RCX),
                        StackVal::Register(Register::RCX) => StackVal::Stack(0),
                        StackVal::Stack(i) => StackVal::Stack(i + 1),
                        _ => unreachable!(),
                    };

                    return v;
                };

                sim.used_registers().into_iter().for_each(|&reg| {
                    asm::push(&mut buff, Arg::Register(reg, Size::QWord));
                    asm::push(&mut buff, Arg::Register(reg, Size::QWord));
                    jit.flush(buff.get());
                });
                sim.used_locals().for_each(|reg| {
                    asm::push(&mut buff, Arg::Register(reg, Size::QWord));
                    jit.flush(buff.get());
                });

                arg_types.into_iter().for_each(|arg_typ| match arg_gen() {
                    StackVal::Register(r) => {
                        let from = sim.pop().into();
                        let to = (arg_typ, StackVal::Register(r)).into();
                        if from != to {
                            asm::mov(&mut buff, to, from);
                            jit.flush(buff.get());
                        }
                    }
                    StackVal::Stack(_) => {
                        asm::push(&mut buff, sim.pop().into());
                        jit.flush(buff.get());
                    }
                    _ => unreachable!(),
                });

                let rax_is_used = sim.used_registers().contains(&Register::RAX);

                if rax_is_used {
                    asm::push(&mut buff, Arg::Register(Register::RAX, Size::QWord));
                    jit.flush(buff.get());
                }

                asm::mov(
                    &mut buff,
                    Arg::Register(Register::RAX, Size::QWord),
                    Arg::Imm64(addr as i64),
                );
                jit.flush(buff.get());

                asm::call(&mut buff, Arg::Register(Register::RAX, Size::QWord));
                jit.flush(buff.get());

                let result = if rax_is_used {
                    let reg = sim.get_reg();
                    asm::mov(
                        &mut buff,
                        Arg::from((ShapeType::Int64, reg)),
                        Arg::Register(Register::RAX, Size::QWord),
                    );
                    jit.flush(buff.get());

                    asm::pop(&mut buff, Arg::Register(Register::RAX, Size::QWord));
                    jit.flush(buff.get());
                    reg
                } else {
                    StackVal::Register(Register::RAX)
                };

                sim.used_locals()
                    .collect::<Vec<Register>>()
                    .into_iter()
                    .rev()
                    .for_each(|reg| {
                        asm::pop(&mut buff, Arg::Register(reg, Size::QWord));
                        jit.flush(buff.get());
                    });
                sim.used_registers().into_iter().rev().for_each(|&reg| {
                    asm::pop(&mut buff, Arg::Register(reg, Size::QWord));
                    jit.flush(buff.get());
                });

                sim.push((ret_type, result));
            }
            Bytecode::Call(i) => {
                let (arg_types, ret_type, ptr) = jit.functions.get(&i).unwrap();
                let ret_type = *ret_type;
                let f = *ptr as i32;
                let arg_types = arg_types.clone();

                let mut next = StackVal::Register(Register::RDI);
                let mut arg_gen = move || {
                    let v = next;
                    next = match next {
                        StackVal::Register(Register::RDI) => StackVal::Register(Register::RSI),
                        StackVal::Register(Register::RSI) => StackVal::Register(Register::RDX),
                        StackVal::Register(Register::RDX) => StackVal::Register(Register::RCX),
                        StackVal::Register(Register::RCX) => StackVal::Stack(0),
                        StackVal::Stack(i) => StackVal::Stack(i + 1),
                        _ => unreachable!(),
                    };

                    return v;
                };

                sim.used_registers().into_iter().for_each(|&reg| {
                    asm::push(&mut buff, Arg::Register(reg, Size::QWord));
                    jit.flush(buff.get());
                });
                sim.used_locals().for_each(|reg| {
                    asm::push(&mut buff, Arg::Register(reg, Size::QWord));
                    jit.flush(buff.get());
                });

                arg_types.into_iter().for_each(|arg_typ| match arg_gen() {
                    StackVal::Register(r) => {
                        let from = sim.pop().into();
                        let to = (arg_typ, StackVal::Register(r)).into();
                        if from != to {
                            asm::mov(&mut buff, to, from);
                            jit.flush(buff.get());
                        }
                    }
                    StackVal::Stack(_) => {
                        asm::push(&mut buff, sim.pop().into());
                        jit.flush(buff.get());
                    }
                    _ => unreachable!(),
                });

                let current = jit.ptr as i32;
                asm::call(&mut buff, Arg::Rel(f - current));
                jit.flush(buff.get());

                sim.used_locals()
                    .collect::<Vec<Register>>()
                    .into_iter()
                    .rev()
                    .for_each(|reg| {
                        asm::pop(&mut buff, Arg::Register(reg, Size::QWord));
                        jit.flush(buff.get());
                    });
                sim.used_registers().into_iter().rev().for_each(|&reg| {
                    asm::pop(&mut buff, Arg::Register(reg, Size::QWord));
                    jit.flush(buff.get());
                });

                sim.push((ret_type, StackVal::Register(Register::RAX)));
            }
            Bytecode::CallRet(i) => {
                let (arg_types, ret_type, ptr) = jit.functions.get(&i).unwrap();
                let ret_type = *ret_type;
                let f = *ptr as i32;
                let arg_types = arg_types.clone();

                let mut next = StackVal::Register(Register::RDI);
                let mut arg_gen = move || {
                    let v = next;
                    next = match next {
                        StackVal::Register(Register::RDI) => StackVal::Register(Register::RSI),
                        StackVal::Register(Register::RSI) => StackVal::Register(Register::RDX),
                        StackVal::Register(Register::RDX) => StackVal::Register(Register::RCX),
                        StackVal::Register(Register::RCX) => StackVal::Stack(0),
                        StackVal::Stack(i) => StackVal::Stack(i + 1),
                        _ => unreachable!(),
                    };

                    return v;
                };

                arg_types.into_iter().for_each(|arg_typ| match arg_gen() {
                    StackVal::Register(r) => {
                        let from = sim.pop().into();
                        let to = (arg_typ, StackVal::Register(r)).into();
                        if from != to {
                            asm::mov(&mut buff, to, from);
                            jit.flush(buff.get());
                        }
                    }
                    StackVal::Stack(_) => {
                        asm::push(&mut buff, sim.pop().into());
                        jit.flush(buff.get());
                    }
                    _ => unreachable!(),
                });

                let current = jit.ptr as i32;
                asm::jmp(&mut buff, Arg::Rel(f - current));
                jit.flush(buff.get());
            }
            Bytecode::Ret => {
                let ret = sim.pop();

                if let StackVal::Register(Register::RAX) = ret.1 {
                } else {
                    asm::mov(
                        &mut buff,
                        Arg::Register(Register::RAX, ret.0.into()),
                        ret.into(),
                    );
                    jit.flush(buff.get());
                }

                asm::ret(&mut buff);
                jit.flush(buff.get());
            }
            Bytecode::Remove => {
                sim.pop();
            }
            _ => unimplemented!(),
        }
    }

    Ok(())
}

const LOADER_BYTES: [u8; 87] = [
    0x48, 0x89, 0xF8, 0x41, 0x89, 0xF0, 0x49, 0x89, 0xD1, 0x48, 0x6B, 0xF6, 0x08, 0x48, 0x83, 0xEE,
    0x20, 0x48, 0x89, 0xF3, 0x41, 0x83, 0xF8, 0x01, 0x7C, 0x30, 0x48, 0x8B, 0x38, 0x41, 0x83, 0xF8,
    0x02, 0x7C, 0x27, 0x48, 0x8B, 0x70, 0x08, 0x41, 0x83, 0xF8, 0x03, 0x7C, 0x1D, 0x48, 0x8B, 0x50,
    0x10, 0x41, 0x83, 0xF8, 0x04, 0x7C, 0x13, 0x48, 0x8B, 0x48, 0x18, 0x41, 0x83, 0xF8, 0x04, 0x74,
    0x09, 0x41, 0xFF, 0xC8, 0x42, 0xFF, 0x34, 0xC0, 0xEB, 0xF1, 0x41, 0xFF, 0xD1, 0x48, 0x83, 0xFB,
    0x00, 0x7E, 0x03, 0x48, 0x01, 0xDC, 0xC3,
];
