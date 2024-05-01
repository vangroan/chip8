//! Rewrite of disassembler which is more structured in its analyses.
use std::collections::{HashMap, HashSet};
use std::fmt::{self, Write as FmtWrite};
use std::iter::Enumerate;

use crate::bytecode::{op_code, opcodes};
use crate::constants::{Address, MEM_SIZE, MEM_START};

use super::ir::{Instr, Op};

pub struct DisassemblerV2<'a> {
    /// Original bytecode input.
    bytecode: &'a [u8],
    cursor: usize,
    /// Resulting instructions are organised into blocks.
    blocks: Vec<Block>,
    /// Mapping of memory addresses to instructions.
    ///
    /// Should map to blocks?
    addresses: Box<[usize]>,
    /// The current subroutine being analysed.
    subroutine: Block,
    /// Stack of subroutines.
    subroutines: Vec<Block>,
    /// Monotonically increasing block counter.
    block_id: usize,
    /// Monotonically increasing subroutine counter.
    subroutine_id: usize,
    instructions: Vec<Instr>,
    /// Mapping of bytecode indices to labels.
    labels: HashMap<usize, ()>,
    /// Bytecode indices that are candidates for data blocks.
    data_blocks: HashSet<usize>,
    errors: (),
    warnings: (),
}

struct Block {
    /// Semantic kind of block, relevant to some behaviour
    /// in the disassembler.
    kind: BlockKind,
    /// Human-readable label intended for output.
    label: String,
    /// Resulting instructions.
    ops: Vec<Instr>,
}

#[derive(Debug)]
enum BlockKind {
    /// Simplest control flow block, used for loops and conditionals.
    Simple,
    /// Callable via the `CALL addr` instruction
    /// and should ideally end with a `RET` instruction.
    Subroutine,
    /// Data intended to be drawn to screen.
    Sprite,
}

impl<'a> DisassemblerV2<'a> {
    pub fn new(bytecode: &'a [u8]) -> Self {
        // Starts with an implicit top level subroutine.
        let subroutine = Block {
            kind: BlockKind::Subroutine,
            label: "__main__".into(),
            ops: vec![],
        };

        Self {
            bytecode,
            cursor: 0,
            blocks: vec![],
            addresses: Vec::new().into_boxed_slice(),
            subroutine,
            subroutines: vec![],
            block_id: 0,
            subroutine_id: 0,
            instructions: vec![],
            labels: HashMap::new(),
            data_blocks: HashSet::new(),
            errors: (),
            warnings: (),
        }
    }

    pub fn disassemble<W: FmtWrite>(&mut self, w: &mut W) -> fmt::Result {
        for mut instr in Decoder::new(self.bytecode.iter().cloned()) {
            // TODO: Label jump destinations
            // TODO: Mark data blocks
            match instr.op {
                Op::Load_Address { address } => {
                    self.data_blocks.insert((address as usize) - MEM_START);
                }
                Op::Draw { vx, vy, n } => {
                    // TODO: Mark all rows as data
                }
                _ => { /* Do Nothing */ }
            }

            if self.data_blocks.contains(&instr.index) {
                instr.op = Op::Data;
            }

            self.instructions.push(instr);
        }

        // Format instructions
        for instr in &self.instructions {
            writeln!(
                w,
                "0x{:04X} {:04X} {} {:?}",
                instr.addr,
                instr.bytecode(),
                instr.repr(),
                instr.op
            )?;
        }

        Ok(())
    }

    fn address(&self) -> Address {
        MEM_START as u16 + self.cursor as u16
    }

    fn op(&self) -> [u8; 2] {
        [self.bytecode[self.cursor], self.bytecode[self.cursor + 1]]
    }

    fn bump(&mut self) {
        self.cursor += 2;
    }
}

struct Decoder<I> {
    iter: Enumerate<I>,
}

impl<I: Iterator> Decoder<I> {
    fn new(iter: I) -> Self {
        Self {
            iter: iter.enumerate(),
        }
    }
}

impl<I> Decoder<I> {
    #[inline(always)]
    fn decode(&self, bytecode: [u8; 2]) -> Op {
        let [a, b] = bytecode;
        let op = a >> 4; // 0xF000
        let vx = a & 0xF; // 0x0F00
        let vy = b >> 4; // 0x00F0
        let n = b & 0xF; // 0x000F
        let nn = b; // 0x00FF
        let nnn = (((a as u16) & 0xF) << 8) | b as u16; // 0x0FFF

        match op {
            // Miscellaneous instructions identified by nn
            0x0 => {
                match nn {
                    0x0 => Op::NoOp,
                    // 00E0 (CLS)
                    //
                    // Clear display
                    0xE0 => Op::ClearScreen,
                    // 00EE (RET)
                    //
                    // Return from a subroutine.
                    0xEE => Op::Return,
                    _ => Op::Unknown,
                }
            }
            // 1nnn (JP addr)
            //
            // Jump to address.
            0x1 => Op::JumpAddress { address: nnn },
            // 2nnn (CALL addr)
            //
            // Call subroutine at NNN.
            0x2 => Op::Call { address: nnn },
            // 3xnn (SE Vx, byte)
            //
            // Skip the next instruction if register VX equals value NN.
            0x3 => Op::Skip_Eq_Byte { vx, nn },
            // 4xnn (SNE Vx, byte)
            //
            // Skip the next instruction if register VX does not equal value NN.
            0x4 => Op::Skip_NotEq_Byte { vx, nn },
            // 5xy0 (SE Vx, Vy)
            //
            // Skip the next instruction if register VX equals value VY.
            0x5 => Op::Skip_Eq { vx, vy },
            // 6xnn (LD Vx, byte)
            //
            // Set register VX to value NN.
            0x6 => Op::Load_Byte { vx, nn },
            // 7xnn (ADD Vx, byte)
            //
            // Add byte to the value in register `Vx`, store the result in `Vx`.
            // Carry bit is not set.
            0x7 => Op::Add_Byte { vx, nn },
            // Arithmetic.
            0x8 => match n {
                0x0 => Op::Load_Vx_Vy { vx, vy },
                0x1 => Op::Or_Vx_Vy { vx, vy },
                0x2 => Op::And_Vx_Vy { vx, vy },
                0x3 => Op::Xor_Vx_Vy { vx, vy },
                0x4 => Op::Add_Vx_Vy { vx, vy },
                0x5 => Op::Sub_Vx_Vy { vx, vy },
                0x6 => Op::ShiftRight { vx },
                0x7 => Op::SubReverse_Vx_Vy { vx, vy },
                0x8 => Op::ShiftLeft { vx },
                _ => Op::Unknown,
            },
            /// Annn (LD I, addr)
            //
            // Set address register I to value NNN.
            0xA => Op::Load_Address { address: nnn },
            // Bnnn (JP V0, addr)
            //
            // Jump to location nnn + V0.
            0xB => Op::Jump_Vx { address: nnn },
            // Cxnn (RND Vx, byte)
            //
            // Generate random number.
            0xC => Op::Random { vx, nn },
            // Dxyn (DRW Vx, Vy, byte)
            //
            // Draw sprite to the display buffer.
            0xD => Op::Draw { vx, vy, n },
            _ => Op::Unknown,
        }
    }
}

impl<I: Iterator<Item = u8>> Iterator for Decoder<I> {
    type Item = Instr;

    fn next(&mut self) -> Option<Instr> {
        let (index, a) = self.iter.next()?;
        let (_, b) = self.iter.next()?;

        let op = self.decode([a, b]);

        let addr = MEM_START + index;
        if addr >= MEM_SIZE {
            panic!("program size exceeds chip-8 memory limit");
        }

        Some(Instr {
            addr: addr as Address,
            index,
            bytes: [a, b],
            op,
        })
    }
}
