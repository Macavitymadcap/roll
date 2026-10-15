use std::num;
use thiserror::Error;

pub const MAX_INTEGER: u32 = 1_000;

#[derive(Debug, Error, PartialEq)]
pub enum ParseError {
    #[error("invalid dice notation; expected something like 2d6+3")]
    InvalidFormat,

    #[error("invalid number: {0}")]
    InvalidNumber(#[from] num::ParseIntError),

    #[error("{0} must be greater than zero")]
    Zero(&'static str),

    #[error("invalid operator; expected one of +, -, /, * ")]
    InvalidOperator,

    #[error("number too large; maximum is {0}")]
    NumberTooLarge(u32),
}
