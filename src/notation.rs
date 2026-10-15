use regex::Regex;
use std::{str::FromStr, sync::LazyLock};

mod error;
mod modifier;

pub use error::{MAX_INTEGER, ParseError};
pub use modifier::{Modifier, Operator};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Notation {
    pub count: u32,
    pub sides: u32,
    pub modifier: Option<Modifier>,
}

impl FromStr for Notation {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let captures = NOTATION_RE.captures(s).ok_or(ParseError::InvalidFormat)?;

        let count = captures
            .name("count")
            .map(|m| m.as_str().parse::<u32>())
            .transpose()?
            .unwrap_or(1);

        if count == 0 {
            return Err(ParseError::Zero("count"));
        }

        if count > MAX_INTEGER {
            return Err(ParseError::NumberTooLarge(MAX_INTEGER));
        }

        let sides = captures["sides"].parse::<u32>()?;

        if sides == 0 {
            return Err(ParseError::Zero("sides"));
        }

        if sides > MAX_INTEGER {
            return Err(ParseError::NumberTooLarge(MAX_INTEGER));
        }

        let modifier = match (captures.name("op"), captures.name("value")) {
            (Some(op), Some(value)) => {
                let value: u32 = value.as_str().parse()?;

                if value > MAX_INTEGER {
                    return Err(ParseError::NumberTooLarge(MAX_INTEGER));
                }

                Some(Modifier {
                    operator: op.as_str().parse()?,
                    value,
                })
            }
            (None, None) => None,
            _ => return Err(ParseError::InvalidFormat),
        };

        if matches!(
            modifier,
            Some(Modifier {
                operator: Operator::Div,
                value: 0
            })
        ) {
            return Err(ParseError::Zero("divisor"));
        }

        Ok(Notation {
            count,
            sides,
            modifier,
        })
    }
}

impl Notation {
    pub fn critical(self) -> Notation {
        Notation {
            count: self.count * 2,
            ..self
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Format
    #[test]
    fn parses_ignoring_whitespace() {
        // Arrange
        let input = "   1    d   4  + 5  ";

        // Act
        let result: Notation = input.parse().unwrap();

        // Assert
        assert_eq!(result.count, 1);
        assert_eq!(result.sides, 4);
        assert_eq!(
            result.modifier,
            Some(Modifier {
                operator: Operator::Add,
                value: 5
            })
        );
    }

    #[test]
    fn parse_rejects_whitespace_between_digits_of_sides() {
        // Arrange
        let input = "2d1 0";

        // Act
        let result = input.parse::<Notation>();

        // Assert
        assert!(matches!(result, Err(ParseError::InvalidFormat)))
    }

    #[test]
    fn parse_rejects_invalid_string() {
        // Arrange
        let input = "banana";

        // Act
        let result = input.parse::<Notation>();

        // Assert
        assert!(matches!(result, Err(ParseError::InvalidFormat)));
    }

    #[test]
    fn parse_rejects_notation_with_operator_but_no_value() {
        // Arrange
        let input = "d10+";

        // Act
        let result = input.parse::<Notation>();

        // Assert
        assert!(matches!(result, Err(ParseError::InvalidFormat)))
    }

    // Count
    #[test]
    fn parses_die_with_no_count() {
        // Arrange
        let input = "d20";

        // Act
        let result: Notation = input.parse().unwrap();

        // Assert
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
    fn parses_die_with_with_uppercase_d() {
        // Arrange
        let input = "D100";

        // Act
        let result: Notation = input.parse().unwrap();

        // Assert
        assert_eq!(
            result,
            Notation {
                count: 1,
                sides: 100,
                modifier: None
            }
        );
    }

    #[test]
    fn parse_rejects_count_of_zero() {
        // Arrange
        let input = "0d8";

        // Act
        let result = input.parse::<Notation>();

        // Assert
        assert!(matches!(result, Err(ParseError::Zero("count"))))
    }

    #[test]
    fn parses_dice_with_count() {
        // Arrange
        let input = "3d6";

        // Act
        let result: Notation = input.parse().unwrap();

        // Assert
        assert_eq!(
            result,
            Notation {
                count: 3,
                sides: 6,
                modifier: None
            }
        );
    }

    #[test]
    fn parses_count_of_one_thousand() {
        // Arrange
        let input = format!("{MAX_INTEGER}d10");

        // Act
        let result: Notation = input.parse().unwrap();

        // Assert
        assert_eq!(
            result,
            Notation {
                count: MAX_INTEGER,
                sides: 10,
                modifier: None
            }
        );
    }

    #[test]
    fn parse_rejects_count_greater_than_one_thousand() {
        // Arrange
        let input = "1001d6";

        // Act
        let result = input.parse::<Notation>();

        // Assert
        assert!(matches!(
            result,
            Err(ParseError::NumberTooLarge(MAX_INTEGER))
        ));
    }

    #[test]
    fn parse_rejects_count_integer_overflow() {
        // Arrange
        let input = "99999999999d6";

        // Act
        let result = input.parse::<Notation>();

        // Assert
        assert!(matches!(result, Err(ParseError::InvalidNumber(_))));
    }

    // Sides
    #[test]
    fn parse_rejects_sides_of_zero() {
        // Arrange
        let input = "d0";

        // Act
        let result = input.parse::<Notation>();

        // Assert
        assert!(matches!(result, Err(ParseError::Zero("sides"))))
    }

    #[test]
    fn parses_sides_of_one_thousand() {
        // Arrange
        let input = "d1000";

        // Act
        let result: Notation = input.parse().unwrap();

        // Assert
        assert_eq!(
            result,
            Notation {
                count: 1,
                sides: MAX_INTEGER,
                modifier: None
            }
        )
    }

    #[test]
    fn parse_rejects_sides_greater_than_one_thousand() {
        // Arrange
        let input = "d1001";

        // Act
        let result = input.parse::<Notation>();

        // Assert
        assert!(matches!(
            result,
            Err(ParseError::NumberTooLarge(MAX_INTEGER))
        ))
    }

    #[test]
    fn parse_rejects_sides_integer_overflow() {
        // Arrange
        let input = "d99999999999";

        // Act
        let result = input.parse::<Notation>();

        // Assert
        assert!(matches!(result, Err(ParseError::InvalidNumber(_))));
    }

    // Modifier
    #[test]
    fn parses_die_with_add_modifier() {
        // Arrange
        let input = "1d8+2";

        // Act
        let result: Notation = input.parse().unwrap();

        // Assert
        assert_eq!(
            result,
            Notation {
                count: 1,
                sides: 8,
                modifier: Some(Modifier {
                    operator: Operator::Add,
                    value: 2
                })
            }
        );
    }

    #[test]
    fn parses_dice_with_sub_modifier() {
        // Arrange
        let input = "2d4-1";

        // Act
        let result: Notation = input.parse().unwrap();

        // Assert
        assert_eq!(result.count, 2);
        assert_eq!(result.sides, 4);
        assert_eq!(
            result.modifier,
            Some(Modifier {
                operator: Operator::Sub,
                value: 1
            })
        );
    }

    #[test]
    fn parses_die_with_mul_modifier() {
        // Arrange
        let input = "d10*3";

        // Act
        let result: Notation = input.parse().unwrap();

        // Assert
        assert_eq!(result.count, 1);
        assert_eq!(result.sides, 10);
        assert_eq!(
            result.modifier,
            Some(Modifier {
                operator: Operator::Mul,
                value: 3
            })
        );
    }

    #[test]
    fn parses_dice_with_div_modifier() {
        // Arrange
        let input = "6d6/2";

        // Act
        let result: Notation = input.parse().unwrap();

        // Assert
        assert_eq!(result.count, 6);
        assert_eq!(result.sides, 6);
        assert_eq!(
            result.modifier,
            Some(Modifier {
                operator: Operator::Div,
                value: 2
            })
        );
    }

    #[test]
    fn parse_rejects_dividing_by_zero() {
        // Arrange
        let input = "2d10/0";

        // Act
        let result = input.parse::<Notation>();

        // Assert
        assert!(matches!(result, Err(ParseError::Zero("divisor"))))
    }

    #[test]
    fn parses_modifier_value_of_one_thousand() {
        // Arrange
        let input = format!("1d4+{MAX_INTEGER}");

        // Act
        let result: Notation = input.parse().unwrap();

        // Assert
        assert_eq!(
            result,
            Notation {
                count: 1,
                sides: 4,
                modifier: Some(Modifier {
                    operator: Operator::Add,
                    value: MAX_INTEGER
                })
            }
        )
    }

    #[test]
    fn parse_rejects_modifier_greater_than_one_thousand() {
        // Arrange
        let input = "d4+10001";

        // Act
        let result = input.parse::<Notation>();

        // Assert
        assert!(matches!(
            result,
            Err(ParseError::NumberTooLarge(MAX_INTEGER))
        ));
    }

    #[test]
    fn parse_rejects_modifier_integer_overflow() {
        // Arrange
        let input = "d4+99999999999";

        // Act
        let result = input.parse::<Notation>();

        // Assert
        assert!(matches!(result, Err(ParseError::InvalidNumber(_))));
    }

    // critical()
    #[test]
    fn critical_returns_new_notation_with_count_doubled() {
        // Arrange
        let notation = Notation {
            count: 1,
            sides: 8,
            modifier: Some(Modifier {
                operator: Operator::Add,
                value: 2,
            }),
        };

        // Act
        let result = notation.critical();

        // Assert
        assert_eq!(
            result,
            Notation {
                count: 2,
                sides: 8,
                modifier: Some(Modifier {
                    operator: Operator::Add,
                    value: 2
                })
            }
        );
    }
}
