/// Intermediate representation.
use super::register::RegisterId;
use std::fmt;

/// Intermediate representation.
///
/// Instead of the code emitter generating assembly, then
/// going through parsing the assembly into bytecode, we
/// just use an enum.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IR {
    /// `6XNN - LD Vx, NN`
    /// Set Vx to NN.
    SetConst(RegisterId, u8),
    /// `8XY0 - Assign Vx, Vy`
    Assign(RegisterId, RegisterId),
    /// `8XY4 - ADD Vx, Vy`
    /// Adds VY to VX
    MathAdd(RegisterId, RegisterId),
    /// `FX55 - LD [I], Vx`
    /// Save registers V0 to Vx in memory starting at location I.
    SaveMem(u16, RegisterId),
    /// `FX65 - LD Vx, [I]`
    /// Read memory at location I into registers V0 to Vx.
    ReadMem(RegisterId, u16),
}

/// Outputs instruction as assembly.
impl fmt::Display for IR {
    #[rustfmt::skip]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            IR::SetConst(vx, nn) => write!(f, "LD V{}, {}", vx, nn),
            IR::Assign(vx, vy)   => write!(f, "LD, V{}, V{}", vx, vy),
            IR::MathAdd(vx, vy)  => write!(f, "ADD V{}, V{}", vx, vy),
            IR::SaveMem(i, vx)   => write!(f, "LD [{}], V{}", i, vx),
            IR::ReadMem(vx, i)   => write!(f, "LD V{}, [{}]", vx, i),
        }
    }
}

pub fn assemble(irs: &[IR]) -> Vec<u8> {
    // TODO: Convert IR to bytecode
    irs.iter()
        .map(|ir| match ir {
            IR::SetConst(vx, nn) => 0,
            IR::Assign(vx, vy) => 0,
            IR::MathAdd(vx, vy) => 0,
            IR::SaveMem(i, vx) => 0,
            IR::ReadMem(vx, i) => 0,
        })
        .collect()
}
