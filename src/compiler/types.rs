//! Scalar, pointer, string-reference, and global-array types.
//! Storage width is independent of signed interpretation.

use super::ast::TypeKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CType {
    Void,
    Bool,
    U8,
    I8,
    U16,
    I16,
    Str,
    Ptr(PtrType),
    Array(ArrayType),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PtrType {
    pub base: PtrBase,
    pub depth: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PtrBase {
    Bool,
    U8,
    I8,
    U16,
    I16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ArrayType {
    pub elem: ArrayElem,
    pub len: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ArrayElem {
    Bool,
    U8,
    I8,
    U16,
    I16,
}

impl PtrBase {
    pub fn from_ctype(ty: CType) -> Option<Self> {
        Some(match ty {
            CType::Bool => Self::Bool,
            CType::U8 => Self::U8,
            CType::I8 => Self::I8,
            CType::U16 => Self::U16,
            CType::I16 => Self::I16,
            _ => return None,
        })
    }

    pub fn to_ctype(self) -> CType {
        match self {
            Self::Bool => CType::Bool,
            Self::U8 => CType::U8,
            Self::I8 => CType::I8,
            Self::U16 => CType::U16,
            Self::I16 => CType::I16,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Bool => "bool",
            Self::U8 => "u8",
            Self::I8 => "i8",
            Self::U16 => "u16",
            Self::I16 => "i16",
        }
    }
}

impl ArrayElem {
    pub fn from_ctype(ty: CType) -> Option<Self> {
        Some(match ty {
            CType::Bool => Self::Bool,
            CType::U8 => Self::U8,
            CType::I8 => Self::I8,
            CType::U16 => Self::U16,
            CType::I16 => Self::I16,
            _ => return None,
        })
    }

    pub fn to_ctype(self) -> CType {
        match self {
            Self::Bool => CType::Bool,
            Self::U8 => CType::U8,
            Self::I8 => CType::I8,
            Self::U16 => CType::U16,
            Self::I16 => CType::I16,
        }
    }

    pub fn byte_width(self) -> u8 {
        match self {
            Self::Bool | Self::U8 | Self::I8 => 1,
            Self::U16 | Self::I16 => 2,
        }
    }
}

impl PtrType {
    pub fn of(elem: CType) -> Option<Self> {
        match elem {
            CType::Ptr(p) => Some(Self {
                base: p.base,
                depth: p.depth.checked_add(1)?,
            }),
            other => Some(Self {
                base: PtrBase::from_ctype(other)?,
                depth: 1,
            }),
        }
    }

    pub fn pointee(self) -> CType {
        if self.depth > 1 {
            CType::Ptr(Self {
                base: self.base,
                depth: self.depth - 1,
            })
        } else {
            self.base.to_ctype()
        }
    }
}

impl CType {
    pub fn from_ast(kind: &TypeKind) -> Self {
        match kind {
            TypeKind::Void => Self::Void,
            TypeKind::Bool => Self::Bool,
            TypeKind::U8 => Self::U8,
            TypeKind::I8 => Self::I8,
            TypeKind::U16 => Self::U16,
            TypeKind::I16 => Self::I16,
            TypeKind::Str => Self::Str,
            TypeKind::Ptr(inner) => {
                let inner = Self::from_ast(&inner.kind);
                PtrType::of(inner).map(Self::Ptr).unwrap_or(Self::Void)
            }
        }
    }

    pub fn as_str(self) -> String {
        match self {
            Self::Void => "void".to_string(),
            Self::Bool => "bool".to_string(),
            Self::U8 => "u8".to_string(),
            Self::I8 => "i8".to_string(),
            Self::U16 => "u16".to_string(),
            Self::I16 => "i16".to_string(),
            Self::Str => "str".to_string(),
            Self::Ptr(p) => {
                let mut s = p.base.as_str().to_string();
                for _ in 0..p.depth {
                    s = format!("ptr<{s}>");
                }
                s
            }
            Self::Array(a) => format!("{}[{}]", a.elem.to_ctype().as_str(), a.len),
        }
    }

    pub fn is_integer(self) -> bool {
        matches!(self, Self::U8 | Self::I8 | Self::U16 | Self::I16)
    }

    pub fn is_signed(self) -> bool {
        matches!(self, Self::I8 | Self::I16)
    }

    pub fn is_wordish(self) -> bool {
        matches!(self, Self::U16 | Self::I16 | Self::Ptr(_) | Self::Str)
    }

    pub fn pointee(self) -> Option<CType> {
        match self {
            Self::Ptr(p) => Some(p.pointee()),
            _ => None,
        }
    }

    pub fn byte_width(self) -> Option<u8> {
        match self {
            Self::Void | Self::Array(_) => None,
            Self::Bool | Self::U8 | Self::I8 => Some(1),
            Self::U16 | Self::I16 | Self::Str | Self::Ptr(_) => Some(2),
        }
    }

    pub fn data_size(self) -> Option<u16> {
        match self {
            Self::Array(a) => Some(u16::from(a.elem.byte_width()).saturating_mul(a.len)),
            Self::Str => None,
            other => other.byte_width().map(u16::from),
        }
    }

    pub fn bit_width(self) -> Option<u32> {
        self.byte_width().map(|b| u32::from(b) * 8)
    }

    pub fn mask(self) -> u16 {
        match self.byte_width() {
            Some(1) => 0x00FF,
            Some(2) => 0xFFFF,
            _ => 0,
        }
    }

    pub fn min_value(self) -> i32 {
        match self {
            Self::U8
            | Self::Bool
            | Self::U16
            | Self::Void
            | Self::Str
            | Self::Ptr(_)
            | Self::Array(_) => 0,
            Self::I8 => -128,
            Self::I16 => -32768,
        }
    }

    pub fn max_value(self) -> i32 {
        match self {
            Self::Void | Self::Array(_) => 0,
            Self::Bool => 1,
            Self::U8 => 255,
            Self::I8 => 127,
            Self::U16 | Self::Str | Self::Ptr(_) => 65535,
            Self::I16 => 32767,
        }
    }

    pub fn contains_int(self, value: i32) -> bool {
        value >= self.min_value() && value <= self.max_value()
    }

    pub fn wrap_bits(self, value: i32) -> u16 {
        (value as u16) & self.mask()
    }

    pub fn interpret_bits(self, bits: u16) -> i32 {
        let bits = bits & self.mask();
        match self {
            Self::I8 => bits as i8 as i32,
            Self::I16 => bits as i16 as i32,
            Self::Bool => i32::from(bits != 0),
            _ => i32::from(bits),
        }
    }
}
