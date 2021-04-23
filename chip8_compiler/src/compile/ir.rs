use std::fmt;

/// Intermediate representation.
pub enum IR {
    /// `6XNN - LD Vx, NN`
    /// Set Vx to NN.
    SetConst(u8, u8),
    /// `FX55 - LD [I], Vx`
    /// Save registers V0 to Vx in memory starting at location I.
    SaveMem(usize, u8),
    /// `FX65 - LD Vx, [I]`
    /// Read memory at location I into registers V0 to Vx.
    ReadMem(u8, usize),
}

/// Outputs instruction as assembly.
impl fmt::Display for IR {
    #[rustfmt::skip]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            IR::SetConst(vx, nn) => write!(f, "LD V{}, {}", vx, nn),
            IR::SaveMem(i, vx)   => write!(f, "LD [{}], V{}", i, vx),
            IR::ReadMem(vx, i)   => write!(f, "LD V{}, [{}]", vx, i),
        }
    }
}
