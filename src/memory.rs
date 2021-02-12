
pub const MEM_SIZE: u16 = 8192;

pub struct Memory {
    mem: Vec<u8>,
    mar: u16,
}

pub fn new(size: u16) -> Memory {
    Memory {
        mem: vec![0; size as usize],
        mar: 0,
    }
}

#[allow(dead_code)]
impl Memory {
    pub fn set_addr(&mut self, addr: u16) {
        self.mar = addr;
    }

    pub fn read(&self) -> u8 {
        assert!(self.mar < self.mem.len() as u16);
        self.mem[self.mar as usize]
    }

    pub fn write(&mut self, value: u8) {
        assert!(self.mar < self.mem.len() as u16);
        self.mem[self.mar as usize] = value;
    }

    pub fn public_read(&self, addr: u16) -> u8 {
        assert!(addr < self.mem.len() as u16);
        self.mem[addr as usize]
    }

    pub fn public_write(&mut self, value: u8, addr: u16) {
        assert!(addr < self.mem.len() as u16);
        self.mem[addr as usize] = value;
    }

    pub fn load_image(&mut self, image: Vec<u8>) {
        self.mem = image;
    }
}