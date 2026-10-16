//! Parsing and validation of dice notation.
//!
//! Notation takes the form `XdY?Z`: an optional number of dice `X`, a die
//! with `Y` sides, and an optional modifier made of an operator (`+`, `-`,
//! `*` or `/`) and a value `Z`. Whitespace between the parts is ignored,
//! and `d` may be upper or lower case.
//!
//! A [`Notation`] can only be created through [`Notation::try_new`] or by
//! parsing, so every notation built from user input is within bounds.

use regex::{Captures, Regex};
use std::fmt;
use std::{str::FromStr, sync::LazyLock};

mod error;
mod modifier;

pub use error::NotationError;
pub use modifier::{Modifier, Operator};

/// The largest value allowed for the number of dice, the number of sides,
/// or a modifier's value.
pub const MAX_VALUE: u32 = 1_000;

static NOTATION_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?x)                                    # verbose mode, literal whitespace ignored
      ^ \s*
      (?<count>[0-9]+)? \s*                           # optional number of dice
      [dD] \s*
      (?<sides>[0-9]+)                                # faces per die
      (?: \s* (?<op>[-+*/]) \s* (?<value>[0-9]+) )?   # optional modifier
      \s* $
    ",
    )
    .expect("Notation regex should be valid")
});

/// A validated dice expression, such as `2d6+3`.
///
/// Create one by parsing a string or with [`Notation::try_new`].
///
/// # Examples
///
/// ```
/// use roll::notation::Notation;
///
/// let notation: Notation = "2d6+3".parse().unwrap();
///
/// assert_eq!(notation.count(), 2);
/// assert_eq!(notation.sides(), 6);
/// assert!(notation.modifier().is_some());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Notation {
    count: u32,
    sides: u32,
    modifier: Option<Modifier>,
}

impl Notation {
    /// Creates a notation, checking that `count` and `sides` are within bounds.
    ///
    /// # Errors
    ///
    /// Returns [`NotationError::Zero`] if `count` or `sides` is zero, or
    /// [`NotationError::NumberTooLarge`] if either exceeds [`MAX_VALUE`].
    ///
    /// # Examples
    ///
    /// ```
    /// use roll::notation::Notation;
    ///
    /// let notation = Notation::try_new(2, 6, None).unwrap();
    /// assert_eq!(notation.count(), 2);
    /// ```
    pub fn try_new(
        count: u32,
        sides: u32,
        modifier: Option<Modifier>,
    ) -> Result<Self, NotationError> {
        Ok(Notation {
            count: check_bounds(count, "count")?,
            sides: check_bounds(sides, "sides")?,
            modifier,
        })
    }

    /// The number of dice to roll.
    #[must_use]
    pub fn count(self) -> u32 {
        self.count
    }

    /// The number of sides of each die.
    #[must_use]
    pub fn sides(self) -> u32 {
        self.sides
    }

    /// The modifier to apply to the total, if any.
    #[must_use]
    pub fn modifier(self) -> Option<Modifier> {
        self.modifier
    }

    /// A single d20 with an optional modifier: the default roll for
    /// advantage and disadvantage.
    ///
    /// # Examples
    ///
    /// ```
    /// use roll::notation::Notation;
    ///
    /// let notation = Notation::d20(None);
    /// assert_eq!((notation.count(), notation.sides()), (1, 20));
    /// ```
    #[must_use]
    pub fn d20(modifier: Option<Modifier>) -> Notation {
        Notation {
            count: 1,
            sides: 20,
            modifier,
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
    /// use roll::notation::Notation;
    ///
    /// let notation: Notation = "1d8+2".parse().unwrap();
    /// let critical = notation.critical();
    ///
    /// assert_eq!(critical.count(), 2);
    /// assert_eq!(critical.modifier(), notation.modifier());
    /// ```
    #[must_use]
    pub fn critical(self) -> Notation {
        Notation {
            count: self.count * 2,
            ..self
        }
    }
}

impl FromStr for Notation {
    type Err = NotationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let captures = NOTATION_RE
            .captures(s)
            .ok_or(NotationError::InvalidFormat)?;

        Notation::try_new(
            parse_count(&captures)?,
            parse_sides(&captures)?,
            parse_modifier(&captures)?,
        )
    }
}

impl fmt::Display for Notation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.count != 1 {
            write!(f, "{}", self.count)?
        }

        write!(f, "d{}", self.sides)?;

        if let Some(modifier) = self.modifier {
            write!(f, "{modifier}")?
        }

        Ok(())
    }
}

fn parse_count(captures: &Captures) -> Result<u32, NotationError> {
    let count = captures
        .name("count")
        .map(|m| m.as_str().parse::<u32>())
        .transpose()?
        .unwrap_or(1);

    Ok(count)
}

fn parse_sides(captures: &Captures) -> Result<u32, NotationError> {
    let sides = captures["sides"].parse()?;

    Ok(sides)
}

fn parse_modifier(captures: &Captures) -> Result<Option<Modifier>, NotationError> {
    match (captures.name("op"), captures.name("value")) {
        (Some(op), Some(value)) => {
            let operator = op.as_str().parse()?;
            let value = value.as_str().parse()?;

            Modifier::try_new(operator, value).map(Some)
        }
        (None, None) => Ok(None),
        _ => Err(NotationError::InvalidFormat),
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
    use crate::test_helpers::modifier;
    use ::rstest::rstest;

    mod try_new {
        use super::*;

        #[rstest]
        #[case::typical(2, 6, None)]
        #[case::with_modifier(1, 20, Some(modifier(Operator::Add, 5)))]
        #[case::max_count(MAX_VALUE, 100, None)]
        #[case::max_sides(5, MAX_VALUE, None)]
        fn accepts_valid_values(
            #[case] count: u32,
            #[case] sides: u32,
            #[case] modifier: Option<Modifier>,
        ) {
            let result = Notation::try_new(count, sides, modifier);

            assert_eq!(
                result,
                Ok(Notation {
                    count,
                    sides,
                    modifier
                })
            );
        }

        #[rstest]
        #[case::zero_count(0, 6, NotationError::Zero("count"))]
        #[case::zero_sides(1, 0, NotationError::Zero("sides"))]
        #[case::count_too_large(MAX_VALUE + 1, 6, NotationError::NumberTooLarge(MAX_VALUE))]
        #[case::sides_too_large(1, MAX_VALUE + 1, NotationError::NumberTooLarge(MAX_VALUE))]
        fn rejects_invalid_values(
            #[case] count: u32,
            #[case] sides: u32,
            #[case] expected: NotationError,
        ) {
            let result = Notation::try_new(count, sides, None);

            assert_eq!(result, Err(expected));
        }
    }

    mod from_str {
        use super::*;

        #[test]
        fn defaults_to_one() {
            let input = "d20";

            let result: Notation = input.parse().unwrap();

            assert_eq!(
                result,
                Notation {
                    count: 1,
                    sides: 20,
                    modifier: None
                }
            );
        }

        #[test]
        fn ignores_whitespace() {
            let input = "   3    d   4  + 5  ";

            let result: Notation = input.parse().unwrap();

            assert_eq!(
                result,
                Notation {
                    count: 3,
                    sides: 4,
                    modifier: Some(modifier(Operator::Add, 5))
                }
            );
        }

        #[test]
        fn accepts_uppercase_d() {
            let input = "D100";

            let result: Notation = input.parse().unwrap();

            assert_eq!(
                result,
                Notation {
                    count: 1,
                    sides: 100,
                    modifier: None
                }
            );
        }

        #[rstest]
        #[case::word("banana")]
        #[case::whitespace_between_digits("2d1 0")]
        #[case::operator_without_value("d10+")]
        fn rejects_invalid_format(#[case] input: &str) {
            let result = input.parse::<Notation>();

            assert!(matches!(result, Err(NotationError::InvalidFormat)));
        }

        #[rstest]
        #[case::count("99999999999d6")]
        #[case::sides("d99999999999")]
        #[case::modifier("d10+99999999999")]
        fn rejects_overflow(#[case] input: &str) {
            let result = input.parse::<Notation>();

            assert!(matches!(result, Err(NotationError::InvalidNumber(_))));
        }

        #[rstest]
        #[case::add("+", Operator::Add)]
        #[case::sub("-", Operator::Sub)]
        #[case::mul("*", Operator::Mul)]
        #[case::div("/", Operator::Div)]
        fn accepts_modifier(#[case] input: &str, #[case] expected: Operator) {
            let input = format!("1d8{input}2");

            let result: Notation = input.parse().unwrap();

            assert_eq!(
                result,
                Notation {
                    count: 1,
                    sides: 8,
                    modifier: Some(modifier(expected, 2))
                }
            );
        }

        #[rstest]
        #[case::zero_count("0d6", NotationError::Zero("count"))]
        #[case::number_too_large("d1001", NotationError::NumberTooLarge(MAX_VALUE))]
        #[case::zero_divisor("d6/0", NotationError::Zero("divisor"))]
        fn passes_on_validation(#[case] input: &str, #[case] expected: NotationError) {
            let result = input.parse::<Notation>();

            assert_eq!(result, Err(expected));
        }
    }

    mod display {
        use super::*;

        #[rstest]
        #[case::one_die(Notation { count: 1, sides: 100, modifier: None}, "d100")]
        #[case::many_dice(Notation {count: 5, sides: 6, modifier: None}, "5d6")]
        #[case::with_modifier(Notation { count: 10, sides: 4, modifier: Some(modifier(Operator::Sub, 5))}, "10d4-5")]
        fn returns_formatted_string(#[case] notation: Notation, #[case] expected: &str) {
            let result = notation.to_string();

            assert_eq!(result, expected);
        }
    }

    mod d20 {
        use super::*;

        #[rstest]
        #[case::plain(None)]
        #[case::with_modifier(Some(modifier(Operator::Add, 5)))]
        fn builds_correct_notation(#[case] modifier: Option<Modifier>) {
            let result = Notation::d20(modifier);

            assert_eq!(
                result,
                Notation {
                    count: 1,
                    sides: 20,
                    modifier
                }
            )
        }
    }

    mod critical {
        use super::*;

        #[test]
        fn doubles_count() {
            let modifier = modifier(Operator::Add, 2);
            let notation = Notation {
                count: 1,
                sides: 8,
                modifier: Some(modifier),
            };

            let result = notation.critical();

            assert_eq!(
                result,
                Notation {
                    count: 2,
                    sides: 8,
                    modifier: Some(modifier)
                }
            );
        }
    }
}
