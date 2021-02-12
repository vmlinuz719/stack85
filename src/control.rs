use crate::alu;
use crate::memory;
use std::cell::UnsafeCell;
use std::num::Wrapping;
use std::sync::{Arc, RwLock};
use std::thread;

// bytecode!

// increment 1
pub const WAIT: u8 = 0b00000000; // pause execution until interrupt
pub const RESET: u8 = 0b00000001; // reset everything

pub const OVERFLOW: u8 = 0b00000010;

pub const BRANCH: u8 = 0b00000100; // add top of stack to inst ptr
pub const BRANCH_S: u8 = 0b00000101; // same but uses sign-magnitude

pub const ENTER: u8 = 0b00000110; // new local variable pointer
pub const LEAVE: u8 = 0b00000111; // restore

pub const LOAD_0: u8 = 0b00001000; // push from save_0
pub const LOAD_1: u8 = 0b00001001; // push from save_1
pub const LOAD_2: u8 = 0b00001010; // push from save_2
pub const LOAD_3: u8 = 0b00001011; // push from save_3

pub const UNLINK: u8 = 0b00001100; // push link register
pub const LINK: u8 = 0b00001101; // pop link register
pub const CALL: u8 = 0b00001110;
pub const GOBACK: u8 = 0b00001111;

pub const SAVE_0: u8 = 0b00010000; // pop into save_0
pub const SAVE_1: u8 = 0b00010001; // pop into save_1
pub const SAVE_2: u8 = 0b00010010; // pop into save_2
pub const SAVE_3: u8 = 0b00010011; // pop into save_3

pub const LOCAL_0: u8 = 0b00010100;
pub const LOCAL_1: u8 = 0b00010101;
pub const LOCAL_2: u8 = 0b00010110;
pub const LOCAL_3: u8 = 0b00010111;

pub const CONST_0: u8 = 0b00011000; // push 0
pub const CONST_1: u8 = 0b00011001; // push 1
pub const CONST_2: u8 = 0b00011010; // push 2
pub const CONST_3: u8 = 0b00011011; // push 3

pub const LOAD: u8 = 0b00011100;
pub const SAVE: u8 = 0b00011101;

pub const DUP_B: u8 = 0b00011110; // duplicate byte atop stack

pub const CLEAR_FLAGS: u8 = 0b00100000;
pub const TEST: u8 = 0b00100001;
pub const ADD: u8 = 0b00100010;
pub const ADD_CARRY: u8 = 0b00100011;
pub const SUBTRACT: u8 = 0b00100100;
pub const SUB_BORROW: u8 = 0b00100101;
pub const MULTIPLY: u8 = 0b00100110;
pub const COMPARE: u8 = 0b00100111; // subtract, no push back
pub const SHIFT_LEFT: u8 = 0b00101000;
pub const SHIFT_RIGHT: u8 = 0b00101001;
pub const ROTATE_LEFT: u8 = 0b00101010;
pub const ROTATE_RIGHT: u8 = 0b00101011;
pub const NOT: u8 = 0b00101100;
pub const AND: u8 = 0b00101101;
pub const INCLUSIVE_OR: u8 = 0b00101110;
pub const EXCLUSIVE_OR: u8 = 0b00101111;

// conditional execution
pub const IF_EQUAL: u8 = 0b00110000;
pub const IF_UNEQUAL: u8 = 0b00110001;
pub const IF_POSITIVE: u8 = 0b00110010;
pub const IF_NEGATIVE: u8 = 0b00110011;
pub const IF_ODD: u8 = 0b00110100;
pub const IF_EVEN: u8 = 0b00110101;
pub const IF_OVERFLOW: u8 = 0b00110110;
pub const IF_NO_OVERFLOW: u8 = 0b00110111;
pub const IF_GREATER_EQUAL: u8 = 0b00111000;
pub const IF_LESS_EQUAL: u8 = 0b00111001;
pub const IF_GREATER: u8 = 0b00111010;
pub const IF_LESS: u8 = 0b00111011;
pub const IF_HIGHER: u8 = 0b00111100; // unsigned >
pub const IF_LOWER: u8 = 0b00111101; // unsigned <
pub const IF_CARRY: u8 = 0b00111110;
pub const IF_NO_CARRY: u8 = 0b00111111;

// increment 2
pub const IMM_BRANCH: u8 = 0b01000000; // add immediate byte to inst ptr
pub const IMM_BRANCH_S: u8 = 0b01000001; // same but respects sign
pub const IMM_CONST: u8 = 0b01000010; // push immediate value
pub const LOCAL: u8 = 0b01001000;

// increment 3
pub const GOTO: u8 = 0b10000010; // set instruction pointer
pub const SET_STACK: u8 = 0b10000011; // set stack pointer
pub const IMM_LOAD: u8 = 0b10001100; // load from immediate address
pub const IMM_LOAD_OFFSET_B: u8 = 0b10001101; // above plus top byte of stack
pub const IMPL_DEP_0: u8 = 0b10001111;
pub const IMM_CONST_D: u8 = IMPL_DEP_0;
pub const IMM_SAVE: u8 = 0b10010100; // save to immediate address
pub const IMM_SAVE_OFFSET_B: u8 = 0b10010101; // above plus top byte of stack
pub const IMPL_DEP_1: u8 = 0b10010111;

pub struct Control {
    instr_ptr: Wrapping<u16>,
    stack_ptr: Wrapping<u16>,
    alu: alu::ALU,
    mem: memory::Memory,
    save_0: u8,
    save_1: u8,
    save_2: u8,
    save_3: u8,
    link: u16,
    local: u16,
    running: bool,
}

pub fn new() -> Control {
    Control {
        instr_ptr: Wrapping(0),
        stack_ptr: Wrapping(0),
        alu: alu::new(),
        mem: memory::new(0),
        save_0: 0,
        save_1: 0,
        save_2: 0,
        save_3: 0,
        link: 0,
        local: 0,
        running: false,
    }
}

macro_rules! push {
    ($slf:expr, $x:expr) => {
        ($slf).stack_ptr += Wrapping(1);
        ($slf).mem.set_addr(($slf).stack_ptr.0);
        ($slf).mem.write(($x));
    };
}

macro_rules! alu_op {
    ($slf:expr, $x:expr) => {
        ($slf).mem.set_addr(($slf).stack_ptr.0);
        ($slf).alu.load_y(($slf).mem.read());
        ($slf).mem.set_addr((($slf).stack_ptr - Wrapping(1)).0);
        ($slf).alu.load_x(($slf).mem.read());
        ($slf).alu.load_op(($x));
        ($slf).alu.compute();
        ($slf).stack_ptr -= Wrapping(1);
        ($slf).mem.set_addr(($slf).stack_ptr.0);
        ($slf).mem.write(($slf).alu.result());
    };
}

macro_rules! local {
    ($slf:expr, $x:expr) => {
        let address = (Wrapping(($slf).local) + Wrapping(($x))).0;
        ($slf).mem.set_addr(address);
        let x = ($slf).mem.read();
        push!(($slf), x);
    };
}

macro_rules! cond {
    ($slf:expr, $x:expr) => {
        ($slf).mem.set_addr(($slf).instr_ptr.0);
        let skip_distance = (($slf).mem.read() >> 6) as u16;
        if !($x) {
            ($slf).instr_ptr += Wrapping(skip_distance + 1);
        }
    };
}

impl Control {
    pub fn load_image(&mut self, image: Vec<u8>) {
        self.mem.load_image(image);
    }

    pub fn start(&mut self) {
        self.running = true;
    }

    pub fn is_running(&self) -> bool {
        self.running
    }

    pub fn view(&self) {
        println!("IP: {:04X} SP: {:04X}", self.instr_ptr, self.stack_ptr);
        println!("LN: {:04X} LO: {:04X}", self.link, self.local);
        println!("S0:   {:02X} S1:   {:02X}", self.save_0, self.save_1);
        println!("S2:   {:02X} S3:   {:02X}\n", self.save_2, self.save_3);
    }

    pub fn execute_instruction(&mut self) {
        // fetch instruction
        // println!("{:04X}", self.instr_ptr.0);
        self.mem.set_addr(self.instr_ptr.0);
        let instruction = self.mem.read();
        self.mem.set_addr(self.instr_ptr.0 + 1);
        let param_low = self.mem.read();
        self.mem.set_addr(self.instr_ptr.0 + 2);
        let param_high = self.mem.read();
        let param_16 = (param_high as u16) << 8 | (param_low as u16);

        // decode: calculate increment
        self.instr_ptr += Wrapping(1 + (instruction >> 6) as u16);

        // execute
        match instruction {
            WAIT => self.running = false,
            RESET => {
                self.instr_ptr = Wrapping(0);
                self.stack_ptr = Wrapping(0);
                self.alu.reset();
                self.save_0 = 0;
                self.save_1 = 0;
                self.save_2 = 0;
                self.save_3 = 0;
                self.link = 0;
                self.local = 0;
                self.running = true;
            }

            IMM_BRANCH => {
                self.instr_ptr += Wrapping(param_low as u16);
            }
            IMM_BRANCH_S => {
                self.instr_ptr += Wrapping(((param_low as i8) as i16) as u16);
            }

            OVERFLOW => {
                self.stack_ptr += Wrapping(1);
                self.mem.set_addr(self.stack_ptr.0);
                self.mem.write(self.alu.res_hi());
            }

            LOAD_0 => {
                push!(self, self.save_0);
            }
            LOAD_1 => {
                push!(self, self.save_1);
            }
            LOAD_2 => {
                push!(self, self.save_2);
            }
            LOAD_3 => {
                push!(self, self.save_3);
            }

            CONST_0 => {
                push!(self, 0);
            }
            CONST_1 => {
                push!(self, 1);
            }
            CONST_2 => {
                push!(self, 2);
            }
            CONST_3 => {
                push!(self, 3);
            }

            DUP_B => {
                self.mem.set_addr(self.stack_ptr.0);
                let x = self.mem.read();
                push!(self, x);
            }

            IMM_CONST => {
                push!(self, param_low);
            }

            SAVE_0 => {
                self.mem.set_addr(self.stack_ptr.0);
                let data = self.mem.read();
                self.save_0 = data;
                self.stack_ptr -= Wrapping(1);
            }
            SAVE_1 => {
                self.mem.set_addr(self.stack_ptr.0);
                let data = self.mem.read();
                self.save_1 = data;
                self.stack_ptr -= Wrapping(1);
            }
            SAVE_2 => {
                self.mem.set_addr(self.stack_ptr.0);
                let data = self.mem.read();
                self.save_2 = data;
                self.stack_ptr -= Wrapping(1);
            }
            SAVE_3 => {
                self.mem.set_addr(self.stack_ptr.0);
                let data = self.mem.read();
                self.save_3 = data;
                self.stack_ptr -= Wrapping(1);
            }

            UNLINK => {
                let ret_high = (self.link >> 8 & 0xFF) as u8;
                let ret_low = (self.link & 0xFF) as u8;
                push!(self, ret_low);
                push!(self, ret_high);
            }
            LINK => {
                self.mem.set_addr(self.stack_ptr.0);
                let ret_high = self.mem.read();
                self.stack_ptr -= Wrapping(1);
                self.mem.set_addr(self.stack_ptr.0);
                let ret_low = self.mem.read();
                self.stack_ptr -= Wrapping(1);

                self.link = (ret_high as u16) << 8 | (ret_low as u16);
            }

            ENTER => {
                let ret_high = (self.local >> 8 & 0xFF) as u8;
                let ret_low = (self.local & 0xFF) as u8;

                push!(self, ret_low);
                push!(self, ret_high);

                self.local = (self.stack_ptr + Wrapping(1)).0;
            }
            LEAVE => {
                self.mem.set_addr((Wrapping(self.local) - Wrapping(1)).0);
                let ret_high = self.mem.read();
                self.mem.set_addr((Wrapping(self.local) - Wrapping(2)).0);
                let ret_low = self.mem.read();
                self.stack_ptr = Wrapping(self.local) - Wrapping(3);

                self.local = (ret_high as u16) << 8 | (ret_low as u16);
            }

            CALL => {
                self.mem.set_addr(self.stack_ptr.0);
                let tgt_high = self.mem.read();
                self.stack_ptr -= Wrapping(1);
                self.mem.set_addr(self.stack_ptr.0);
                let tgt_low = self.mem.read();
                self.stack_ptr -= Wrapping(1);

                let target = (tgt_high as u16) << 8 | (tgt_low as u16);

                self.link = self.instr_ptr.0;
                self.instr_ptr = Wrapping(target);
            }
            GOBACK => {
                self.instr_ptr = Wrapping(self.link);
            }

            IMM_CONST_D => {
                push!(self, param_low);
                push!(self, param_high);
            }

            CLEAR_FLAGS => {
                self.alu.load_op(alu::ALU_RST);
                self.alu.compute();
            }
            TEST => {
                self.mem.set_addr(self.stack_ptr.0);
                self.alu.load_x(self.mem.read());
                self.alu.load_op(alu::ALU_NOP);
                self.alu.compute();
                self.stack_ptr -= Wrapping(1);
            }
            NOT => {
                self.mem.set_addr(self.stack_ptr.0);
                self.alu.load_x(self.mem.read());
                self.alu.load_op(alu::ALU_NOT);
                self.alu.compute();
                self.mem.set_addr(self.stack_ptr.0);
                self.mem.write(self.alu.result());
            }
            COMPARE => {
                self.mem.set_addr(self.stack_ptr.0);
                self.alu.load_y(self.mem.read());
                self.mem.set_addr((self.stack_ptr - Wrapping(1)).0);
                self.alu.load_x(self.mem.read());
                self.alu.load_op(alu::ALU_SUB);
                self.alu.compute();
                self.stack_ptr -= Wrapping(2);
            }
            ADD => {
                alu_op!(self, alu::ALU_ADD);
            }
            ADD_CARRY => {
                alu_op!(self, alu::ALU_ADC);
            }
            SUBTRACT => {
                alu_op!(self, alu::ALU_SUB);
            }
            SUB_BORROW => {
                alu_op!(self, alu::ALU_SBB);
            }
            MULTIPLY => {
                alu_op!(self, alu::ALU_MUL);
            }
            SHIFT_LEFT => {
                alu_op!(self, alu::ALU_SHL);
            }
            SHIFT_RIGHT => {
                alu_op!(self, alu::ALU_SHR);
            }
            ROTATE_LEFT => {
                alu_op!(self, alu::ALU_ROL);
            }
            ROTATE_RIGHT => {
                alu_op!(self, alu::ALU_ROR);
            }
            AND => {
                alu_op!(self, alu::ALU_AND);
            }
            INCLUSIVE_OR => {
                alu_op!(self, alu::ALU_IOR);
            }
            EXCLUSIVE_OR => {
                alu_op!(self, alu::ALU_XOR);
            }

            IF_EQUAL => {
                cond!(self, self.alu.test_z());
            }
            IF_UNEQUAL => {
                cond!(self, !self.alu.test_z());
            }
            IF_POSITIVE => {
                cond!(self, !self.alu.test_n());
            }
            IF_NEGATIVE => {
                cond!(self, self.alu.test_n());
            }
            IF_EVEN => {
                cond!(self, !self.alu.test_o());
            }
            IF_ODD => {
                cond!(self, self.alu.test_o());
            }
            IF_CARRY => {
                cond!(self, self.alu.test_c());
            }
            IF_NO_CARRY => {
                cond!(self, !self.alu.test_c());
            }

            SET_STACK => {
                self.stack_ptr = Wrapping(param_16);
            }

            GOTO => {
                self.instr_ptr = Wrapping(param_16);
            }

            IMM_LOAD => {
                self.mem.set_addr(param_16);
                let x = self.mem.read();
                push!(self, x);
            }
            IMM_LOAD_OFFSET_B => {
                self.mem.set_addr(self.stack_ptr.0);
                let offset = self.mem.read();
                let final_addr = (Wrapping(param_16) + Wrapping(offset as u16)).0;
                self.mem.set_addr(final_addr);
                let x = self.mem.read();

                self.mem.set_addr(self.stack_ptr.0);
                self.mem.write(x);
            }
            LOAD => {
                self.mem.set_addr(self.stack_ptr.0);
                let addr_high = self.mem.read();
                self.stack_ptr -= Wrapping(1);
                self.mem.set_addr(self.stack_ptr.0);
                let addr_low = self.mem.read();

                let addr: u16 = (addr_high as u16) << 8 | (addr_low as u16);

                self.mem.set_addr(addr);
                let x = self.mem.read();

                self.mem.set_addr(self.stack_ptr.0);
                self.mem.write(x);
            }

            IMM_SAVE => {
                self.mem.set_addr(self.stack_ptr.0);
                let x = self.mem.read();
                self.stack_ptr -= Wrapping(1);
                self.mem.set_addr(param_16);
                self.mem.write(x);
            }
            IMM_SAVE_OFFSET_B => {
                self.mem.set_addr(self.stack_ptr.0);
                let offset = self.mem.read();
                let final_addr = (Wrapping(param_16) + Wrapping(offset as u16)).0;
                self.stack_ptr -= Wrapping(1);
                self.mem.set_addr(self.stack_ptr.0);
                let x = self.mem.read();
                self.stack_ptr -= Wrapping(1);

                self.mem.set_addr(final_addr);
                self.mem.write(x);
            }
            SAVE => {
                self.mem.set_addr(self.stack_ptr.0);
                let addr_high = self.mem.read();
                self.stack_ptr -= Wrapping(1);
                self.mem.set_addr(self.stack_ptr.0);
                let addr_low = self.mem.read();
                self.stack_ptr -= Wrapping(1);

                let addr: u16 = (addr_high as u16) << 8 | (addr_low as u16);

                self.mem.set_addr(self.stack_ptr.0);
                let x = self.mem.read();
                self.stack_ptr -= Wrapping(1);

                self.mem.set_addr(addr);
                self.mem.write(x);
            }

            LOCAL => {
                local!(self, param_low as u16);
            }
            LOCAL_0 => {
                local!(self, 0);
            }
            LOCAL_1 => {
                local!(self, 1);
            }
            LOCAL_2 => {
                local!(self, 2);
            }
            LOCAL_3 => {
                local!(self, 3);
            }

            _ => self.running = false,
        }
    }
}

pub fn debug_print(image: &Vec<u8>) {
    print!("     0  1  2  3  4  5  6  7  8  9  A  B  C  D  E  F");

    let mut address: u16 = 0;
    let mut count: u8 = 16;

    for x in image {
        if count == 16 {
            count = 0;
            println!();
            print!("{:03X} ", address >> 4 & 0xFFF);
        }

        print!("{:02X} ", x);
        count += 1;
        address += 1;
    }
    println!();
}

struct ControlRace(UnsafeCell<Control>);

unsafe impl Sync for ControlRace {}
impl ControlRace {
    fn new(v: Control) -> ControlRace {
        return ControlRace(UnsafeCell::new(v));
    }

    unsafe fn get(&self) -> *mut Control {
        return self.0.get();
    }
}

pub fn test_pgm() {
    let program: Vec<u8> = vec![
        // initialize
        SET_STACK,
        0x00,
        0x01,
        CONST_3,
        SAVE_0,
        // START of program
        ENTER,
        IMM_CONST,
        48,
        IMM_CONST,
        16,
        IMM_CONST_D,
        0x10,
        0x00, // SUM_EQUALS_64
        CALL,
        LEAVE,
        WAIT,
        // subroutine SUM_EQUALS_64
        LOCAL_1,
        LOCAL_0,
        ADD,
        LOCAL_2,
        IMM_CONST,
        64,
        COMPARE,
        IF_EQUAL,
        IMM_BRANCH,
        2,
        IMM_BRANCH,
        3,
        CONST_1,
        SAVE_0,
        GOBACK,
        CONST_0,
        SAVE_0,
        GOBACK,
    ];

    debug_print(&program);
    println!();

    let mut image: Vec<u8> = vec![0; 512];

    let mut index = 0;
    for x in program {
        image[index] = x;
        index += 1;
    }

    let mut control = new();
    control.load_image(image);

    control.start();

    let ptr = Arc::new(ControlRace::new(control));
    let cln1 = ptr.clone();
    let cln2 = ptr.clone();

    unsafe {
        let exec = thread::spawn(move || {
            let control = (*cln1).get();
            while (*control).is_running() {
                (*control).execute_instruction();
            }
        });

        let monitor = thread::spawn(move || {
            let control = (*cln2).get();
            loop {
                if (*control).mem.public_read(175) == 1 {
                    for addr in 176..255 {
                        let ch = (*control).mem.public_read(addr);
                        if 32 <= ch && ch <= 126 {}
                    }
                }
            }
        });

        exec.join().unwrap();

        let control = ptr.clone().get();
        (*control).view();
    }
}
