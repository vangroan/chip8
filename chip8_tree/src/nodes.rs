//! Opcode implementations.
use crate::SimNode;
use chip8_core::{prelude::*, Address, SCREEN_BUF_COUNT, SCREEN_HEIGHT, SCREEN_WIDTH};
use rand::{prelude::*, thread_rng};

pub enum OpCode {
    NoOp,
    Jump(Jump),
    SkipEqual(SeNode),
    Load(LdNode),
    Add(AddNode),
    LoadAddress(LdINode),
    Random(RndNode),
    Draw(DrwNode),
}

/// 1NNN
/// Jump to address.
pub struct Jump {
    pub address: Address,
}

impl SimNode for Jump {
    #[inline]
    fn exec(&self, cpu: &mut Chip8Cpu) {
        cpu.pc = self.address as usize;
    }
}

/// 3XNN
/// Skip the next instruction if register VX equals value NN.
pub struct SeNode {
    pub vx: u8,
    pub nn: u8,
}

impl SimNode for SeNode {
    #[inline]
    fn exec(&self, cpu: &mut Chip8Cpu) {
        if cpu.registers[self.vx as usize] == self.nn {
            cpu.pc += 4;
        } else {
            cpu.pc += 2;
        }
    }
}

/// 6XNN
/// Set register VX to value NN.
pub struct LdNode {
    pub vx: u8,
    pub nn: u8,
}

impl SimNode for LdNode {
    #[inline]
    fn exec(&self, cpu: &mut Chip8Cpu) {
        cpu.registers[self.vx as usize] = self.nn;
        cpu.pc += 2;
    }
}

/// 7XNN
/// Add value NN to register VX. Carry flag is not set.
pub struct AddNode {
    pub vx: u8,
    pub nn: u8,
}

impl SimNode for AddNode {
    #[inline]
    fn exec(&self, cpu: &mut Chip8Cpu) {
        cpu.registers[self.vx as usize] += self.nn;
        cpu.pc += 2;
    }
}

/// ANNN
/// Set address register I to value NNN.
pub struct LdINode {
    pub nnn: u16,
}

impl SimNode for LdINode {
    #[inline]
    fn exec(&self, cpu: &mut Chip8Cpu) {
        cpu.address = self.nnn;
        cpu.pc += 2;
    }
}

/// CXNN
/// Set register VX to the result of bitwise AND between a random number and NN.
pub struct RndNode {
    pub vx: u8,
    pub nn: u8,
}

impl SimNode for RndNode {
    #[inline]
    fn exec(&self, cpu: &mut Chip8Cpu) {
        cpu.registers[self.vx as usize] = self.nn & thread_rng().gen::<u8>();
        cpu.pc += 2;
    }
}

/// DXYN
///
/// Draw sprite in the display buffer, at coordinate as per registers VX and VY.
/// Sprite is encoded as 8 pixels wide, N+1 pixels high, stored in bits located in
/// memory pointed to by address register I.
///
/// If the sprite is drawn outside of the display area, it is wrapped around to the other side.
///
/// If the drawing operation erases existing pixels in the display buffer, register VF is set to
/// 1, and set to 0 if no display bits are unset. This is used for collision detection.
pub struct DrwNode {
    pub vx: u8,
    pub vy: u8,
    pub n: u8,
}

impl SimNode for DrwNode {
    #[inline]
    fn exec(&self, cpu: &mut Chip8Cpu) {
        let (x, y) = (
            cpu.registers[self.vx as usize] as usize,
            cpu.registers[self.vy as usize] as usize,
        );
        let mut is_erased = false;

        // Iteration from pointer in address register I to number of rows specified by opcode value N.
        for (r, row) in cpu
            .ram
            .iter()
            .skip(cpu.address as usize)
            .take(self.n as usize)
            .enumerate()
        {
            // Each row is 8 bits representing the 8 pixels of the sprite.
            for c in 0..8 {
                let d = ((x + c) % SCREEN_WIDTH) + ((y + r) % SCREEN_HEIGHT) * SCREEN_WIDTH;

                let old_px = cpu.screen[d];
                let new_px = old_px ^ ((row >> (7 - c) & 0x1) == 1);

                // XOR erases a pixel when both the old and new values are both 1.
                is_erased |= old_px && new_px;

                // Write to display buffer
                cpu.screen[d] = new_px;
            }
        }

        // If a pixel was erased, then a collision occurred.
        cpu.registers[0xF] = is_erased as u8;
        cpu.pc += 2;
    }
}
