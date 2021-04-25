use super::register::RegisterId;
use std::fmt;

/// Intermediate representation.
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
    SaveMem(usize, RegisterId),
    /// `FX65 - LD Vx, [I]`
    /// Read memory at location I into registers V0 to Vx.
    ReadMem(RegisterId, usize),
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
