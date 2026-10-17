# roll

A command-line dice roller for tabletop RPGs, using standard dice notation
such as `2d6+3`. Special rolls follow the Dungeons & Dragons 5th Edition
(2014) rules: advantage, disadvantage, critical hits and ability scores.

```console
$ roll 2d6+3
2d6+3: [4, 2] + 3 = 9

$ roll -a d20+5
d20+5 (Advantage): [12] + 5 = 17, dropped [4] + 5 = 9
```

## Installation

Building from source needs a recent stable Rust toolchain, installed with
[rustup](https://rustup.rs).

```console
$ cargo install --git https://github.com/Macavitymadcap/roll
```

Or from a local clone:

```console
$ git clone https://github.com/Macavitymadcap/roll
$ cd roll
$ cargo install --path .
```

## Usage

```plaintext
roll [OPTIONS] [NOTATION]
```

| Option                 | Effect                                                 |
| ---------------------- | ------------------------------------------------------ |
| `-a`, `--advantage`    | Roll twice and keep the higher total                   |
| `-d`, `--disadvantage` | Roll twice and keep the lower total                    |
| `-c`, `--critical`     | Roll a critical hit: double the dice, modifier once    |
| `-s`, `--scores`       | Roll six ability scores: 4d6 each, dropping the lowest |
| `-h`, `--help`         | Show help                                              |
| `-V`, `--version`      | Show the version                                       |

Only one of `-a`, `-d`, `-c` and `-s` can be used at a time. Critical hits
need a notation; ability scores take none.

Without a notation, `roll` rolls a single d20.

## Dice notation

Notation takes the form `XdY?Z`:

| Part | Meaning                                    | Required |
| ---- | ------------------------------------------ | -------- |
| `X`  | Number of dice; defaults to 1              | No       |
| `dY` | A die with `Y` sides; `d` or `D`           | Yes      |
| `?`  | An operator: `+`, `-`, `*` or `/`          | No       |
| `Z`  | The value the operator applies to the roll | No       |

The modifier applies to the total of the dice, not to each die.

Whitespace between the parts is allowed, but the shell splits arguments on
spaces, so quote notation that contains any:

```console
$ roll "2d6 + 3"
2d6+3: [5, 1] + 3 = 9
```

### Limits

- The number of dice, the number of sides and the modifier's value must
  each be no more than 1,000.
- The number of dice and the number of sides must be at least 1.
- Dividing by zero is rejected.

### Rounding and bounds

Following the 5e rules, division rounds down. Totals never go below zero:
`1d4-5` gives 0.

## Roll types

### Advantage and disadvantage

The notation is rolled twice. With advantage the higher total is kept;
with disadvantage the lower. Both rolls are shown, kept first.

```console
$ roll -d d20+3
d20+3 (Disadvantage): [7] + 3 = 10, dropped [15] + 3 = 18
```

### Critical hits

The number of dice is doubled and the modifier is applied once, so
`1d8+2` is rolled as `2d8+2`. The original notation is shown.

```console
$ roll -c 1d8+2
1d8+2 (Critical!): [5, 3] + 2 = 10
```

### Ability scores

Six scores are rolled, each from 4d6 with the lowest die dropped. Dice are
shown highest first, with the dropped die in brackets.

```console
$ roll -s
[5, 4, 3, (2)] = 12
[6, 2, 2, (1)] = 10
[4, 4, 3, (1)] = 11
[6, 5, 1, (1)] = 12
[6, 6, 4, (3)] = 16
[5, 4, 3, (2)] = 12
```

## Using the library

The parsing and rolling logic is also available as a library, independent
of the command line:

```rust
use roll::notation::Notation;
use roll::roller::{self, RollType};

let notation: Notation = "d20+5".parse()?;
let rolls = roller::roll(RollType::Advantage(notation), &mut rand::rng());
```

Randomness comes from a generator passed in by the caller, so a seeded
generator gives repeatable results in tests. Run `cargo doc --open` for
the full API documentation.

## Development

```console
$ cargo test                  # unit, integration and documentation tests
$ cargo clippy --all-targets  # lints, including clippy::pedantic
$ cargo fmt                   # formatting
$ cargo llvm-cov --html       # coverage; needs cargo-llvm-cov installed
```

### Project structure

```plaintext
src/
├── lib.rs              # library root
├── notation.rs         # Notation, parsing and validation
├── notation/
│   ├── error.rs        # NotationError
│   └── modifier.rs     # Modifier and Operator
├── roller.rs           # rolling logic for each roll type
├── main.rs             # binary entry point
├── args.rs             # command-line arguments
└── output.rs           # formatting results for the terminal
tests/
└── cli.rs              # end-to-end tests of the binary
```

## Roadmap

- Daggerheart duality dice: Hope and Fear d12s.
- Compound notation with several dice terms, such as `d8+3d6+2`.
- Exploding dice.

## Licence

MIT Version 2.0.