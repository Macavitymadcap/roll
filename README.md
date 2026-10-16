# roll

A CLI tool for rolling dice, mostly for D&D 5e (2014).

Here's the shape of the CLI module, piece by piece: the clap struct, mapping flags to `RollType`, formatting output with `Display`, and how `main` ties it together. Then what to test.

## Designing the output

This part is up to you. Show enough for a player to check the arithmetic, and make each mode's result obvious. For example:

```plaintext
$ roll 2d6+3
2d6+3: [4, 2] + 3 = 9

$ roll -a d20+5
d20+5 with advantage: 17 (12 + 5), dropped 9 (4 + 5)

$ roll -s
[6, 5, 3, (1)] = 14
[4, 4, 2, (2)] = 10
…
```

The `Display` impl for `Notation` handles the first part of each line, and the dropped ability-score die can be shown in brackets.


## `main.rs`

With the work in `cli.rs`, `main` is tiny:

```rust
mod cli;

use clap::Parser;

fn main() {
    let cli = cli::Cli::parse();
    let roll_type = cli.roll_type();
    let rolls = roll::roller::roll(roll_type, &mut rand::rng());

    println!("{}", cli::format_rolls(&roll_type, &rolls));
}
```

`Cli::parse()` comes from the `Parser` trait, which must be in scope for the method to be callable. That's the same trait rule as `RngExt`. All the invalid-input cases are handled inside clap's parsing, so `main` needs no `Result` and no error handling.


## Formatting test cases

- **Small pure functions that format one line from plain data**, such as `fn format_roll(roll: &Roll, modifier: Option<Modifier>) -> String` and `fn format_ability_score(roll: &Roll) -> String`. Tests can pass any values they like, so assertions can be exact.
- **`format_rolls(roll_type, rolls)`**, which adds labels and layout for each mode and calls the line formatters. Tests for this level need real `Roll`s, and d1s make them predictable: `2d1+3` always rolls `[1, 1]` with a total of 5, even from a seeded generator. That lets you assert exact output here too, except for ability scores, which are always 4d6.

## Single roll lines

- **No modifier:** `[4, 2]` with a total of 6 gives something like `[4, 2] = 6`.
- **A single die:** decide whether `d20` shows `[17] = 17` or just `17`, and pin it down.
- **Each operator's symbol:** one case each for `+`, `−`, `×` and `÷`, which suits `rstest`. If you want a true minus sign for subtraction (`−`, U+2212) rather than a hyphen, this is where to decide it.
- **The total is shown as given.** For example, `[1]` with `− 5` and a total of 0 shows `= 0`. The formatter shouldn't recalculate anything.

## Ability score lines

- **The lowest die is marked**, for example `[6, 5, 3, (1)] = 14`.
- **Ties mark exactly one die.** With `[3, 3, 5, 6]`, only one 3 is bracketed. Pin down which one: the first is easiest.
- **The dice stay in rolled order.** Marking shouldn't sort them.

## `format_rolls`, per mode

- **Normal:** one line, prefixed with the notation, e.g. `2d1+3: [1, 1] + 3 = 5`.
- **Critical:** labelled as a crit. Decide whether the prefix shows the original notation (`1d8+2`) or the doubled one (`2d8+2`).
- **Advantage:** both rolls shown, with the kept roll first and labelled. Ordering is already covered by the roller's tests.
- **Disadvantage:** the same, with its own label.
- **Ability scores:** six lines.

## Format details to pin down early

- **Trailing newline:** `println!` adds one, so `format_rolls` probably shouldn't end with one. Put a test on that.
- **Line separators** for the multi-line modes: a single test on line count catches mistakes here.

