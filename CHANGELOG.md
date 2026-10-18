# Changelog

All notable changes to this project are recorded in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and the project follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- New `NotationError` variants:
  - `UnexpectedCharacter`: The tokeniser meets a character it doesn't know 
  - `UnexpectedToken`: The parser meets a token it didn't expect
  - `UnexpectedEnd`: The input ends while the parser still needs more
  - `NoDice`: No term is a dice term
  - `TooManyDice`: The total dice across terms exceeds `MAX_DICE`
  - `TooManyTerms`: There are more than `MAX_TERMS` terms
- `token` module to handle tokenising of notation, with `TokenKind` enum, 
  `Token` struct and `tokenise` function; 

## [0.1.0] - 2026-10-17

The first release of `rust-roll`, a dice roller for tabletop RPGs.

### Added

- The `roll` command, which rolls dice described in `XdY?Z` notation: an
  optional number of dice, a die with any number of sides, and an optional
  modifier using `+`, `-`, `*` or `/`.
- Notation accepts `d` or `D`, and whitespace between its parts when
  quoted.
- Rolling a single d20 when no notation is given.
- Roll types following the D&D 5th Edition (2014) rules:
  - `-a`, `--advantage`: roll twice, keeping the higher total.
  - `-d`, `--disadvantage`: roll twice, keeping the lower total.
  - `-c`, `--critical`: double the dice and apply the modifier once.
  - `-s`, `--scores`: roll six ability scores, each 4d6 dropping the
    lowest die.
- Output showing each die and the arithmetic behind every total, with
  dropped rolls and dice labelled.
- Division rounding down, and totals never going below zero.
- Validation with specific error messages for malformed notation, zero
  dice or sides, division by zero, and values above 1,000.
- `--help` documenting the options and the notation, and `--version`.
- A library alongside the command:
  - `notation`: `Notation`, `Modifier`, `Operator` and `NotationError`, for
    parsing and validating notation.
  - `roller`: `roll`, `RollType` and `Roll`, for rolling a notation in any
    mode with a random number generator supplied by the caller.

[Unreleased]: https://github.com/Macavitymadcap/roll/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/Macavitymadcap/roll/releases/tag/v0.1.0
