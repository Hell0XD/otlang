


pub struct Buffer {
    buff: [u8; 12],
    bp: usize, 
}

impl Buffer {
    pub fn new() -> Buffer {
        Buffer {
            buff: [0; 12],
            bp: 0
        }
    }

    pub fn get(&mut self) -> &[u8] {
        let ptr = &self.buff[0..self.bp];
        self.bp = 0;
        return ptr;
    }

    pub fn push(&mut self, b: u8) {
        self.buff[self.bp] = b;
        self.bp += 1;
    }

    pub fn push_many(&mut self, bs: &[u8]) {
        for b in bs {
            self.buff[self.bp] = *b;
            self.bp += 1;
        }
    }
}