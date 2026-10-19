//! Parsing and validation of dice notation.
//!
//! Notation takes the form `XdY?Z`: an optional number of dice `X`, a die
//! with `Y` sides, and an optional modifier made of an operator (`+`, `-`,
//! `*` or `/`) and a value `Z`. Whitespace between the parts is ignored,
//! and `d` may be upper or lower case.
//!
//! A [`Notation`] can only be created through [`Notation::try_new`] or by
//! parsing, so every notation built from user input is within bounds.

use std::fmt;
use std::str::FromStr;

mod error;
mod modifier;
mod scale;
mod term;
mod token;

pub use error::NotationError;
pub use modifier::{Modifier, Operator};
pub use scale::{Scale, ScaleOperator};
pub use term::{Sign, Term, TermKind};

/// The largest value allowed for each number of sides, each constant and
/// scale value.
pub const MAX_VALUE: u32 = 1_000;

/// The total number of dice across all terms.
pub const MAX_DICE: u32 = 1_000;

/// The number of terms, dice and constants together.
pub const MAX_TERMS: usize = 20;

/// A validated dice expression, such as `2d6+3`.
///
/// Create one by parsing a string or with [`Notation::try_new`].
///
/// # Examples
///
/// ```
/// use rust_roll::notation::Notation;
///
/// let notation: Notation = "2d6+3".parse().unwrap();
///
/// assert_eq!(notation.count(), 2);
/// assert_eq!(notation.sides(), 6);
/// assert!(notation.modifier().is_some());
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Notation {
    terms: Vec<Term>,
    scale: Option<Scale>,
}

impl Notation {
    /// Creates a notation, checking that the each `Term` is in bounds and that
    /// the total number of terms does not exceed [`MAX_TERMS`].
    ///
    /// # Errors
    ///
    /// Returns [`NotationError::Zero`] if any `count` or `sides` is zero, or
    /// [`NotationError::NumberTooLarge`] if either exceeds [`MAX_VALUE`].
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_roll::notation::Notation;
    ///
    /// let notation = Notation::try_new(2, 6, None).unwrap();
    /// assert_eq!(notation.count(), 2);
    /// ```
    pub fn try_new(terms: Vec<Term>, scale: Option<Scale>) -> Result<Self, NotationError> {
        if terms.len() > MAX_TERMS {
            return Err(NotationError::TooManyTerms(MAX_TERMS));
        };

        if !terms
            .iter()
            .any(|term| matches!(term.kind, TermKind::Dice { .. }))
        {
            return Err(NotationError::NoDice);
        };

        for term in &terms {
            match term.kind {
                TermKind::Dice { count, sides } => {
                    check_bounds(count, "count")?;
                    check_bounds(sides, "sides")?;
                }
                TermKind::Constant(value) => {
                    if value > MAX_VALUE {
                        return Err(NotationError::NumberTooLarge(MAX_VALUE));
                    }
                }
            }
        }

        let total_dice: u32 = terms
            .iter()
            .filter_map(|term| match term.kind {
                TermKind::Dice { count, .. } => Some(count),
                TermKind::Constant(_) => None,
            })
            .sum();

        if total_dice > MAX_DICE {
            return Err(NotationError::TooManyDice(MAX_DICE));
        };

        Ok(Notation { terms, scale })
    }

    /// The terms of the notation.
    #[must_use]
    pub fn terms(&self) -> &[Term] {
        &self.terms
    }

    /// The scale to apply to the total.
    #[must_use]
    pub fn scale(&self) -> Option<Scale> {
        self.scale
    }

    /// A single d20 with an optional modifier: the default roll for
    /// advantage and disadvantage.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_roll::notation::Notation;
    ///
    /// let notation = Notation::d20(None);
    /// assert_eq!((notation.count(), notation.sides()), (1, 20));
    /// ```
    #[must_use]
    pub fn d20() -> Notation {
        Notation {
            terms: vec![Term {
                kind: TermKind::Dice {
                    count: 1,
                    sides: 20,
                },
                sign: Sign::Plus,
            }],
            scale: None,
        }
    }

    /// Returns the notation for a critical hit, following the D&D 5e (2014)
    /// rules: the number of dice is doubled, while the sides and modifier are
    /// unchanged, so the modifier is still applied only once.
    ///
    /// The doubled count may exceed [`MAX_VALUE`]: a critical `1000d6` becomes
    /// `2000d6`. This is deliberate, as capping it would change the rule.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_roll::notation::Notation;
    ///
    /// let notation: Notation = "1d8+2".parse().unwrap();
    /// let critical = notation.critical();
    ///
    /// assert_eq!(critical.count(), 2);
    /// assert_eq!(critical.modifier(), notation.modifier());
    /// ```
    #[must_use]
    pub fn critical(self) -> Notation {
        todo!()
    }
}

impl FromStr for Notation {
    type Err = NotationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        todo!()
    }
}

impl fmt::Display for Notation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

fn check_bounds(value: u32, name: &'static str) -> Result<u32, NotationError> {
    match value {
        0 => Err(NotationError::Zero(name)),
        1..=MAX_VALUE => Ok(value),
        _ => Err(NotationError::NumberTooLarge(MAX_VALUE)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ::rstest::rstest;

    mod try_new {
        use super::*;
        use crate::test_helpers::scale;

        #[rstest]
        #[case::too_many_terms(vec![Term::dice(1, 4); 21], NotationError::TooManyTerms(MAX_TERMS))]
        #[case::no_dice_terms(vec![Term::constant(1)], NotationError::NoDice)]
        #[case::empty_terms(vec![], NotationError::NoDice)]
        #[case::too_many_dice(vec![Term::dice(MAX_DICE / 2 + 1, 6); 2], NotationError::TooManyDice(MAX_DICE))]
        #[case::zero_count(vec![Term::dice(0, 8)], NotationError::Zero("count"))]
        #[case::count_too_large(vec![Term::dice(MAX_VALUE + 1, 4)], NotationError::NumberTooLarge(MAX_VALUE))]
        #[case::zero_sides(vec![Term::dice(2, 0)], NotationError::Zero("sides"))]
        #[case::too_many_sides(vec![Term::dice(4, MAX_VALUE + 1)], NotationError::NumberTooLarge(MAX_VALUE))]
        #[case::constant_too_large(vec![Term::dice(1, 6), Term::constant(MAX_VALUE + 1)],  NotationError::NumberTooLarge(MAX_VALUE))]
        #[case::too_many_total_dice(vec![Term::dice(600, 6), Term::dice(600, 6)], NotationError::TooManyDice(MAX_DICE))]
        #[case::overflow(vec![Term::dice(u32::MAX, 10), Term::dice(1, 8)], NotationError::NumberTooLarge(MAX_VALUE))]
        fn rejects_invalid(#[case] terms: Vec<Term>, #[case] expected: NotationError) {
            let result = Notation::try_new(terms, None).unwrap_err();

            assert_eq!(result, expected);
        }

        #[rstest]
        #[case::single_dice_term(vec![Term::dice(1, 6)], None)]
        #[case::die_and_constant(vec![Term::dice(2, 6), Term::constant(3)], None)]
        #[case::several_dice_terms(vec![Term::dice(1, 8), Term::dice(3, 6), Term::constant(3)], None)]
        #[case::negative_terms(vec![Term::dice(1, 8), -Term::constant(2), -Term::dice(1, 4)], None)]
        #[case::leading_negative_const(vec![-Term::constant(2), Term::dice(2, 6)], None)]
        #[case::with_a_scale(vec![Term::dice(4, 8)], Some(scale(ScaleOperator::Multiply, 2)))]
        #[case::maximum_terms(vec![Term::dice(1, 4); MAX_TERMS], None)]
        #[case::maximum_total_dice(vec![Term::dice(500, 10), Term::dice(500, 8)], None)]
        #[case::constant_of_zero(vec![Term::dice(1, 6), Term::constant(0)], None)]
        fn accepts_valid(
            #[case] terms: Vec<Term>,
            #[case] scale: Option<Scale>,
        ) {
            let expected = Notation { terms: terms.clone(), scale };

            let result = Notation::try_new(terms, scale);

            assert_eq!(result, Ok(expected));
        }
    }

    mod d20 {
        use super::*;

        #[test]
        fn builds_correct_notation() {
            let result = Notation::d20();

            assert_eq!(
                result,
                Notation {
                    terms: vec![Term {
                        kind: TermKind::Dice {
                            count: 1,
                            sides: 20
                        },
                        sign: Sign::Plus
                    }],
                    scale: None
                }
            );
        }
    }
}
