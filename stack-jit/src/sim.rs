use super::{asm::arg::Register, shape::ShapeType};

#[derive(Debug, Clone, Copy)]
pub enum StackVal {
    Register(Register),
    Stack(i32),

    Imm32(i32),
}

#[derive(Debug, Clone)]
pub struct Simulation {
    locals: Vec<(StackVal, ShapeType)>,
    stack: Vec<(StackVal, ShapeType, bool)>,
    free_registers: FreeRegisters,

    sp_offset: u32,

    currently_used_registers: Vec<Register>,
}

impl Simulation {
    pub fn new() -> Simulation {
        Simulation {
            locals: Vec::with_capacity(256),
            stack: Vec::with_capacity(256),

            free_registers: FreeRegisters::new(),
            sp_offset: 0,

            currently_used_registers: Vec::new(),
        }
    }

    pub fn load_args(&mut self, args: &[ShapeType]) {
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
        self.locals.extend(args.iter().map(|typ| (arg_gen(), *typ)));
        self.sp_offset += 8 + self
            .locals
            .iter()
            .filter(|(v, _)| {
                if let StackVal::Stack(_) = v {
                    true
                } else {
                    false
                }
            })
            .count() as u32
            * 8;
    }

    pub fn reserve_locals(&mut self, locals: &[ShapeType]) {
        let mut local_gen = local_generator();
        self.locals
            .extend(locals.iter().map(|typ| (local_gen(), *typ)));
    }

    pub fn used_registers(&self) -> &[Register] {
        &self.currently_used_registers
    }

    pub fn used_locals<'a>(&'a self) -> impl Iterator<Item = Register> + 'a {
        self.locals.iter().filter_map(|(s, _)| match s {
            &StackVal::Register(r) => Some(r),
            _ => None,
        })
    }

    pub fn swap(&mut self, other: &mut Simulation) {
        std::mem::swap(self, other);
    }

    pub fn local_get(&mut self, i: u8) {
        let (source, typ) = self.locals[i as usize];
        self.stack.push((source, typ, false));
    }

    pub fn local_set(&mut self, i: u8) -> (ShapeType, StackVal, StackVal) {
        let (source, typ, _) = self.stack.pop().unwrap();
        self.locals[i as usize].1 = typ;
        return (typ, source, self.locals[i as usize].0);
    }

    pub fn pop(&mut self) -> (ShapeType, StackVal) {
        self.stack
            .pop()
            .map(|(val, shape, _)| (shape, val))
            .map(|(s, v)| match v {
                StackVal::Stack(offset) => (
                    s,
                    StackVal::Stack(((self.sp_offset as i32) - offset * 8) - 8),
                ),
                StackVal::Register(r) => {
                    if let Some(index) =
                        self.currently_used_registers.iter().position(|r2| r == *r2)
                    {
                        self.currently_used_registers.remove(index);
                    }

                    (s, StackVal::Register(r))
                }
                v => (s, v),
            })
            .unwrap()
    }

    pub fn push(&mut self, (typ, v): (ShapeType, StackVal)) {
        let writable = match v {
            StackVal::Register(r) => {
                self.currently_used_registers.push(r);
                true
            }
            _ => false,
        };
        self.stack.push((v, typ, writable));
    }

    pub fn get_reg(&mut self) -> StackVal {
        let reg = self.free_registers.get().unwrap();
        self.currently_used_registers.push(reg);
        return StackVal::Register(reg);
    }

    /// checks if value on stack with given offset is writable
    fn is_writable(&self, offset: usize) -> bool {
        let len = self.stack.len();
        self.stack[len - offset - 1..len][0].2
    }

    pub fn to_writable(
        &mut self,
        offset: usize,
        write: impl FnOnce(ShapeType, StackVal, StackVal),
    ) {
        if !self.is_writable(offset) {
            let reg = self.free_registers.get().unwrap();
            let len = self.stack.len();
            let old = self.stack.get_mut(len - offset - 1).unwrap();
            write(old.1, old.0, StackVal::Register(reg));
            old.0 = StackVal::Register(reg);
            old.2 = true;

            self.currently_used_registers.push(reg);
        }
    }
}

fn local_generator() -> impl FnMut() -> StackVal {
    let mut next = Register::RBX;
    return move || {
        let v = StackVal::Register(next);
        next = match next {
            Register::RBX => Register::RBP,
            _ => unreachable!(),
        };

        return v;
    };
}

#[derive(Debug, Clone)]
struct FreeRegisters {
    regs: Vec<Register>,
}

impl FreeRegisters {
    pub fn new() -> FreeRegisters {
        let mut v = vec![
            Register::RAX,
            Register::R8,
            Register::R9,
            Register::R10,
            Register::R11,
            Register::R12,
            Register::R13,
            Register::R14,
            Register::R15,
        ];
        v.reverse();
        FreeRegisters { regs: v }
    }

    pub fn get(&mut self) -> Option<Register> {
        self.regs.pop()
    }
}
