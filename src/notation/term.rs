use crate::notation::term::TermKind::Constant;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sign {
    Plus,
    Minus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TermKind {
    Dice { count: u32, sides: u32 },
    Constant(u32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Term {
    pub sign: Sign,
    pub kind: TermKind,
}

impl Term {
    pub fn dice(count: u32, sides: u32) -> Self {
        Term {
            sign: Sign::Plus,
            kind: TermKind::Dice { count, sides },
        }
    }

    pub fn constant(value: u32) -> Self {
        Term {
            sign: Sign::Plus,
            kind: Constant(value),
        }
    }

    pub fn negated(self) -> Self {
        Term {
            sign: Sign::Minus,
            kind: self.kind,
        }
    }
}
