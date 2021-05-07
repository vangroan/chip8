use super::Register;
use crate::parsing::Ident;
use std::{collections::HashMap, convert::TryFrom, fmt};

#[derive(Debug)]
pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
    /// Static type known at compile time.
    pub ty: ValueType,
}

impl Symbol {
    /// Build a unique function signature string.
    #[inline]
    pub fn to_func_sig<W: fmt::Write>(&self, f: &mut W) -> fmt::Result {
        match self.kind {
            SymbolKind::Function(ref args) => {
                write!(f, "{}", &self.name)?;
                write!(f, "(")?;

                for (idx, arg_ty) in args.iter().enumerate() {
                    if idx == args.len() - 1 {
                        write!(f, "{}", arg_ty)?;
                    } else {
                        write!(f, "{},", arg_ty)?;
                    }
                }

                write!(f, ")")?;
                write!(f, "->")?;
                write!(f, "{}", self.ty)?;

                Ok(())
            }
            _ => Err(fmt::Error),
        }
    }
}

/// Static value type of the symbol known at compile time.
///
/// Compiler is pretty simple so no fancy type system.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ValueType {
    /// The nothing type.
    Void,
    /// One and only number type.
    U8,
    /// Zero and one can cover bools.
    Bool,
    /// Multiple values packed together.
    Record,
    /// 12-bit memory address.
    /// FIXME: Is this even possible? We can't fo arithmetic with 12-bit values.
    Pointer,
}

impl TryFrom<&Ident> for ValueType {
    type Error = ();

    fn try_from(ident: &Ident) -> Result<Self, Self::Error> {
        match ident.name.as_str() {
            "void" => Ok(ValueType::Void),
            "u8" => Ok(ValueType::U8),
            "bool" => Ok(ValueType::Bool),
            _ => Err(()),
        }
    }
}

impl fmt::Display for ValueType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ValueType::Void => write!(f, "void"),
            ValueType::U8 => write!(f, "u8"),
            ValueType::Bool => write!(f, "bool"),
            _ => todo!("type cannot be displayed yet"),
        }
    }
}

#[derive(Debug)]
pub enum SymbolKind {
    /// Constants have a value fixed at compile time.
    Const,
    /// Variables can change value during runtime.
    /// Assigned a register of its own, unique in its
    /// containing function scope.
    Var,
    /// Function with list of arguments.
    Function(Vec<ValueType>),
}

impl SymbolKind {
    #[inline]
    pub fn is_func(&self) -> bool {
        matches!(self, SymbolKind::Function(_))
    }
}

#[derive(Debug)]
pub enum SymbolScope {
    Global,
    Local,
    Parameter,
}

#[derive(Debug, Default)]
pub struct SymbolTable {
    pub var_count: usize,
    /// Constants in the global scope.
    pub consts: HashMap<String, Symbol>,
    /// Variables in the global scope.
    pub vars: HashMap<String, Symbol>,
    /// Functions in a compilation unit.
    pub funcs: HashMap<String, Symbol>,
    pub scopes: Vec<SymbolTable>,
}
