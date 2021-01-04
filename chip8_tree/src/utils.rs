use chip8_core::Chip8Cpu;

#[inline(always)]
pub fn op_code(ram: &[u8], pc: usize) -> u8 {
    (ram[pc] & 0b1111_0000) >> 4
}

#[inline(always)]
pub fn op_nnn(ram: &[u8], pc: usize) -> u16 {
    ((ram[pc] as u16 & 0b1111) << 8) | ram[pc + 1] as u16
}

#[inline(always)]
pub fn op_xnn(ram: &[u8], pc: usize) -> (u8, u8) {
    // Opcode is in upper nibble and needs to be masked out.
    ((ram[pc] & 0b1111), ram[pc + 1])
}

#[inline(always)]
pub fn op_xyn(ram: &[u8], pc: usize) -> (u8, u8, u8) {
    // Opcode is in upper nibble and needs to be masked out.
    let op = ram[pc] & 0b1111;
    let data = ram[pc + 1];

    (op, (data & 0b1111_0000) >> 4, data & 0b1111)
}
