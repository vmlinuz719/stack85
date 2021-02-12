use bit::BitIndex;
use std::num::Wrapping;

pub const FLAG_O: usize = 4; // odd parity flag
pub const FLAG_N: usize = 3; // negative flag
pub const FLAG_Z: usize = 2; // zero flag
pub const FLAG_V: usize = 1; // overflow flag
pub const FLAG_C: usize = 0; // carry flag

pub const ALU_RST: u8 = 0;
pub const ALU_NOP: u8 = 1;
pub const ALU_ADD: u8 = 2;
pub const ALU_ADC: u8 = 3;
pub const ALU_SUB: u8 = 4;
pub const ALU_SBB: u8 = 5;
pub const ALU_MUL: u8 = 6;
pub const ALU_RES: u8 = 7;
pub const ALU_SHL: u8 = 8;
pub const ALU_SHR: u8 = 9;
pub const ALU_ROL: u8 = 10;
pub const ALU_ROR: u8 = 11;
pub const ALU_NOT: u8 = 12;
pub const ALU_AND: u8 = 13;
pub const ALU_IOR: u8 = 14;
pub const ALU_XOR: u8 = 15;

pub const ALU_MAX_OPCODE: u8 = 15;

pub struct ALU {
    x: u8,
    y: u8,
    op: u8,
    result: u8,
    res_hi: u8,
    flags: u8,
}

pub fn new() -> ALU {
    ALU {
        x: 0,
        y: 0,
        op: 0,
        result: 0,
        res_hi: 0,
        flags: 0,
    }
}

#[allow(dead_code)]
impl ALU {
    pub fn load_x(&mut self, x: u8) {
        self.x = x;
    }

    pub fn load_y(&mut self, y: u8) {
        self.y = y;
    }

    pub fn load_op(&mut self, op: u8) {
        self.op = op;
    }

    pub fn compute(&mut self) {
        assert!(self.op < ALU_MAX_OPCODE);
        match self.op {
            ALU_RST => alu_rst(self),
            ALU_NOP => alu_nop(self),
            ALU_ADD => alu_add(self),
            ALU_ADC => alu_adc(self),
            ALU_SUB => alu_sub(self),
            ALU_SBB => alu_sbb(self),
            ALU_MUL => alu_mul(self),
            ALU_RES => alu_nop(self),
            ALU_SHL => alu_shl(self),
            ALU_SHR => alu_shr(self),
            ALU_ROL => alu_rol(self),
            ALU_ROR => alu_ror(self),
            ALU_NOT => alu_not(self),
            ALU_AND => alu_and(self),
            ALU_IOR => alu_ior(self),
            ALU_XOR => alu_xor(self),

            _ => alu_nop(self),
        }
    }

    pub fn reset(&mut self) {
        self.flags = 0;
    }

    pub fn result(&self) -> u8 {
        self.result
    }

    pub fn res_hi(&self) -> u8 {
        self.res_hi
    }

    pub fn flags(&self) -> u8 {
        self.flags
    }

    pub fn test_o(&self) -> bool {
        self.flags.bit(FLAG_O)
    }

    pub fn test_n(&self) -> bool {
        self.flags.bit(FLAG_N)
    }

    pub fn test_z(&self) -> bool {
        self.flags.bit(FLAG_Z)
    }

    pub fn test_v(&self) -> bool {
        self.flags.bit(FLAG_V)
    }

    pub fn test_c(&self) -> bool {
        self.flags.bit(FLAG_C)
    }
}

// Reset all
fn alu_rst(alu: &mut ALU) {
    alu.x = 0;
    alu.y = 0;
    alu.op = 0;
    alu.result = 0;
    alu.res_hi = 0;
    alu.flags = 0;
}

// No operation; result = x
fn alu_nop(alu: &mut ALU) {
    alu.result = alu.x;

    alu.flags = 0;

    if alu.x.bit(0) {
        alu.flags.set_bit(FLAG_O, true);
    }

    if alu.x.bit(7) {
        alu.flags.set_bit(FLAG_N, true);
    }

    if alu.x == 0 {
        alu.flags.set_bit(FLAG_Z, true);
    }
}

// Add without carry
fn alu_add(alu: &mut ALU) {
    let result = (Wrapping(alu.x) + Wrapping(alu.y)).0;
    let carry = alu.x as u16 + alu.y as u16;

    alu.flags = 0;
    // calculate odd flag
    if result.bit(0) {
        alu.flags.set_bit(FLAG_O, true);
    }

    // calculate negative flag
    if result.bit(7) {
        alu.flags.set_bit(FLAG_N, true);
    }

    // calculate zero flag
    if result == 0 {
        alu.flags.set_bit(FLAG_Z, true);
    }

    // calculate overflow flag
    if !alu.x.bit(7) && !alu.y.bit(7) && result.bit(7)
        || alu.x.bit(7) && alu.y.bit(7) && !result.bit(7)
    {
        alu.flags.set_bit(FLAG_V, true);
    }

    // calculate carry flag
    if carry.bit(8) {
        alu.flags.set_bit(FLAG_C, true);
    }

    // write result
    alu.result = result;
}

// Add with carry
fn alu_adc(alu: &mut ALU) {
    let carry_in = alu.test_c();

    let result: u8 = if carry_in {
        (Wrapping(alu.x) + Wrapping(alu.y)).0 + 1
    } else {
        (Wrapping(alu.x) + Wrapping(alu.y)).0
    };

    let carry: u16 = if carry_in {
        alu.x as u16 + alu.y as u16 + 1
    } else {
        alu.x as u16 + alu.y as u16
    };

    alu.flags = 0;
    // calculate odd flag
    if result.bit(0) {
        alu.flags.set_bit(FLAG_O, true);
    }

    // calculate negative flag
    if result.bit(7) {
        alu.flags.set_bit(FLAG_N, true);
    }

    // calculate zero flag
    if result == 0 {
        alu.flags.set_bit(FLAG_Z, true);
    }

    // calculate overflow flag
    if !alu.x.bit(7) && !alu.y.bit(7) && result.bit(7)
        || alu.x.bit(7) && alu.y.bit(7) && !result.bit(7)
    {
        alu.flags.set_bit(FLAG_V, true);
    }

    // calculate carry flag
    if carry.bit(8) {
        alu.flags.set_bit(FLAG_C, true);
    }

    // write result
    alu.result = result;
}

// Subtract without borrow
fn alu_sub(alu: &mut ALU) {
    let result = (Wrapping(alu.x) - Wrapping(alu.y)).0;

    alu.flags = 0;

    // calculate odd flag
    if result.bit(0) {
        alu.flags.set_bit(FLAG_O, true);
    }

    // calculate negative flag
    if result.bit(7) {
        alu.flags.set_bit(FLAG_N, true);
    }

    // calculate zero flag
    if result == 0 {
        alu.flags.set_bit(FLAG_Z, true);
    }

    // calculate overflow flag
    if !alu.x.bit(7) && alu.y.bit(7) && result.bit(7)
        || alu.x.bit(7) && !alu.y.bit(7) && !result.bit(7)
    {
        alu.flags.set_bit(FLAG_V, true);
    }

    // calculate borrow flag
    if alu.x < alu.y {
        alu.flags.set_bit(FLAG_C, true);
    }

    // write result
    alu.result = result;
}

// Subtract with borrow
fn alu_sbb(alu: &mut ALU) {
    let borrow = if alu.test_c() { 1 } else { 0 };

    let result = (Wrapping(alu.x) - Wrapping(alu.y)).0 - borrow;

    alu.flags = 0;

    // calculate odd flag
    if result.bit(0) {
        alu.flags.set_bit(FLAG_O, true);
    }

    // calculate negative flag
    if result.bit(7) {
        alu.flags.set_bit(FLAG_N, true);
    }

    // calculate zero flag
    if result == 0 {
        alu.flags.set_bit(FLAG_Z, true);
    }

    // calculate overflow flag
    if !alu.x.bit(7) && alu.y.bit(7) && result.bit(7)
        || alu.x.bit(7) && !alu.y.bit(7) && !result.bit(7)
    {
        alu.flags.set_bit(FLAG_V, true);
    }

    // calculate borrow flag
    if alu.x < (alu.y + borrow) {
        alu.flags.set_bit(FLAG_C, true);
    }

    // write result
    alu.result = result;
}

// Integer multiply
fn alu_mul(alu: &mut ALU) {
    let result = alu.x as u16 * alu.y as u16;
    let result_low = (result & 0xFF) as u8;
    let result_high = (result >> 8 & 0xFF) as u8;

    alu.flags = 0;

    if result.bit(0) {
        alu.flags.set_bit(FLAG_O, true);
    }

    if result == 0 {
        alu.flags.set_bit(FLAG_Z, true);
    }

    if result_high != 0 {
        alu.flags.set_bit(FLAG_C, true);
    }

    alu.result = result_low;
    alu.res_hi = result_high;
}

// Logical shift left
fn alu_shl(alu: &mut ALU) {
    let result = (alu.x as u32).checked_shl(alu.y as u32).unwrap_or(0);
    let result_low = (result & 0xFF) as u8;
    let result_high = (result >> 8 & 0xFF) as u8;
    let result_veryhigh = (result >> 16 & 0xFFFF) as u16;

    alu.flags = 0;

    if result_low.bit(0) {
        alu.flags.set_bit(FLAG_O, true);
    }

    if result_low == 0 && result_high == 0 {
        alu.flags.set_bit(FLAG_Z, true);
    }

    if result_veryhigh != 0 || (alu.x != 0 && alu.y > 23) {
        alu.flags.set_bit(FLAG_V, true);
    }

    if result.bit(8) {
        alu.flags.set_bit(FLAG_C, true);
    }

    alu.result = result_low;
    alu.res_hi = result_high;
}

// Logical shift right
fn alu_shr(alu: &mut ALU) {
    let result = ((alu.x as u32) << 24)
        .checked_shr(alu.y as u32)
        .unwrap_or(0);
    let result_high = (result >> 24 & 0xFF) as u8;
    let result_low = (result >> 16 & 0xFF) as u8;
    let result_verylow = (result & 0xFFFF) as u16;

    alu.flags = 0;

    if result_high.bit(0) {
        alu.flags.set_bit(FLAG_O, true);
    }

    if result_low == 0 && result_high == 0 {
        alu.flags.set_bit(FLAG_Z, true);
    }

    if result_verylow != 0 || (alu.x != 0 && alu.y > 23) {
        alu.flags.set_bit(FLAG_V, true);
    }

    if result.bit(23) {
        alu.flags.set_bit(FLAG_C, true);
    }

    alu.result = result_high;
    alu.res_hi = result_low;
}

// Logical rotate left
fn alu_rol(alu: &mut ALU) {
    let result = alu.x.rotate_left(alu.y as u32);

    alu.flags = 0;

    if result.bit(0) {
        alu.flags.set_bit(FLAG_O, true);
    }

    if result == 0 {
        alu.flags.set_bit(FLAG_Z, true);
    }

    alu.result = result;
}

// Logical rotate right
fn alu_ror(alu: &mut ALU) {
    let result = alu.x.rotate_right(alu.y as u32);

    alu.flags = 0;

    if result.bit(0) {
        alu.flags.set_bit(FLAG_O, true);
    }

    if result == 0 {
        alu.flags.set_bit(FLAG_Z, true);
    }

    alu.result = result;
}

// Bitwise NOT
fn alu_not(alu: &mut ALU) {
    alu.result = !alu.x;

    alu.flags = 0;

    if alu.result.bit(0) {
        alu.flags.set_bit(FLAG_O, true);
    }

    if alu.result.bit(7) {
        alu.flags.set_bit(FLAG_N, true);
    }

    if alu.result == 0 {
        alu.flags.set_bit(FLAG_Z, true);
    }
}

// Bitwise AND
fn alu_and(alu: &mut ALU) {
    alu.result = alu.x & alu.y;

    alu.flags = 0;

    if alu.result.bit(0) {
        alu.flags.set_bit(FLAG_O, true);
    }

    if alu.result.bit(7) {
        alu.flags.set_bit(FLAG_N, true);
    }

    if alu.result == 0 {
        alu.flags.set_bit(FLAG_Z, true);
    }
}

// Bitwise OR
fn alu_ior(alu: &mut ALU) {
    alu.result = alu.x | alu.y;

    alu.flags = 0;

    if alu.result.bit(0) {
        alu.flags.set_bit(FLAG_O, true);
    }

    if alu.result.bit(7) {
        alu.flags.set_bit(FLAG_N, true);
    }

    if alu.result == 0 {
        alu.flags.set_bit(FLAG_Z, true);
    }
}

// Bitwise XOR
fn alu_xor(alu: &mut ALU) {
    alu.result = alu.x ^ alu.y;

    alu.flags = 0;

    if alu.result.bit(0) {
        alu.flags.set_bit(FLAG_O, true);
    }

    if alu.result.bit(7) {
        alu.flags.set_bit(FLAG_N, true);
    }

    if alu.result == 0 {
        alu.flags.set_bit(FLAG_Z, true);
    }
}
