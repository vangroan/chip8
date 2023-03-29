use log::trace;

/// Helpers for extracting data from opcodes.
use crate::constants::*;

#[rustfmt::skip]
pub mod opcodes {
    /// 00E0 (CLS)
    pub const CLS: u8        = 0xE0;
    /// 00EE (RET)
    pub const RET: u8        = 0xEE;
    /// 1nnn (JP addr)
    pub const JP_ADDR: u8    = 0x1;
    /// 2nnn (CALL addr)
    pub const CALL_ADDR: u8  = 0x2;
    /// 3xnn (SE Vx, byte)
    pub const SE_VX_NN: u8   = 0x3;
    /// 4xnn (SNE Vx, byte)
    pub const SNE_VX_NN: u8  = 0x4;
    /// 5xy0 (SE Vx, Vy)
    pub const SE_VX_VY: u8   = 0x5;
    /// 6xnn (LD Vx, byte)
    pub const LD_VX_NN: u8   = 0x6;
    /// 7xnn (ADD Vx, byte)
    pub const ADD_VX_NN: u8  = 0x7;
    /// 8xy0 (LD Vx, byte)
    pub const LD_VX_VY: [u8; 2]    = [0x8, 0x0];
    /// 8xy1 (OR Vx, Vy)
    pub const OR_VX_VY: [u8; 2]    = [0x8, 0x1];
    /// 8xy2 (AND Vx, Vy)
    pub const AND_VX_VY: [u8; 2]   = [0x8, 0x2];
    /// 8xy3 (XOR Vx, Vy)
    pub const XOR_VX_VY: [u8; 2]   = [0x8, 0x3];
    /// 8xy4 (ADD Vx, Vy)
    pub const ADD_VX_VY: [u8; 2]   = [0x8, 0x4];
    /// 8xy5 (SUB Vx, Vy)
    pub const SUB_VX_VY: [u8; 2]   = [0x8, 0x5];
    /// 8xy6 (SHR Vx {, Vy})
    pub const SHR_VX_VY: [u8; 2]   = [0x8, 0x6];
    /// 8xy7 (SUBN Vx, Vy)
    pub const SUBN_VX_VY: [u8; 2]  = [0x8, 0x7];
    /// 8xyE (SHL Vx {, Vy})
    pub const SHL_VX_VY: [u8; 2]   = [0x8, 0xE];
    /// 9xy0 (SNE Vx, Vy)
    pub const SNE_VX_VY: u8   = 0x9;
    /// Annn (LD I, addr)
    pub const LD_I_NNN: u8    = 0xA;
    /// Bnnn (JP V0, addr)
    pub const JP_V0_ADDR: u8  = 0xB;
    /// Cxnn (RND Vx, byte)
    pub const RND_VX_NN: u8   = 0xC;
    /// Dxyn (DRW Vx, Vy, byte)
    pub const DRW_VX_VY_N: u8 = 0xD;
    /// Ex9E (SKP Vx)
    pub const SKP_VX: [u8; 2]       = [0xE, 0x9E];
    /// ExA1 (SKNP Vx)
    pub const SKNP_VX: [u8; 2]      = [0xE, 0xA1];
    /// Fx07 (LD Vx, DT)
    pub const LD_VX_DT: [u8; 2]     = [0xF, 0x07];
    /// Fx0A (LD Vx, K)
    pub const LD_VX_K: [u8; 2]      = [0xF, 0x0A];
    /// Fx15 (LD DT, Vx)
    pub const LD_DT_VX: [u8; 2]     = [0xF, 0x15];
    /// Fx18 (LD ST, Vx)
    pub const LD_ST_VX: [u8; 2]     = [0xF, 0x18];
    /// Fx1E (ADD I, Vx)
    pub const ADD_I_VX: [u8; 2]     = [0xF, 0x1E];
    /// Fx29 (LD F, Vx)
    pub const LD_F_VX: [u8; 2]      = [0xF, 0x29];
    /// Fx33 (LD B, Vx)
    pub const LD_B_VX: [u8; 2]      = [0xF, 0x33];
    /// Fx55 (LD [I], Vx)
    pub const LD_ARR_VX: [u8; 2]    = [0xF, 0x55];
    /// Fx65 (LD Vx, [I])
    pub const LD_VX_ARR: [u8; 2]    = [0xF, 0x65];
}

/// Returns true if the program can fit in VM memory.
#[inline]
pub(crate) fn check_program_size(program: &[u8]) -> bool {
    program.len() <= (MEM_SIZE - MEM_START)
}

/// Extract opcode from the buffer at the cursor.
#[inline(always)]
pub fn op_code(bytecode: &[u8], cursor: usize) -> u8 {
    (bytecode[cursor] & 0b1111_0000) >> 4
}

/// Extract operand NNN from the buffer at the cursor.
#[inline(always)]
pub fn op_nnn(bytecode: &[u8], cursor: usize) -> u16 {
    ((bytecode[cursor] as u16 & 0b1111) << 8) | bytecode[cursor + 1] as u16
}

/// Extract operand NN from the buffer at the cursor.
#[inline(always)]
pub fn op_nn(bytecode: &[u8], cursor: usize) -> u8 {
    bytecode[cursor]
}

/// Extract operands VX and NN from the buffer at the cursor.
#[inline(always)]
pub fn op_xnn(bytecode: &[u8], cursor: usize) -> (u8, u8) {
    // Opcode is in upper nibble and needs to be masked out.
    ((bytecode[cursor] & 0b1111), bytecode[cursor + 1])
}

/// Extract operands VX and NN from the buffer at the cursor.
#[inline(always)]
pub fn op_xyn(bytecode: &[u8], cursor: usize) -> (u8, u8, u8) {
    // Opcode is in upper nibble and needs to be masked out.
    let x = bytecode[cursor] & 0b1111;
    let next = bytecode[cursor + 1];
    let y = (next & 0b1111_0000) >> 4;
    let n = next & 0b1111;

    (x, y, n)
}

/// Extract operands VX and VY from the buffer at the cursor.
#[inline(always)]
pub fn op_xy(bytecode: &[u8], cursor: usize) -> (u8, u8) {
    // Opcode is in upper nibble and needs to be masked out.
    let x = bytecode[cursor] & 0b1111;

    // Lower nibble is unused.
    let y = (bytecode[cursor + 1] & 0b1111_0000) >> 4;

    (x, y)
}

/// Extract operands VX, VY and N from the buffer at the cursor.
#[inline(always)]
pub fn op_x(bytecode: &[u8], cursor: usize) -> u8 {
    // Opcode is in upper nibble and needs to be masked out.
    bytecode[cursor] & 0b1111
}

/// Extract operand N from the buffer at the cursor.
#[inline(always)]
pub fn op_n(bytecode: &[u8], cursor: usize) -> u8 {
    let data = bytecode[cursor + 1];
    data & 0b1111
}

// Encode a a bare instruction, which has no arguments.
pub fn encode_bare(opcode: u8) -> [u8; 2] {
    trace!("encode 0x{:03X}", opcode);
    [0, opcode]
}

pub fn encode_xnn(opcode: u8, vx: u8, nn: u8) -> [u8; 2] {
    trace!("encode {:02X} {:02X}, {:02X}", opcode, vx, nn);
    [(opcode << 4) | (vx & 0xF), nn]
}

pub fn encode_xyn(opcode: u8, vx: u8, vy: u8, n: u8) -> [u8; 2] {
    trace!("encode {:02X} {:02X}, {:02X}, {:02X}", opcode, vx, vy, n);
    [(opcode << 4) | (vx & 0xF), (vy << 4) | (n & 0xF)]
}

pub fn encode_nnn(opcode: u8, nnn: u16) -> [u8; 2] {
    trace!("encode {:02X} 0x{:03X}", opcode, nnn);
    let part1 = ((nnn & 0b1111_0000_0000) >> 8) as u8;
    let part2 = (nnn & 0b1111_1111) as u8;
    [(opcode << 4) | part1, part2]
}
