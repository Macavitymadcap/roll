//! A dice-rolling library and command-line tool using tabletop RPG
//! notation such as `2d6+3`.
//!
//! The library has two parts:
//!
//! - [`notation`] parses dice notation into a validated [`notation::Notation`].
//! - [`roller`] rolls a notation in one of several modes, such as advantage
//!   or ability scores, following the D&D 5e (2014) rules.
//!
//! # Examples
//!
//! ```
//! use roll::notation::Notation;
//! use roll::roller::{roll, RollType};
//!
//! let notation: Notation = "d20+5".parse().unwrap();
//! let rolls = roll(RollType::Advantage(notation), &mut rand::rng());
//!
//! assert_eq!(rolls.len(), 2);
//! assert!(rolls[0].total >= rolls[1].total);
//! ```

#![warn(missing_docs)]

pub mod notation;
pub mod roller;
#[cfg(test)]
mod test_helpers;
