
pub struct ModRM {
    md: MOD,
    reg: u8,
    rm: u8,
}

pub enum MOD {
    ZeroByteDisplacement,
    OneByteDisplacement,
    FourByteDisplacemant,
    Register
}

impl ModRM {
    pub fn new() -> ModRM {
        ModRM { 
            md: MOD::Register,
            reg: 0, 
            rm: 0 
        }
    }

    pub fn md(&mut self, m: MOD) {
        self.md = m;
    }

    pub fn reg(&mut self, r: u8) {
        self.reg = r;
    }

    pub fn rm(&mut self, rm: u8) {
        self.rm = rm;
    }
}

impl Into<u8> for ModRM {
    fn into(self) -> u8 {
        (self.md as u8) << 6 
        | (self.reg) << 3
        | (self.rm)
    }
} 


pub struct Opcode {
    pub op: u8,
    direction: u8
}

impl Opcode {
    pub fn new(opcode: u8) -> Opcode {
        Opcode { 
            op: opcode, 
            direction: 0,  
        }
    }

    pub fn memory_to_register(&mut self) {
        self.direction = 0b0000_0010;
    }
}

impl Into<u8> for Opcode {
    fn into(self) -> u8 {
        self.op | self.direction
    }
}


pub struct REX(u8);


impl REX {
    pub fn new() -> REX {
        REX(0b0100_0000)
    }

    pub fn qword_operand(&mut self) {
        self.0 |= 0b0000_1000;
    }

    pub fn reg_extension(&mut self) {
        self.0 |= 0b0000_0100;
    }

    pub fn rm_extension(&mut self) {
        self.0 |= 0b0000_0001;
    }
}

impl Into<u8> for REX {
    fn into(self) -> u8 {
        self.0
    }
}