//! Disassembler.
use std::fmt::{self, Write as FmtWrite};

use crate::bytecode::*;

pub struct Disassembler<'a> {
    bytecode: &'a [u8],
    cursor: usize,
}

impl<'a> Disassembler<'a> {
    pub fn new(bytecode: &'a [u8]) -> Self {
        Self {
            bytecode,
            cursor: 0,
        }
    }

    pub fn print_bytecode(&mut self) {
        let mut s = String::new();
        while self.cursor < self.bytecode.len() {
            self.disassemble(&mut s).expect("Failed to print bytecode");
            self.cursor += 2;
        }
        self.cursor = 0;

        println!("{}", s);
    }

    /// Write a single instruction to the given writer.
    pub fn disassemble<W: FmtWrite>(&self, w: &mut W) -> fmt::Result {
        let code = op_code(&self.bytecode, self.cursor);

        match code {
            0x00E0 => self.dis_simple(w, "Clear Screen"),
            0x00EE => self.dis_simple(w, "Return"),
            0x1 => self.dis_nnn(w, "Jump"),
            0x2 => self.dis_nnn(w, "Call"),
            0x3 => self.dis_xnn(w, "Skip Equal"),
            0x4 => self.dis_xnn(w, "Skip Not-equal"),
            0x5 => self.dis_xy(w, "Skip Equal"),
            0x6 => self.dis_xnn(w, "Load"),
            0x7 => self.dis_xnn(w, "Add"),
            0x8 => match op_n(&self.bytecode, self.cursor) {
                0x0 => self.dis_xy_op(w, "Load"),
                0x1 => self.dis_xy_op(w, "OR"),
                0x2 => self.dis_xy_op(w, "AND"),
                0x3 => self.dis_xy_op(w, "XOR"),
                0x4 => self.dis_xy_op(w, "Add"),
                0x5 => self.dis_xy_op(w, "Subtract"),
                _ => Ok(()),
            },
            _ => Ok(()),
        }
    }

    fn dis_simple<W: FmtWrite>(&self, w: &mut W, name: &str) -> fmt::Result {
        writeln!(w, "{:04X}: {}", self.cursor, name)
    }

    fn dis_nnn<W: FmtWrite>(&self, w: &mut W, name: &str) -> fmt::Result {
        let nnn = op_nnn(&self.bytecode, self.cursor);
        writeln!(w, "{:04X}: {} {:03X}", self.cursor, name, nnn)
    }

    fn dis_xnn<W: FmtWrite>(&self, w: &mut W, name: &str) -> fmt::Result {
        let (vx, nn) = op_xnn(&self.bytecode, self.cursor);
        writeln!(w, "{:04X}: {} V{:02X} {:02X}", self.cursor, name, vx, nn)
    }

    fn dis_xy<W: FmtWrite>(&self, w: &mut W, name: &str) -> fmt::Result {
        let (vx, vy) = op_xy(&self.bytecode, self.cursor);
        writeln!(w, "{:04X}: {} V{:02X} V{:02X}", self.cursor, name, vx, vy)
    }

    fn dis_xy_op<W: FmtWrite>(&self, w: &mut W, name: &str) -> fmt::Result {
        let (vx, vy) = op_xy(&self.bytecode, self.cursor);
        writeln!(w, "{:04X}: {} V{:02X} V{:02X}", self.cursor, name, vx, vy)
    }
}
