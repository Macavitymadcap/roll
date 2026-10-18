use super::error::NotationError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum TokenKind {
    Number(u32),
    Die,
    Plus,
    Minus,
    Times,
    Divide,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Token {
    pub(super) kind: TokenKind,
    pub(super) position: usize,
}

pub(super) fn tokenise(input: &str) -> Result<Vec<Token>, NotationError> {
    let mut chars = input.char_indices().peekable();
    let mut tokens = Vec::new();

    while let Some((position, character)) = chars.next() {
        let kind = match character {
            c if c.is_whitespace() => continue,
            '+' => TokenKind::Plus,
            '-' => TokenKind::Minus,
            '*' => TokenKind::Times,
            '/' => TokenKind::Divide,
            'd' | 'D' => TokenKind::Die,
            '0'..='9' => {
                let mut digits = String::from(character);
                while let Some((_, digit)) = chars.next_if(|&(_, c)| c.is_ascii_digit()) {
                    digits.push(digit);
                }

                TokenKind::Number(digits.parse()?)
            }
            _ => {
                return Err(NotationError::UnexpectedCharacter {
                    character,
                    position,
                });
            }
        };

        tokens.push(Token { kind, position });
    }

    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    fn kinds(input: &str) -> Vec<TokenKind> {
        tokenise(input)
            .unwrap()
            .iter()
            .map(|token| token.kind)
            .collect()
    }

    #[rstest]
    #[case::plus("+", vec![TokenKind::Plus])]
    #[case::minus("-", vec![TokenKind::Minus])]
    #[case::times("*", vec![TokenKind::Times])]
    #[case::divide("/", vec![TokenKind::Divide])]
    fn matches_arithmetic_symbol(#[case] input: &str, #[case] expected: Vec<TokenKind>) {
        let result = kinds(input);

        assert_eq!(result, expected);
    }

    #[rstest]
    #[case::lowercase("d", vec![TokenKind::Die])]
    #[case::uppercase("D", vec![TokenKind::Die])]
    fn matches_die(#[case] input: &str, #[case] expected: Vec<TokenKind>) {
        let result = kinds(input);

        assert_eq!(result, expected);
    }

    #[rstest]
    #[case::single("6", vec![TokenKind::Number(6)])]
    #[case::multiple("120", vec![TokenKind::Number(120)])]
    fn matches_digits(#[case] input: &str, #[case] expected: Vec<TokenKind>) {
        let result = kinds(input);

        assert_eq!(result, expected);
    }

    #[rstest]
    #[case::single_digit_integers("2d6", vec![0, 1, 2])]
    #[case::multi_digit_integer("12d10", vec![0, 2, 3])]
    #[case::whitespace(" 2 d 6 ", vec![1, 3, 5])]
    fn matches_positions(#[case] input: &str, #[case] expected: Vec<usize>) {
        let result = tokenise(input)
            .unwrap()
            .iter()
            .map(|token| token.position)
            .collect::<Vec<_>>();

        assert_eq!(result, expected);
    }

    #[test]
    fn whitespace_splits_numbers() {
        let input = "1 0";

        let result = kinds(input);

        assert_eq!(result, vec![TokenKind::Number(1), TokenKind::Number(0)]);
    }

    #[test]
    fn empty_input_returns_empty_vector() {
        let input = "";

        let result = tokenise(input).unwrap();

        assert_eq!(result, vec![]);
    }

    #[test]
    fn rejects_unknown_character() {
        let input = "2d6%";

        let result = tokenise(input);

        assert!(matches!(
            result,
            Err(NotationError::UnexpectedCharacter {
                character: '%',
                position: 3
            })
        ));
    }

    #[test]
    fn rejects_overflow() {
        let input = "99999999999";

        let result = tokenise(input);

        assert!(matches!(result, Err(NotationError::InvalidNumber(_))));
    }
}
