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
        let code = op_code(self.bytecode, self.cursor);

        match code {
            0x00E0 => self.dis_simple(w, "CLS"),
            0x00EE => self.dis_simple(w, "RET"),
            0x1 => self.dis_nnn(w, "JP"),
            0x2 => self.dis_nnn(w, "CALL"),
            0x3 => self.dis_xnn(w, "SE"),
            0x4 => self.dis_xnn(w, "SNE"),
            0x5 => self.dis_xy(w, "SE"),
            0x6 => self.dis_xnn(w, "LD"),
            0x7 => self.dis_xnn(w, "ADD"),
            0x8 => match op_n(self.bytecode, self.cursor) {
                0x0 => self.dis_xy_op(w, "LD"),
                0x1 => self.dis_xy_op(w, "OR"),
                0x2 => self.dis_xy_op(w, "AND"),
                0x3 => self.dis_xy_op(w, "XOR"),
                0x4 => self.dis_xy_op(w, "ADD"),
                0x5 => self.dis_xy_op(w, "SUB"),
                _ => self.write_unknown(w),
            },
            0xA => self.dis_innn(w, "LD"),
            0xC => self.dis_xnn(w, "RND"),
            0xD => self.dis_xyn(w, "DRW"),
            _ => self.write_unknown(w),
        }
    }

    fn write_pc<W: FmtWrite>(&self, w: &mut W) -> fmt::Result {
        write!(w, "0x{:04X}\t", self.cursor)
    }

    fn write_unknown<W: FmtWrite>(&self, w: &mut W) -> fmt::Result {
        self.write_pc(w)?;
        writeln!(w, "0x{:02X}\tUNKNOWN", op_code(self.bytecode, self.cursor))
    }

    fn dis_simple<W: FmtWrite>(&self, w: &mut W, name: &str) -> fmt::Result {
        self.write_pc(w)?;
        writeln!(w, "{}", name)
    }

    fn dis_nnn<W: FmtWrite>(&self, w: &mut W, name: &str) -> fmt::Result {
        self.write_pc(w)?;
        let nnn = op_nnn(self.bytecode, self.cursor);
        writeln!(w, "{}\t0x{:03X}", name, nnn)
    }

    fn dis_innn<W: FmtWrite>(&self, w: &mut W, name: &str) -> fmt::Result {
        self.write_pc(w)?;
        let nnn = op_nnn(self.bytecode, self.cursor);
        writeln!(w, "{}\tI, 0x{:03X}", name, nnn)
    }

    fn dis_xnn<W: FmtWrite>(&self, w: &mut W, name: &str) -> fmt::Result {
        self.write_pc(w)?;
        let (vx, nn) = op_xnn(self.bytecode, self.cursor);
        writeln!(w, "{}\tV{:02X}, {:02X}", name, vx, nn)
    }

    fn dis_xyn<W: FmtWrite>(&self, w: &mut W, name: &str) -> fmt::Result {
        self.write_pc(w)?;
        let (vx, vy, n) = op_xyn(self.bytecode, self.cursor);
        writeln!(w, "{}\tV{:02X}, V{:02X}, {:02X}", name, vx, vy, n)
    }

    fn dis_xy<W: FmtWrite>(&self, w: &mut W, name: &str) -> fmt::Result {
        self.write_pc(w)?;
        let (vx, vy) = op_xy(self.bytecode, self.cursor);
        writeln!(w, "{}\tV{:02X}, V{:02X}", name, vx, vy)
    }

    fn dis_xy_op<W: FmtWrite>(&self, w: &mut W, name: &str) -> fmt::Result {
        self.write_pc(w)?;
        let (vx, vy) = op_xy(self.bytecode, self.cursor);
        writeln!(w, "{}\tV{:02X}, V{:02X}", name, vx, vy)
    }
}
