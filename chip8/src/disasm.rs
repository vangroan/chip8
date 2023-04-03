//! Disassembler.
use std::fmt::{self, Write as FmtWrite};

use crate::{bytecode::*, constants::*};

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
            0x0 => match op_nn(self.bytecode, self.cursor + 1) {
                0xE0 => self.dis_simple(w, "CLS"),
                0xEE => self.dis_simple(w, "RET"),
                _ => self.write_unknown(w),
            },
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
                0x6 => self.dis_xy_op(w, "SHR"),
                0x7 => self.dis_xy_op(w, "SUBN"),
                0xE => self.dis_xy_op(w, "SHL"),
                _ => self.write_unknown(w),
            },
            0x9 => self.dis_xy(w, "SNE"),
            0xA => self.dis_innn(w, "LD"),
            0xB => self.dis_v0_nnn(w, "JP"),
            0xC => self.dis_xnn(w, "RAND"),
            0xD => self.dis_xyn(w, "DRW"),
            0xE => match op_nn(self.bytecode, self.cursor + 1) {
                0x9E => self.dis_x(w, "SKP"),
                0xA1 => self.dis_x(w, "SKNP"),
                _ => self.write_unknown(w),
            },
            0xF => match op_nn(self.bytecode, self.cursor + 1) {
                0x07 => self.dis_xk(w, "LD", "DT"),
                0x0A => self.dis_xk(w, "LD", "K"),
                0x15 => self.dis_kx(w, "LD", "DT"),
                0x18 => self.dis_kx(w, "LD", "ST"),
                0x1E => self.dis_kx(w, "ADD", "I"),
                0x29 => self.dis_kx(w, "LD", "F"),
                0x33 => self.dis_kx(w, "LD", "B"),
                0x55 => self.dis_kx(w, "LD", "[I]"),
                0x65 => self.dis_xk(w, "LD", "[I]"),
                _ => self.write_unknown(w),
            },
            _ => self.write_unknown(w),
        }
    }

    fn write_pc<W: FmtWrite>(&self, w: &mut W) -> fmt::Result {
        write!(w, "0x{:04X}\t", MEM_START + self.cursor)
    }

    fn write_unknown<W: FmtWrite>(&self, w: &mut W) -> fmt::Result {
        self.write_pc(w)?;
        let a = self.bytecode.get(self.cursor).cloned().unwrap_or_default();
        let b = self
            .bytecode
            .get(self.cursor + 1)
            .cloned()
            .unwrap_or_default();
        writeln!(w, "{:02X}{:02X}\tUNKNOWN", a, b)
    }

    fn dis_simple<W: FmtWrite>(&self, w: &mut W, name: &str) -> fmt::Result {
        self.write_pc(w)?;
        writeln!(w, "{name}")
    }

    fn dis_nnn<W: FmtWrite>(&self, w: &mut W, name: &str) -> fmt::Result {
        self.write_pc(w)?;
        let nnn = op_nnn(self.bytecode, self.cursor);
        writeln!(w, "{name}\t0x{nnn:03X}")
    }

    fn dis_innn<W: FmtWrite>(&self, w: &mut W, name: &str) -> fmt::Result {
        self.write_pc(w)?;
        let nnn = op_nnn(self.bytecode, self.cursor);
        writeln!(w, "{name}\tI, 0x{nnn:03X}")
    }

    fn dis_v0_nnn<W: FmtWrite>(&self, w: &mut W, name: &str) -> fmt::Result {
        self.write_pc(w)?;
        let nnn = op_nnn(self.bytecode, self.cursor);
        writeln!(w, "{name}\tv0, 0x{nnn:03X}")
    }

    fn dis_xnn<W: FmtWrite>(&self, w: &mut W, name: &str) -> fmt::Result {
        self.write_pc(w)?;
        let (vx, nn) = op_xnn(self.bytecode, self.cursor);
        writeln!(w, "{name}\tv{vx:x}, 0x{nn:02X}")
    }

    fn dis_xyn<W: FmtWrite>(&self, w: &mut W, name: &str) -> fmt::Result {
        self.write_pc(w)?;
        let (vx, vy, n) = op_xyn(self.bytecode, self.cursor);
        writeln!(w, "{name}\tv{vx:x}, v{vy:x}, 0x{n:02X}")
    }

    fn dis_x<W: FmtWrite>(&self, w: &mut W, name: &str) -> fmt::Result {
        self.write_pc(w)?;
        let vx = op_x(self.bytecode, self.cursor);
        writeln!(w, "{name}\tv{vx:x}")
    }

    fn dis_xy<W: FmtWrite>(&self, w: &mut W, name: &str) -> fmt::Result {
        self.write_pc(w)?;
        let (vx, vy) = op_xy(self.bytecode, self.cursor);
        writeln!(w, "{}\tv{:x}, v{:x}", name, vx, vy)
    }

    fn dis_xk<W: FmtWrite>(&self, w: &mut W, name: &str, k: &str) -> fmt::Result {
        self.write_pc(w)?;
        let vx = op_x(self.bytecode, self.cursor);
        writeln!(w, "{}\tv{:02X}, {}", name, vx, k)
    }

    fn dis_kx<W: FmtWrite>(&self, w: &mut W, name: &str, k: &str) -> fmt::Result {
        self.write_pc(w)?;
        let vx = op_x(self.bytecode, self.cursor);
        writeln!(w, "{name}\t{k}, v{vx:02X}")
    }

    fn dis_xy_op<W: FmtWrite>(&self, w: &mut W, name: &str) -> fmt::Result {
        self.write_pc(w)?;
        let (vx, vy) = op_xy(self.bytecode, self.cursor);
        writeln!(w, "{name}\tv{vx:02X}, v{vy:02X}")
    }
}
