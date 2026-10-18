//! Errors from parsing or constructing dice notation.

use std::num::ParseIntError;
use thiserror::Error;

/// An error from parsing or constructing dice notation.
#[derive(Debug, Error, PartialEq)]
#[non_exhaustive]
pub enum NotationError {
    /// The input doesn't match the `XdY?Z` notation format.
    #[error("invalid dice notation; expected something like 2d6+3")]
    InvalidFormat,

    /// The tokeniser meets a character it doesn't know.
    #[error("unexpected '{character}' at column {}", .position + 1)]
    UnexpectedCharacter {
        /// The character that was not recognized.
        character: char,
        /// The zero-based position of the character in the input.
        position: usize,
    },

    /// The parser meets a token it didn't expect.
    #[error("expected {expected} at column {}", .position + 1)]
    UnexpectedToken {
        /// The type of token expected by the parser.
        expected: &'static str,
        /// The zero-based position of the character in the input. 
        position: usize
    },

    /// The input ends while the parser still needs more.
    #[error("expected {expected} at the end of the notation")]
        UnexpectedEnd { 
        /// The type of token expected by the prseeeds more.
        expected: &'static str
    },

    /// No term is a dice term.
    #[error("no dice found; include at least one dice term, such as d6")]
    NoDice,

    /// The total dice across terms exceeds `MAX_DICE`
    #[error("too many dice; the maximum across all terms is {0}")]
    TooManyDice(u32),

    /// There are more than `MAX_TERMS` terms
    #[error("too many terms; the maximum is {0}")]
    TooManyTerms(u32),


    /// A number in the input couldn't be parsed, usually because it's
    /// too large for a `u32`.
    #[error("invalid number: {0}")]
    InvalidNumber(#[from] ParseIntError),

    /// A value that must be positive was zero. Holds the name of the
    /// value: `"count"`, `"sides"` or `"divisor"`.
    #[error("{0} must be greater than zero")]
    Zero(&'static str),

    /// The operator isn't one of `+`, `-`, `*` or `/`.
    #[error("invalid operator; expected one of +, -, /, *")]
    InvalidOperator,

    /// A value exceeded the maximum. Holds the maximum,
    /// [`MAX_VALUE`](crate::notation::MAX_VALUE).
    #[error("number too large; maximum is {0}")]
    NumberTooLarge(u32),
}
