//! Scalar types: storage width is independent of signed interpretation.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CType {
    Void,
    Bool,
    U8,
    I8,
    U16,
    I16,
}

impl CType {
    pub fn from_ast(kind: super::ast::TypeKind) -> Self {
        match kind {
            super::ast::TypeKind::Void => Self::Void,
            super::ast::TypeKind::Bool => Self::Bool,
            super::ast::TypeKind::U8 => Self::U8,
            super::ast::TypeKind::I8 => Self::I8,
            super::ast::TypeKind::U16 => Self::U16,
            super::ast::TypeKind::I16 => Self::I16,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Void => "void",
            Self::Bool => "bool",
            Self::U8 => "u8",
            Self::I8 => "i8",
            Self::U16 => "u16",
            Self::I16 => "i16",
        }
    }

    pub fn is_integer(self) -> bool {
        matches!(self, Self::U8 | Self::I8 | Self::U16 | Self::I16)
    }

    pub fn is_signed(self) -> bool {
        matches!(self, Self::I8 | Self::I16)
    }

    pub fn byte_width(self) -> Option<u8> {
        match self {
            Self::Void => None,
            Self::Bool | Self::U8 | Self::I8 => Some(1),
            Self::U16 | Self::I16 => Some(2),
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
            Self::U8 | Self::Bool | Self::U16 | Self::Void => 0,
            Self::I8 => -128,
            Self::I16 => -32768,
        }
    }

    pub fn max_value(self) -> i32 {
        match self {
            Self::Void => 0,
            Self::Bool => 1,
            Self::U8 => 255,
            Self::I8 => 127,
            Self::U16 => 65535,
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
