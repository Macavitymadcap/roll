//! Terms found in parsed notation.
//!
//! Terms are plain data; their values are validated when a `Notation` is built
//! from them.

use std::ops::Neg;

/// Sign before a term, either '+' or '-'.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sign {
    /// The term is added to the total.
    Plus,
    /// The term is subtracted from the total.
    Minus,
}

impl Neg for Sign {
    type Output = Sign;

    fn neg(self) -> Sign {
        match self {
            Sign::Minus => Sign::Plus,
            Sign::Plus => Sign::Minus,
        }
    }
}

/// The kind of term, whether a die (e.g., '2d8'), or a constant number.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TermKind {
    /// A dice expression, e.g., '8d6'.
    Dice {
        /// The number of dice.
        count: u32,
        /// The number of faces on each die.
        sides: u32,
    },
    /// A number
    Constant(u32),
}

/// A single signed term in a notation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Term {
    /// Whether the term is added to or subtracted from the total.
    pub sign: Sign,
    /// Whether the term is a dice expression or a constant.
    pub kind: TermKind,
}

impl Term {
    /// Returns a new `Term` containing a dice expression, defaults to a positive
    /// sign.
    #[must_use]
    pub fn dice(count: u32, sides: u32) -> Self {
        Term {
            sign: Sign::Plus,
            kind: TermKind::Dice { count, sides },
        }
    }

    /// Returns a new `Term` containing a constant, defaults to a positive sign.
    #[must_use]
    pub fn constant(value: u32) -> Self {
        Term {
            sign: Sign::Plus,
            kind: TermKind::Constant(value),
        }
    }

    /// Returns a new `Term` based on the original with the sign negated, e.g.,
    /// '+2d8' becomes '-2d8', and vice versa.
    #[must_use]
    pub fn negated(self) -> Self {
        Term {
            sign: -self.sign,
            ..self
        }
    }
}

impl Neg for Term {
    type Output = Term;

     fn neg(self) -> Term {
        self.negated()
     }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    mod negated {
        use super::*;

        #[rstest]
        #[case::plus_to_minus(Term::constant(2), Term { sign: Sign::Minus, kind: TermKind::Constant(2) })]
        #[case::minus_to_plus(Term { sign: Sign::Minus, kind: TermKind::Constant(2) }, Term::constant(2))]
        fn returns_expected_sign(#[case] term: Term, #[case] expected: Term) {
            let result = term.negated();

            assert_eq!(result, expected);
        }
    }
}
