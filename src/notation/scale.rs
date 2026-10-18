//! Scales applied to the total of terms in a roll, such as '*3' in 'd4+3d6*3'.

use crate::notation::{MAX_VALUE, NotationError};

/// An arithmetic operator in a scale.
///
/// Parses from '*' or '/'.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScaleOperator {
    /// Multiplies the total ('*')
    Multiply,
    /// Divides the total ('/')
    Divide,
}

/// A validated operator and value applied to a roll's total.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Scale {
    operator: ScaleOperator,
    value: u32,
}

impl Scale {
    /// Creates a scale, checking the value is usable with the operator.
    ///
    /// # Errors
    ///
    /// Returns [`NotationError::Zero`] if the operator is [`ScaleOperator::Divide`]
    /// and the value is zero, or [`NotationError::NumberTooLarge`] if the
    /// value exceeds [`MAX_VALUE`].
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_roll::notation::{Scale, ScaleOperator};
    ///
    /// let scale = Scale::try_new(ScaleOperator::Multiply, 3).unwrap();
    /// assert_eq!(scale.apply(10), 30);
    ///
    /// assert!(Scale::try_new(ScaleOperator::Divide, 0).is_err());
    /// ```
    pub fn try_new(operator: ScaleOperator, value: u32) -> Result<Self, NotationError> {
        match (operator, value) {
            (ScaleOperator::Divide, 0) => Err(NotationError::Zero("divisor")),
            (_, 0..=MAX_VALUE) => Ok(Scale { operator, value }),
            _ => Err(NotationError::NumberTooLarge(MAX_VALUE)),
        }
    }

    /// The scale's operator.
    #[must_use]
    pub fn operator(self) -> ScaleOperator {
        self.operator
    }

    /// The scale's value.
    #[must_use]
    pub fn value(self) -> u32 {
        self.value
    }

    /// Applies the scale to a total.
    ///
    /// The result never goes below zero or above [`u32::MAX`]:  multiplication
    /// saturates and division rounds down, following the D&D 5e (2014) rules.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_roll::notation::{Scale, ScaleOperator};
    ///
    /// let halve = Scale::try_new(ScaleOperator::Divide, 2).unwrap();
    /// assert_eq!(halve.apply(8), 4);
    ///
    /// let double = Scale::try_new(ScaleOperator::Multiply, 2).unwrap();
    /// assert_eq!(double.apply(4), 8);
    /// ```
    #[must_use]
    pub fn apply(self, total: u32) -> u32 {
        match self.operator {
            ScaleOperator::Multiply => total.saturating_mul(self.value),
            ScaleOperator::Divide => total / self.value,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    mod try_new {
        use super::*;

        #[test]
        fn accepts_zero_for_multiply() {
            let operator = ScaleOperator::Multiply;
            let value = 0;

            let result = Scale::try_new(operator, value);

            assert_eq!(result, Ok(Scale { operator, value }));
        }

        #[rstest]
        #[case::mul(ScaleOperator::Multiply)]
        #[case::div(ScaleOperator::Divide)]
        fn accepts_maximum_value(#[case] operator: ScaleOperator) {
            let value = MAX_VALUE;

            let result = Scale::try_new(operator, value);

            assert_eq!(result, Ok(Scale { operator, value }));
        }

        #[rstest]
        #[case::mul(ScaleOperator::Multiply)]
        #[case::div(ScaleOperator::Divide)]
        fn rejects_above_the_maximum(#[case] operator: ScaleOperator) {
            let value = MAX_VALUE + 1;

            let result = Scale::try_new(operator, value);

            assert!(matches!(
                result,
                Err(NotationError::NumberTooLarge(MAX_VALUE))
            ));
        }

        #[test]
        fn rejects_division_by_zero() {
            let operator = ScaleOperator::Divide;
            let value = 0;

            let result = Scale::try_new(operator, value);

            assert!(matches!(result, Err(NotationError::Zero("divisor"))));
        }
    }

    mod apply {
        use super::*;

        #[rstest]
        #[case::multiply(4, ScaleOperator::Multiply, 3, 12)]
        #[case::multiply_saturates(4, ScaleOperator::Multiply, u32::MAX / 2 + 1, u32::MAX)]
        #[case::divide(8, ScaleOperator::Divide, 2, 4)]
        #[case::divide_rounds_down(7, ScaleOperator::Divide, 2, 3)]
        fn returns_expected_value(
            #[case] total: u32,
            #[case] operator: ScaleOperator,
            #[case] value: u32,
            #[case] expected: u32,
        ) {
            let scale = Scale { operator, value };

            let result = scale.apply(total);

            assert_eq!(result, expected);
        }
    }
}
