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
            ' ' | '\t' => continue,
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
            _ => return Err(NotationError::UnexpectedCharacter { character, position }),
        };

        tokens.push(Token { kind, position });
    }

    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use crate::notation::token;

use super::*;
    use rstest::rstest;

    #[rstest]
    #[case::plus("+", vec![Token { kind: TokenKind::Plus, position: 0 }])]
    #[case::minus("-", vec![Token { kind: TokenKind::Minus, position: 0 }])]
    #[case::times("*", vec![Token { kind: TokenKind::Times, position: 0 }])]
    #[case::divide("/", vec![Token { kind: TokenKind::Divide, position: 0 }])]
    fn matches_arithmetic_symbol(#[case] input: &str, #[case] expected: Vec<Token>) {
        let result = tokenise(input).unwrap();

        assert_eq!(result, expected);
    }

    #[rstest]
    #[case::lowercase("d", vec![Token {kind: TokenKind::Die, position: 0}])]
    #[case::uppercase("D", vec![Token {kind: TokenKind::Die, position: 0}])]

    fn matches_die(#[case] input: &str, #[case] expected: Vec<Token>) {
        let result = tokenise(input).unwrap();

        assert_eq!(result, expected);
    }

    #[rstest]
    #[case::single("6", vec![Token {kind: TokenKind::Number(6), position: 0}])]
    #[case::multiple("120", vec![Token {kind: TokenKind::Number(120), position: 0}])]
    fn matches_digits(#[case] input: &str, #[case] expected: Vec<Token>) {
        let result = tokenise(input).unwrap();

        assert_eq!(result, expected);
    }

    #[rstest]
    #[case::no_whitespace("2d6", vec![0, 1, 2])]
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
        ))
    }

    #[test]
    fn rejects_overflow() {
        let input = "99999999999";

        let result = tokenise(input);

        assert!(matches!(
            result,
            Err(NotationError::InvalidNumber(_))
        ))
    }

}
