//! Simulator using static dispatch.
use crate::{
    nodes::{self, OpCode},
    utils::*,
};
use chip8_core::{prelude::*, MEM_START};

pub struct StaticSimulator {
    nodes: Vec<OpCode>,
}

impl StaticSimulator {
    pub fn new() -> Self {
        Self { nodes: vec![] }
    }

    fn compile(&mut self, cpu: &Chip8Cpu) {
        self.nodes.clear();

        self.nodes.extend((0..MEM_START).map(|_| OpCode::NoOp));

        for (i, op) in cpu.ram.iter().enumerate().skip(MEM_START).step_by(2) {
            // We ony compile until we encounter 0, which in this implementation is not a valid opcode.
            if *op == 0 {
                println!("Compile done");
                return;
            }

            let code = op_code(&cpu.ram, i);
            match code {
                0x1 => {
                    let address = op_nnn(&cpu.ram, i);
                    self.nodes.push(OpCode::Jump(nodes::Jump { address }));
                }
                0x3 => {
                    let (vx, nn) = op_xnn(&cpu.ram, i);
                    self.nodes.push(OpCode::SkipEqual(nodes::SeNode { vx, nn }))
                }
                0x6 => {
                    let (vx, nn) = op_xnn(&cpu.ram, i);
                    self.nodes.push(OpCode::Load(nodes::LdNode { vx, nn }))
                }
                0x7 => {
                    let (vx, nn) = op_xnn(&cpu.ram, i);
                    self.nodes.push(OpCode::Add(nodes::AddNode { vx, nn }))
                }
                0xA => {
                    let nnn = op_nnn(&cpu.ram, i);
                    self.nodes.push(OpCode::LoadAddress(nodes::LdINode { nnn }));
                }
                0xC => {
                    let (vx, nn) = op_xnn(&cpu.ram, i);
                    self.nodes.push(OpCode::Random(nodes::RndNode { vx, nn }))
                }
                0xD => {
                    let (vx, vy, n) = op_xyn(&cpu.ram, i);
                    self.nodes.push(OpCode::Draw(nodes::DrwNode { vx, vy, n }))
                }
                _ => panic!("Can not compile unknown opcode {:X}", code),
            }
        }
    }
}

impl Interpreter for StaticSimulator {
    fn on_load(&mut self, cpu: &Chip8Cpu) {
        self.compile(cpu);
    }

    fn execute(&self, _cpu: &mut Chip8Cpu) {
        todo!("Execute simulator")
    }
}
