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
