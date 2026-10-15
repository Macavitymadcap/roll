use std::str;

use super::ParseError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operator {
    Add,
    Sub,
    Mul,
    Div,
}

impl str::FromStr for Operator {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let operator = match s {
            "+" => Operator::Add,
            "-" => Operator::Sub,
            "*" => Operator::Mul,
            "/" => Operator::Div,
            _ => return Err(ParseError::InvalidOperator),
        };

        Ok(operator)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Modifier {
    pub operator: Operator,
    pub value: u32,
}

impl Modifier {
    pub fn apply(self, total: u32) -> u32 {
        match self.operator {
            Operator::Add => total.saturating_add(self.value),
            Operator::Sub => total.saturating_sub(self.value),
            Operator::Mul => total.saturating_mul(self.value),
            Operator::Div => total / self.value,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plus_sign_parsed_as_add_operator() {
        // Arrange
        let input = "+";

        // Act
        let result: Operator = input.parse().unwrap();

        // Assert
        assert_eq!(result, Operator::Add)
    }

    #[test]
    fn hyphen_parsed_as_sub_operator() {
        // Arrange
        let input = "-";

        // Act
        let result: Operator = input.parse().unwrap();

        // Assert
        assert_eq!(result, Operator::Sub)
    }

    #[test]
    fn forward_stroke_parsed_as_div_operator() {
        // Arrange
        let input = "/";

        // Act
        let result: Operator = input.parse().unwrap();

        // Assert
        assert_eq!(result, Operator::Div)
    }

    #[test]
    fn asterisk_parsed_as_mul_operator() {
        // Arrange
        let input = "*";

        // Act
        let result: Operator = input.parse().unwrap();

        // Assert
        assert_eq!(result, Operator::Mul)
    }

    #[test]
    fn percentage_not_parsed_as_operator() {
        // Arrange
        let input = "%";

        // Act
        let result = input.parse::<Operator>();

        // Assert
        assert!(matches!(result, Err(ParseError::InvalidOperator)))
    }

    #[test]
    fn returns_total_summed_with_modifier_given_add_operator() {
        // Arrange
        let total = 4;
        let modifier = Modifier {
            operator: Operator::Add,
            value: 2,
        };

        // Act
        let result = modifier.apply(total);

        // Assert
        assert_eq!(result, 6);
    }

    #[test]
    fn saturates_at_max_when_adding() {
        // Arrange
        let total = u32::MAX;
        let modifier = Modifier {
            operator: Operator::Add,
            value: 1,
        };

        // Act
        let result = modifier.apply(total);

        // Assert
        assert_eq!(result, u32::MAX);
    }

    #[test]
    fn returns_total_with_modifier_subtracted_given_sub_operator() {
        // Arrange
        let total = 4;
        let modifier = Modifier {
            operator: Operator::Sub,
            value: 2,
        };

        // Act
        let result = modifier.apply(total);

        // Assert
        assert_eq!(result, 2);
    }

    #[test]
    fn saturates_at_min_when_subtracting() {
        // Arrange
        let total = 1;
        let modifier = Modifier {
            operator: Operator::Sub,
            value: 2,
        };

        // Act
        let result = modifier.apply(total);

        // Assert
        assert_eq!(result, 0);
    }

    #[test]
    fn returns_total_multiplied_by_modifier_given_mul_operator() {
        // Arrange
        let total = 4;
        let modifier = Modifier {
            operator: Operator::Mul,
            value: 3,
        };

        // Act
        let result = modifier.apply(total);

        // Assert
        assert_eq!(result, 12);
    }

    #[test]
    fn saturates_at_max_when_multiplying() {
        // Arrange
        let total = u32::MAX / 2 + 1;
        let modifier = Modifier {
            operator: Operator::Mul,
            value: 2,
        };

        // Act
        let result = modifier.apply(total);

        // Assert
        assert_eq!(result, u32::MAX);
    }

    #[test]
    fn returns_total_divided_by_modifier_give_div_operator() {
        // Arrange
        let total = 8;
        let modifier = Modifier {
            operator: Operator::Div,
            value: 2,
        };

        // Act
        let result = modifier.apply(total);

        // Assert
        assert_eq!(result, 4);
    }

    #[test]
    fn rounds_down_when_dividing() {
        // Arrange
        let total = 7;
        let modifier = Modifier {
            operator: Operator::Div,
            value: 2,
        };

        // Act
        let result = modifier.apply(total);

        // Assert
        assert_eq!(result, 3);
    }
}
