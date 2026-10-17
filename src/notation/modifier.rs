//! Modifiers applied to the total of a roll, such as the `+3` in `2d6+3`.

use super::{MAX_VALUE, NotationError};
use std::fmt;
use std::str::FromStr;

/// An arithmetic operator in a modifier.
///
/// Parses from `+`, `-`, `*` or `/`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operator {
    /// Adds the value to the total (`+`).
    Add,
    /// Subtracts the value from the total, stopping at zero (`-`).
    Sub,
    /// Multiplies the total by the value (`*`).
    Mul,
    /// Divides the total by the value, rounding down (`/`).
    Div,
}

impl FromStr for Operator {
    type Err = NotationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let operator = match s {
            "+" => Operator::Add,
            "-" => Operator::Sub,
            "*" => Operator::Mul,
            "/" => Operator::Div,
            _ => return Err(NotationError::InvalidOperator),
        };

        Ok(operator)
    }
}

impl fmt::Display for Operator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Operator::Add => write!(f, "+")?,
            Operator::Sub => write!(f, "-")?,
            Operator::Mul => write!(f, "*")?,
            Operator::Div => write!(f, "/")?,
        }

        Ok(())
    }
}

/// A validated operator and value applied to a roll's total.
///
/// Create one with [`Modifier::try_new`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Modifier {
    operator: Operator,
    value: u32,
}

impl Modifier {
    /// Creates a modifier, checking the value is usable with the operator.
    ///
    /// # Errors
    ///
    /// Returns [`NotationError::Zero`] if the operator is [`Operator::Div`]
    /// and the value is zero, or [`NotationError::NumberTooLarge`] if the
    /// value exceeds [`MAX_VALUE`].
    ///
    /// # Examples
    ///
    /// ```
    /// use roll::notation::{Modifier, Operator};
    ///
    /// let modifier = Modifier::try_new(Operator::Add, 3).unwrap();
    /// assert_eq!(modifier.apply(10), 13);
    ///
    /// assert!(Modifier::try_new(Operator::Div, 0).is_err());
    /// ```
    pub fn try_new(operator: Operator, value: u32) -> Result<Self, NotationError> {
        match (operator, value) {
            (Operator::Div, 0) => Err(NotationError::Zero("divisor")),
            (_, 0..=MAX_VALUE) => Ok(Modifier { operator, value }),
            _ => Err(NotationError::NumberTooLarge(MAX_VALUE)),
        }
    }

    /// The modifier's operator.
    #[must_use]
    pub fn operator(self) -> Operator {
        self.operator
    }

    /// The modifier's value.
    #[must_use]
    pub fn value(self) -> u32 {
        self.value
    }

    /// Applies the modifier to a total.
    ///
    /// The result never goes below zero or above [`u32::MAX`]: subtraction
    /// stops at zero and addition and multiplication saturate. Division rounds
    /// down, following the D&D 5e (2014) rules.
    ///
    /// # Examples
    ///
    /// ```
    /// use roll::notation::{Modifier, Operator};
    ///
    /// let halve = Modifier::try_new(Operator::Div, 2).unwrap();
    /// assert_eq!(halve.apply(7), 3);
    ///
    /// let minus_five = Modifier::try_new(Operator::Sub, 5).unwrap();
    /// assert_eq!(minus_five.apply(3), 0);
    /// ```
    #[must_use]
    pub fn apply(self, total: u32) -> u32 {
        match self.operator {
            Operator::Add => total.saturating_add(self.value),
            Operator::Sub => total.saturating_sub(self.value),
            Operator::Mul => total.saturating_mul(self.value),
            Operator::Div => total / self.value,
        }
    }
}

impl fmt::Display for Modifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.operator, self.value)
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    mod operator {
        use super::*;

        mod from_str {
            use super::*;

            #[rstest]
            #[case::add("+", Operator::Add)]
            #[case::sub("-", Operator::Sub)]
            #[case::mul("*", Operator::Mul)]
            #[case::div("/", Operator::Div)]
            fn accepts(#[case] input: &str, #[case] expected: Operator) {
                let result: Operator = input.parse().unwrap();

                assert_eq!(result, expected);
            }

            #[rstest]
            #[case::empty("")]
            #[case::one_char_string("%")]
            #[case::duplicated_operator("++")]
            fn rejects(#[case] input: &str) {
                let result = input.parse::<Operator>();

                assert!(matches!(result, Err(NotationError::InvalidOperator)));
            }
        }

        mod display {
            use super::*;

            #[rstest]
            #[case::add(Operator::Add, String::from("+"))]
            #[case::sub(Operator::Sub, String::from("-"))]
            #[case::mul(Operator::Mul, String::from("*"))]
            #[case::div(Operator::Div, String::from("/"))]
            fn returns_formatted_string(#[case] operator: Operator, #[case] expected: String) {
                assert_eq!(operator.to_string(), expected);
            }
        }
    }

    mod modifier {
        use super::*;

        mod display {
            use super::*;

            #[test]
            fn returns_formatted_string() {
                let modifier = Modifier {
                    operator: Operator::Add,
                    value: 10,
                };

                let result = modifier.to_string();

                assert_eq!(result, "+10");
            }
        }

        mod try_new {
            use super::*;

            #[rstest]
            #[case::add(Operator::Add)]
            #[case::sub(Operator::Sub)]
            #[case::mul(Operator::Mul)]
            fn accepts_zero_for_non_division_operators(#[case] operator: Operator) {
                let value = 0;

                let result = Modifier::try_new(operator, value);

                assert_eq!(result, Ok(Modifier { operator, value }));
            }

            #[rstest]
            #[case::add(Operator::Add)]
            #[case::sub(Operator::Sub)]
            #[case::mul(Operator::Mul)]
            #[case::div(Operator::Div)]
            fn accepts_maximum_value(#[case] operator: Operator) {
                let value = MAX_VALUE;

                let result = Modifier::try_new(operator, value);

                assert_eq!(result, Ok(Modifier { operator, value }));
            }

            #[rstest]
            #[case::add(Operator::Add)]
            #[case::sub(Operator::Sub)]
            #[case::mul(Operator::Mul)]
            #[case::div(Operator::Div)]
            fn rejects_above_the_maximum(#[case] operator: Operator) {
                let value = MAX_VALUE + 1;

                let result = Modifier::try_new(operator, value);

                assert!(matches!(
                    result,
                    Err(NotationError::NumberTooLarge(MAX_VALUE))
                ));
            }

            #[test]
            fn rejects_division_by_zero() {
                let operator = Operator::Div;
                let value = 0;

                let result = Modifier::try_new(operator, value);

                assert!(matches!(result, Err(NotationError::Zero("divisor"))));
            }
        }

        mod apply {
            use super::*;

            #[rstest]
            #[case::add(4, Operator::Add, 2, 6)]
            #[case::add_saturates(u32::MAX, Operator::Add, 1, u32::MAX)]
            #[case::sub(4, Operator::Sub, 2, 2)]
            #[case::sub_saturates(1, Operator::Sub, 2, u32::MIN)]
            #[case::mul(4, Operator::Mul, 3, 12)]
            #[case::mul_saturates(4, Operator::Mul, u32::MAX / 2 + 1, u32::MAX)]
            #[case::div(8, Operator::Div, 2, 4)]
            #[case::div_rounds_down(7, Operator::Div, 2, 3)]
            fn returns_expected_value(
                #[case] total: u32,
                #[case] operator: Operator,
                #[case] value: u32,
                #[case] expected: u32,
            ) {
                let modifier = Modifier { operator, value };

                let result = modifier.apply(total);

                assert_eq!(result, expected);
            }
        }
    }
}
