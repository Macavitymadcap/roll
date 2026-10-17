# Compound notation: implementation guide

This guide covers the `compound-notation` branch of `rust-roll`: replacing
the regex parser with a hand-written tokeniser and parser, and extending
`Notation` from a single dice term to a chain of terms such as
`d8+3d6+2`. It sets out the grammar, the new types, the errors, the
knock-on changes to the roller and CLI, and the test cases for each step.
It assumes the code as it stands on `main`: the `notation` module with
`error` and `modifier` submodules, the `roller` module, and the `args` and
`output` modules in the binary.

New language concepts are explained where they first appear, with links to
the documentation. No new crates are needed; the `regex` crate is removed.

## Decisions

| Decision                      | Choice                                                        |
| ----------------------------- | ------------------------------------------------------------- |
| Operators between terms       | `+` and `-` only                                              |
| Multiplication and division   | At most once, at the end, applied to the whole total          |
| Constants                     | Allowed as terms anywhere in the chain                        |
| Leading sign                  | Optional, so `+d6` and `-2+d6` are valid (see below)          |
| Arithmetic                    | Sum in a signed integer; clamp the final result at zero       |
| Dice required                 | At least one dice term, otherwise a `NoDice` error            |
| Critical hits                 | Double every dice term's count; constants and scale unchanged |
| Parser                        | Hand-written tokeniser and recursive descent parser           |

The leading sign is a suggestion rather than something you decided. Allowing
it means the first term needs no special treatment anywhere: every term has a
sign, and the first one defaults to `+`. If you would rather reject a leading
sign, the parser change is one line and the error tests gain two cases.

## The grammar

Writing the grammar down before any code makes the parser almost mechanical
to write, and it doubles as a checklist for tests. The notation below is
[EBNF](https://en.wikipedia.org/wiki/Extended_Backus%E2%80%93Naur_form):
`[ x ]` means optional, `{ x }` means zero or more repetitions, `|` separates
alternatives, and `,` means "followed by".

```ebnf
notation = [ sign ] , term , { sign , term } , [ scale ] ;
term     = [ number ] , die , number
         | number ;
scale    = ( "*" | "/" ) , number ;
sign     = "+" | "-" ;
die      = "d" | "D" ;
number   = digit , { digit } ;
```

Whitespace is allowed between any two tokens, but not inside a number, so
`2d1 0` is still rejected.

How some inputs break down:

| Input          | Terms                       | Scale  |
| -------------- | --------------------------- | ------ |
| `d20`          | `+1d20`                     | none   |
| `2d6+3`        | `+2d6`, `+3`                | none   |
| `d8+3d6+2`     | `+1d8`, `+3d6`, `+2`        | none   |
| `-2+d6`        | `-2`, `+1d6`                | none   |
| `d8+2d6*2`     | `+1d8`, `+2d6`              | `*2`   |
| `4d6/2`        | `+4d6`                      | `/2`   |

The existing single-term notations are all valid compound notations, so
every existing input keeps working; `2d6+3` is now a dice term and a
constant term rather than a dice term and a modifier.

## Limits

| Constant    | Value | Applies to                                       |
| ----------- | ----- | ------------------------------------------------ |
| `MAX_VALUE` | 1,000 | Each count, each number of sides, each constant, the scale value |
| `MAX_DICE`  | 1,000 | The total number of dice across all terms        |
| `MAX_TERMS` | 20    | The number of terms, dice and constants together |

These limits keep every total well inside a `u32`, which means the
arithmetic needs no saturation:

- The largest dice sum is 1,000 dice × 1,000 sides = 1,000,000.
- Twenty constants add at most 20,000.
- A scale of `*1000` takes that to about 1.02 billion.
- A critical hit doubles the dice, giving about 2.04 billion.
- `u32::MAX` is about 4.29 billion.

Twenty terms is generous for sneak attack and similar stacked damage;
lower it if you prefer. As before, a critical hit can take the dice above
`MAX_DICE`; document that on `critical`.

## Module layout

```plaintext
src/
├── notation.rs           # Notation, try_new, FromStr, Display, limits
└── notation/
    ├── error.rs          # NotationError
    ├── term.rs           # Sign, TermKind, Term
    ├── scale.rs          # Scale, ScaleOperator (replaces modifier.rs)
    ├── token.rs          # Token, TokenKind, tokenise
    └── parser.rs         # Parser
```

Remove the regex dependency once nothing uses it:

```console
$ cargo remove regex
```

`LazyLock` and the `NOTATION_RE` static go with it.

## Step 1: errors

The regex could only say that the input didn't match. A tokeniser and
parser know exactly where they were and what they expected, so the errors
can say so.

### New and changed variants

| Variant               | Raised when                                        | Message                                                 |
| --------------------- | -------------------------------------------------- | ------------------------------------------------------- |
| `UnexpectedCharacter` | The tokeniser meets a character it doesn't know    | `unexpected 'x' at column 5`                            |
| `UnexpectedToken`     | The parser meets a token it didn't expect          | `expected a number at column 3`                         |
| `UnexpectedEnd`       | The input ends while the parser still needs more   | `expected a number at the end of the notation`          |
| `NoDice`              | No term is a dice term                             | `no dice found; include at least one dice term, such as d6` |
| `TooManyDice`         | The total dice across terms exceeds `MAX_DICE`     | `too many dice; the maximum across all terms is 1000`   |
| `TooManyTerms`        | There are more than `MAX_TERMS` terms              | `too many terms; the maximum is 20`                     |

`InvalidNumber`, `Zero` and `NumberTooLarge` stay as they are.
`InvalidFormat` and `InvalidOperator` are replaced by the three
`Unexpected…` variants and can be removed.

### Struct-like variants

The variants so far have been unit variants (`NoDice`) or tuple variants
(`Zero("count")`). Variants can also have named fields, like a struct:

```rust
#[derive(Debug, Error, PartialEq)]
#[non_exhaustive]
pub enum NotationError {
    #[error("unexpected '{character}' at column {}", .position + 1)]
    UnexpectedCharacter { character: char, position: usize },

    #[error("expected {expected} at column {}", .position + 1)]
    UnexpectedToken { expected: &'static str, position: usize },

    #[error("expected {expected} at the end of the notation")]
    UnexpectedEnd { expected: &'static str },

    // …
}
```

Named fields read better than positional ones once a variant carries more
than one value. See
[Defining an enum](https://doc.rust-lang.org/book/ch06-01-defining-an-enum.html)
in the book.

In the `#[error]` attribute, `{character}` refers to a field by name.
Extra format arguments after the string can refer to fields with a leading
dot, so `.position + 1` turns the zero-based position into a one-based
column for the user. The
[thiserror documentation](https://docs.rs/thiserror/latest/thiserror/)
covers both forms.

`expected` is a `&'static str` because the parser only ever passes string
literals such as `"a number"`, which live for the whole program.

### Matching on named fields in tests

Patterns can name some fields and ignore the rest with `..`:

```rust
assert!(matches!(
    result,
    Err(NotationError::UnexpectedToken { position: 4, .. })
));
```

This checks the variant and the position without repeating the exact
`expected` wording in every test.

### `#[non_exhaustive]`

If you haven't added it before publishing, add
[`#[non_exhaustive]`](https://doc.rust-lang.org/reference/attributes/type_system.html#the-non_exhaustive-attribute)
to `NotationError` and `RollType` now. It forces code outside your crate to
include a `_` arm when matching, so adding variants later is not a breaking
change. Inside your own crate it has no effect.

## Step 2: the tokeniser

The tokeniser turns the input string into a list of tokens, dropping
whitespace. The parser then works with tokens and never has to think about
characters.

### Types

```rust
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

pub(super) fn tokenise(input: &str) -> Result<Vec<Token>, NotationError>
```

[`pub(super)`](https://doc.rust-lang.org/reference/visibility-and-privacy.html#pubin-path-pubcrate-pubsuper-and-pubself)
makes an item visible to the parent module, `notation`, and its other
children, such as `parser`, but not outside. Tokens are an implementation
detail of parsing, so they stay out of the public API.

### Walking the characters

[`str::char_indices`](https://doc.rust-lang.org/std/primitive.str.html#method.char_indices)
yields `(position, character)` pairs, so every token knows where it came
from. The position is a byte offset; for ASCII input that is the same as
the character index, and any non-ASCII character is rejected as unexpected
anyway.

Wrap the iterator with
[`.peekable()`](https://doc.rust-lang.org/std/iter/struct.Peekable.html)
to be able to look at the next item without consuming it. Two methods do
most of the work:

- `peek()` returns the next item without advancing.
- [`next_if(predicate)`](https://doc.rust-lang.org/std/iter/struct.Peekable.html#method.next_if)
  consumes the next item only if the predicate returns `true`, and returns
  it as an `Option`.

The overall shape is a loop with a `match` on each character:

```rust
let mut chars = input.char_indices().peekable();
let mut tokens = Vec::new();

while let Some((position, character)) = chars.next() {
    let kind = match character {
        ' ' | '\t' => continue,
        '+' => TokenKind::Plus,
        // '-', '*', '/', 'd' | 'D' …
        '0'..='9' => { /* read the rest of the number */ }
        _ => return Err(NotationError::UnexpectedCharacter { character, position }),
    };

    tokens.push(Token { kind, position });
}
```

- `continue` skips to the next character without pushing a token.
- `'0'..='9'` is a range pattern over characters, the same idea as
  `1..=MAX_VALUE` in `check_bounds`.
- Matching whitespace with
  [`char::is_whitespace`](https://doc.rust-lang.org/std/primitive.char.html#method.is_whitespace)
  instead of listing characters needs a match guard:
  `c if c.is_whitespace() => continue`.

### Reading a multi-digit number

When the first digit arrives, keep taking characters while they are digits.
`next_if` with
[`char::is_ascii_digit`](https://doc.rust-lang.org/std/primitive.char.html#method.is_ascii_digit)
does exactly that:

```rust
let mut digits = String::from(character);

while let Some((_, digit)) = chars.next_if(|&(_, c)| c.is_ascii_digit()) {
    digits.push(digit);
}
```

The closure receives a reference to the next item; the pattern
`|&(_, c)|` destructures it, ignoring the position.

Then parse `digits` with `parse::<u32>()?`. Reusing `str::parse` means an
overflowing number still produces `InvalidNumber` through the existing
`#[from]`, so the overflow tests keep working.

### Tokeniser tests

Test `tokenise` directly in `token.rs`; these are the character-level
concerns that the parser never sees.

| Case                       | Input       | Expected                                        |
| -------------------------- | ----------- | ----------------------------------------------- |
| Each symbol                | `+`, `-`, `*`, `/` | One token of the matching kind (`rstest`)  |
| Both cases of die          | `d`, `D`    | `Die`                                           |
| Single-digit number        | `6`         | `Number(6)`                                     |
| Multi-digit number         | `120`       | `Number(120)`                                   |
| Positions                  | `2d6`       | Positions 0, 1 and 2                            |
| Whitespace skipped         | ` 2 d 6 `   | Three tokens at positions 1, 3 and 5            |
| Whitespace splits numbers  | `1 0`       | `Number(1)`, `Number(0)`                        |
| Empty input                | `""`        | An empty list                                   |
| Unknown character          | `2d6%`      | `UnexpectedCharacter { character: '%', position: 3 }` |
| Overflow                   | `99999999999` | `InvalidNumber(_)`                            |

## Step 3: term and scale types

### Terms

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sign {
    Plus,
    Minus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TermKind {
    Dice { count: u32, sides: u32 },
    Constant(u32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Term {
    pub sign: Sign,
    pub kind: TermKind,
}
```

`Term` is plain data with public fields, like `Roll`. Its values are
validated as part of `Notation::try_new`, so an invalid `Term` can exist but
can never become part of a `Notation`. That keeps the validation in one
place rather than splitting it across three constructors.

Short constructors make tests and the parser read well:

```rust
impl Term {
    pub fn dice(count: u32, sides: u32) -> Self { … }   // sign: Plus
    pub fn constant(value: u32) -> Self { … }           // sign: Plus
    pub fn negated(self) -> Self { … }                  // sign: Minus
}
```

With these, `d8-2` is `vec![Term::dice(1, 8), Term::constant(2).negated()]`.
They're one-liners, so they don't need tests of their own. Add
`#[must_use]` to `negated`; ignoring its result is a likely bug, just as
with `critical`.

### Scale

`Modifier` becomes `Scale`. Addition and subtraction are now term signs,
so its operator only needs the two remaining cases:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScaleOperator {
    Multiply,
    Divide,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Scale {
    operator: ScaleOperator,
    value: u32,
}
```

`Scale::try_new`, its getters and `apply` port across from `Modifier` with
the `Add` and `Sub` cases removed. Division by zero and the `MAX_VALUE`
check stay in `try_new`, since `apply` would panic on a zero divisor.

### Scale tests

Port the existing `Modifier` tests, dropping the addition and subtraction
cases:

- `try_new` accepts zero for `Multiply`, accepts the maximum for both
  operators, rejects above the maximum, and rejects dividing by zero.
- `apply` multiplies, saturates on multiplication overflow, divides, and
  rounds down when dividing.

## Step 4: `Notation`

### Structure

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Notation {
    terms: Vec<Term>,
    scale: Option<Scale>,
}
```

Notice that `Copy` has gone from the derives. The next section explains why,
and what changes as a result.

### Copy and Clone

A type can only be
[`Copy`](https://doc.rust-lang.org/std/marker/trait.Copy.html) if copying
its bytes is a complete copy of the value. A `Vec` owns memory on the heap,
so copying its bytes would leave two vectors pointing at the same memory.
Any type containing a `Vec` can therefore be `Clone`, which makes a deep
copy when you ask for one, but not `Copy`.

This is the same rule you met when `Roll`'s getters took `self` by value.
So far you have been able to pass `Notation` around freely because every
use copied it. Now passing it by value moves it, and the old value can no
longer be used. The book's chapter on
[references and borrowing](https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html)
covers the rules.

The changes that follow:

- Methods take `&self`: `terms(&self) -> &[Term]`,
  `scale(&self) -> Option<Scale>` and `critical(&self) -> Notation`.
  `Scale` is still `Copy`, so returning it by value is fine.
- `RollType` loses `Copy` too, since it contains a `Notation`.
- Functions that only read a notation take `&Notation`. In the roller,
  `roll_once(notation: &Notation, …)` can then be called twice for
  advantage without cloning.
- `roll` takes `&RollType`, so `main` can still use the roll type for
  formatting after rolling.
- `Notation::d20` takes no arguments now: `Notation::d20()`.

The compiler will point at every place that needs changing; work through
its errors after changing the derive.

### `try_new`

```rust
pub fn try_new(terms: Vec<Term>, scale: Option<Scale>) -> Result<Self, NotationError>
```

Check in this order, returning the first error:

1. More than `MAX_TERMS` terms gives `TooManyTerms(MAX_TERMS)`.
2. No dice term gives `NoDice`.
3. For each dice term, `check_bounds` on its count and sides.
4. For each constant, the upper bound only; a constant of 0 is valid.
5. More than `MAX_DICE` dice in total gives `TooManyDice(MAX_DICE)`.

`Scale` has already validated itself, as `Modifier` did.

Some iterator methods that fit these checks:

- [`Iterator::any`](https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.any)
  answers "is there at least one dice term?":
  `terms.iter().any(|term| matches!(term.kind, TermKind::Dice { .. }))`.
- [`Iterator::filter_map`](https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.filter_map)
  keeps the items for which a closure returns `Some`, unwrapping them as it
  goes. Returning `Some(count)` for dice terms and `None` for constants,
  then calling `.sum::<u32>()`, gives the total number of dice. Checking
  the term limit first guarantees the sum can't overflow.

The bounds checks loop over the terms with a `for` loop and `?`, since each
check can return early.

### `critical`

Map over the terms, doubling the count of each dice term and copying
constants unchanged, then collect into a new `Vec`. The scale is copied
unchanged.

### `Notation` tests

`try_new`, as a table of accepted inputs and one of rejected inputs:

| Case                         | Terms and scale                          | Expected                    |
| ---------------------------- | ---------------------------------------- | --------------------------- |
| Single dice term             | `d6`                                     | `Ok`                        |
| Dice and constant            | `2d6`, `+3`                              | `Ok`                        |
| Several dice terms           | `d8`, `+3d6`, `+2`                       | `Ok`                        |
| Negative terms               | `d8`, `-2`, `-d4`                        | `Ok`                        |
| Leading negative constant    | `-2`, `+d6`                              | `Ok`                        |
| With a scale                 | `2d6`, scale `*2`                        | `Ok`                        |
| Maximum terms                | 20 × `d4`                                | `Ok`                        |
| Maximum total dice           | `500d6`, `+500d6`                        | `Ok`                        |
| Constant of zero             | `d6`, `+0`                               | `Ok`                        |
| Constants only               | `3`, `+2`                                | `NoDice`                    |
| Empty                        | no terms                                 | `NoDice`                    |
| Zero count                   | `0d6`                                    | `Zero("count")`             |
| Zero sides                   | `d0`                                     | `Zero("sides")`             |
| Count too large              | `1001d6`                                 | `NumberTooLarge(1000)`      |
| Sides too large              | `d1001`                                  | `NumberTooLarge(1000)`      |
| Constant too large           | `d6`, `+1001`                            | `NumberTooLarge(1000)`      |
| Too many terms               | 21 × `d4`                                | `TooManyTerms(20)`          |
| Too many dice in total       | `600d6`, `+600d6`                        | `TooManyDice(1000)`         |

For the 20 and 21 term cases, build the vector with the `vec!` repeat form,
`vec![Term::dice(1, 4); 21]`, which needs `Term: Clone`.

`critical`:

| Case                         | Input              | Expected            |
| ---------------------------- | ------------------ | ------------------- |
| Every dice term doubled      | `d8+3d6+2`         | `2d8+6d6+2`         |
| Constants unchanged          | `d6-2`             | `2d6-2`             |
| Scale unchanged              | `d6*2`             | `2d6*2`             |

`d20` builds `d20`: one quick test.

## Step 5: the parser

A recursive descent parser has one function per grammar rule. Each function
consumes the tokens for its rule and calls the functions for the rules
inside it. With the grammar above, the parser is three methods and a
helper. [Crafting Interpreters](https://craftinginterpreters.com/parsing-expressions.html)
explains the technique well, if you want the background; the grammar here
is far simpler than the one in the book.

### Structure

```rust
use std::iter::Peekable;
use std::vec::IntoIter;

pub(super) struct Parser {
    tokens: Peekable<IntoIter<Token>>,
}

impl Parser {
    pub(super) fn new(tokens: Vec<Token>) -> Self { … }

    pub(super) fn parse(mut self) -> Result<(Vec<Term>, Option<Scale>), NotationError> { … }

    fn parse_term(&mut self, sign: Sign) -> Result<Term, NotationError> { … }

    fn parse_scale(&mut self) -> Result<Option<Scale>, NotationError> { … }

    fn expect_number(&mut self, expected: &'static str) -> Result<u32, NotationError> { … }
}
```

Some points about the types:

- `tokens.into_iter()` on a `Vec<Token>` gives a
  [`std::vec::IntoIter<Token>`](https://doc.rust-lang.org/std/vec/struct.IntoIter.html),
  an iterator that owns the tokens. Wrapped in `Peekable`, that's the
  field's type. The parser owns its tokens, so it needs no lifetimes.
- The methods take `&mut self` because reading tokens changes the
  iterator's position. Even `peek` takes `&mut self`: to show you the next
  item, `Peekable` has to pull it from the underlying iterator and hold on
  to it.
- `parse` takes `mut self` by value. The parser is used once and then
  discarded, so consuming it makes that explicit: after `parse`, there is
  no parser left to misuse.
- `parse` returns the parts rather than a `Notation`. `from_str` passes them
  to `Notation::try_new`, which keeps validation in one place.

`Token` is `Copy`, so `self.tokens.peek().map(|token| token.kind)` gives you
an owned `TokenKind` to `match` on, with no borrow left hanging.

### What each method does

`parse` follows the `notation` rule:

1. If the next token is `Plus` or `Minus`, consume it as the first term's
   sign; otherwise the sign is `Plus`. `next_if` works on tokens just as on
   characters.
2. Call `parse_term` for the first term.
3. Loop: if the next token is `Plus` or `Minus`, consume it and parse
   another term. If it's `Times`, `Divide` or the end, stop. Anything else
   is `UnexpectedToken`, expecting `"'+', '-', '*', '/' or the end"`.
4. Call `parse_scale`.
5. If any token is left, return `UnexpectedToken` at its position,
   expecting `"the end of the notation"`. This catches `d6*2+3` and
   `d6*2*2`.

`parse_term` follows the `term` rule:

- `Number(n)` followed by `Die`: a dice term with count `n`; the sides come
  from `expect_number`.
- `Number(n)` on its own: a constant.
- `Die`: a dice term with a count of 1.
- Anything else is `UnexpectedToken`, and no token at all is
  `UnexpectedEnd`, both expecting `"a number or 'd'"`.

`parse_scale` follows the `scale` rule: if the next token is `Times` or
`Divide`, consume it, read the value with `expect_number`, and build the
scale with `Scale::try_new`. Otherwise return `Ok(None)`.

`expect_number` consumes the next token if it's a `Number` and returns its
value. Otherwise it returns `UnexpectedToken` at the token's position, or
`UnexpectedEnd` if there are no tokens left, using the `expected`
description passed in.

### Testing the parser

Building token vectors by hand is noisy and tests the same thing as
parsing strings. Test the grammar through `from_str` instead, in the next
step; the tokeniser's own tests already cover the character-level
behaviour.

## Step 6: wiring up `FromStr`

`from_str` becomes three lines: tokenise, parse, construct.

```rust
fn from_str(s: &str) -> Result<Self, Self::Err> {
    let tokens = tokenise(s)?;
    let (terms, scale) = Parser::new(tokens).parse()?;

    Notation::try_new(terms, scale)
}
```

### `from_str` tests

Successful parses:

| Case                       | Input            | Expected                                       |
| -------------------------- | ---------------- | ---------------------------------------------- |
| Single die                 | `d20`            | `d20`                                          |
| Count                      | `3d6`            | `3d6`                                          |
| Dice and constant          | `2d6+3`          | `2d6`, `+3`                                    |
| Several dice terms         | `d8+3d6+2`       | `d8`, `+3d6`, `+2`                             |
| Subtracted constant        | `d20-1`          | `d20`, `-1`                                    |
| Subtracted dice            | `d8-d4`          | `d8`, `-d4`                                    |
| Leading plus               | `+d6`            | `d6`                                           |
| Leading minus              | `-2+d6`          | `-2`, `+d6`                                    |
| Multiply                   | `2d6*2`          | `2d6`, scale `*2`                              |
| Divide                     | `d8+2d6/2`       | `d8`, `+2d6`, scale `/2`                       |
| Whitespace                 | ` d8 + 3d6 + 2 ` | `d8`, `+3d6`, `+2`                             |
| Upper case                 | `D8+3D6`         | `d8`, `+3d6`                                   |

Rejected input, grouped by where the error comes from:

| Case                       | Input            | Expected                                       |
| -------------------------- | ---------------- | ---------------------------------------------- |
| Unknown character          | `banana`         | `UnexpectedCharacter` at 0                     |
| Unknown symbol             | `d6%2`           | `UnexpectedCharacter` at 2                     |
| Empty                      | `""`             | `UnexpectedEnd`                                |
| Only whitespace            | `"   "`          | `UnexpectedEnd`                                |
| Die without sides          | `d`              | `UnexpectedEnd`                                |
| Trailing sign              | `d6+`            | `UnexpectedEnd`                                |
| Trailing scale operator    | `d6*`            | `UnexpectedEnd`                                |
| Two signs                  | `d6++2`          | `UnexpectedToken` at 3                         |
| Two leading signs          | `--d6`           | `UnexpectedToken` at 1                         |
| Missing sign between terms | `2d6 3`          | `UnexpectedToken` at 4                         |
| Space inside a number      | `2d1 0`          | `UnexpectedToken` at 4                         |
| Term after the scale       | `d6*2+3`         | `UnexpectedToken` at 4                         |
| Two scales                 | `d6*2*2`         | `UnexpectedToken` at 4                         |
| Die without sides mid-way  | `d6+d+2`         | `UnexpectedToken` at 4                         |
| No dice                    | `3+2`            | `NoDice`                                       |
| Divide by zero             | `d6/0`           | `Zero("divisor")`                              |
| Zero count                 | `0d6`            | `Zero("count")`                                |
| Too many dice              | `600d6+600d6`    | `TooManyDice(1000)`                            |
| Overflow                   | `99999999999d6`  | `InvalidNumber(_)`                             |

The rejected cases make a good `rstest` table if `NotationError` cases are
written with the `..` pattern for positions, or as a separate table per
variant.

### Updating the existing tests

Most existing `from_str` tests keep their inputs but change their expected
values:

- Tests expecting `InvalidFormat` now expect a specific variant: `banana`
  gives `UnexpectedCharacter`, `d10+` gives `UnexpectedEnd`, and `2d1 0`
  gives `UnexpectedToken`.
- Tests building the expected `Notation` with a `Modifier` now build it
  with a constant term or a `Scale`.
- The bounds tests move to the `try_new` table above, as before.

### Test helpers

Building compound notations from terms in every roller test gets verbose.
A helper that parses a string is easier to read:

```rust
pub(crate) fn notation(input: &str) -> Notation {
    input.parse().unwrap()
}
```

Earlier, roller tests built notations directly so that a parser bug
couldn't break them. With compound notation that trade-off changes: the
parser has thorough tests of its own, so if it broke, those tests would
point at the cause even if roller tests failed too. Remove the `modifier`
helper; a `scale` helper isn't needed if notations are parsed.

## Step 7: `Display`

### Rules

- A dice term displays as `NdS`, omitting a count of 1.
- A constant displays as its value.
- Each term after the first is preceded by its sign.
- The first term shows its sign only if it is `-`.
- A scale is written last as `*N` or `/N`.

Write `Display` for `Sign`, `ScaleOperator` and `Scale` first, then use them
in `Notation`'s implementation: write the first term, then loop over
`self.terms.iter().skip(1)`, writing each sign and term.
[`Iterator::skip`](https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.skip)
does what it says.

`Display` for a term can't show its own sign, because the first term's
sign is shown differently; implement it for `TermKind` instead, and have
`Notation` decide whether to write the sign.

### `Display` tests

| Input            | Displays as     |
| ---------------- | --------------- |
| `d6`             | `d6`            |
| `1d6`            | `d6`            |
| `2d6+3`          | `2d6+3`         |
| `d8+3d6+2`       | `d8+3d6+2`      |
| `d8-d4-1`        | `d8-d4-1`       |
| `+d6`            | `d6`            |
| `-2+d6`          | `-2+d6`         |
| `2d6*2`          | `2d6*2`         |
| `D8 + 3D6`       | `d8+3d6`        |

A round-trip test checks the two halves agree: for each input, parsing the
displayed form gives back the same `Notation`. Use the same cases as an
`rstest` table.

## Step 8: the roller

### Result types

A `Roll` now needs to remember which dice belong to which term, and the
constants, so the output can show the working. Replace `dice` with a list
of rolled terms:

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RolledTerm {
    Dice { sign: Sign, dice: Vec<u32> },
    Constant { sign: Sign, value: u32 },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Roll {
    pub terms: Vec<RolledTerm>,
    pub total: u32,
}
```

An ability score is a `Roll` with a single `Dice` term.

### Signed arithmetic

The total is calculated in `i64`, which can hold negative intermediate
values, then clamped at zero, then scaled:

1. Convert each value to `i64` with
   [`i64::from`](https://doc.rust-lang.org/std/convert/trait.From.html).
   `From` is only implemented for conversions that can never fail, and
   every `u32` fits in an `i64`.
2. Add or subtract each term's value according to its sign.
3. Clamp at zero with
   [`.max(0)`](https://doc.rust-lang.org/std/cmp/trait.Ord.html#method.max).
4. Convert back with
   [`u32::try_from`](https://doc.rust-lang.org/std/convert/trait.TryFrom.html).
   `TryFrom` is the fallible counterpart of `From`, returning a `Result`.
   The limits guarantee the value fits, so
   `.expect("limits keep totals within u32")` documents that guarantee.
5. Apply the scale with `Scale::apply`, as the modifier was before.

Clamping before scaling gives the same result as scaling first: a negative
total multiplied or divided by a positive number is still at most zero, so
it clamps to zero either way. Clamping first means `apply` keeps working on
`u32`.

### Ownership in the roller

- `roll_once(notation: &Notation, rng: &mut impl Rng) -> Roll` borrows the
  notation.
- `roll(roll_type: &RollType, rng: &mut impl Rng) -> Vec<Roll>` borrows the
  roll type. Matching on a reference binds the notation inside as
  `&Notation`, so it passes straight to `roll_once`.
- The critical arm calls `roll_once(&notation.critical(), rng)`. The
  temporary `Notation` returned by `critical` lives until the end of the
  statement, so borrowing it there is fine.

### Roller tests

The d1 trick makes compound totals exact:

| Case                                 | Notation    | Expected                                      |
| ------------------------------------ | ----------- | --------------------------------------------- |
| Dice grouped by term                 | `d1+2d1`    | Two dice terms, `[1]` and `[1, 1]`; total 3   |
| Constants kept                       | `2d1+3`     | A dice term and a constant; total 5           |
| Subtracted constant                  | `3d1-1`     | Total 2                                       |
| Subtracted dice                      | `3d1-d1`    | Total 2                                       |
| Arithmetically honest                | `d1-3+5`    | Total 3 (per-step clamping would give 5)      |
| Clamped at zero                      | `d1-5`      | Total 0                                       |
| Leading minus                        | `-2+d1`     | Total 0                                       |
| Scale applies to the whole total     | `d1+d1*2`   | Total 4 (scaling only the last term gives 3)  |
| Division rounds down                 | `3d1/2`     | Total 1                                       |
| Negative total then scale            | `d1-5*2`    | Total 0                                       |
| Each term rolls its own dice         | `2d6+3d8`   | Two dice terms with 2 and 3 dice (seeded)     |

For `roll`, add one critical case: `d1+2d1+1` gives dice terms of
`[1, 1]` and `[1, 1, 1, 1]` and a total of 7. The other modes are
unaffected beyond the change to `&RollType`, so their tests only need
updating to the new `Roll` shape.

Ability score tests change from `roll.dice` to the single dice term. A
slice pattern unpacks it:

```rust
let [RolledTerm::Dice { dice, .. }] = result.terms.as_slice() else {
    panic!("an ability score is a single dice term");
};
```

## Step 9: the CLI

### Arguments

`roll_type` can no longer copy the notation out of `&self`. Clone it, and
use the default only when it's missing:

```rust
let notation = self.notation.clone().unwrap_or_else(Notation::d20);
```

[`unwrap_or_else`](https://doc.rust-lang.org/std/option/enum.Option.html#method.unwrap_or_else)
takes a function and only calls it for `None`. With `unwrap_or`, the
argument is built every time, even when it's thrown away; Clippy's
[`or_fun_call`](https://rust-lang.github.io/rust-clippy/master/index.html#or_fun_call)
lint flags that. `Notation::d20` is passed as the function itself, the same
trick as `.map(Some)`.

In `main`, borrow the roll type for both calls:

```rust
let rolls = roller::roll(&roll_type, &mut rand::rng());
println!("{}", output::format_rolls(&roll_type, &rolls));
```

The `maps_arguments_to_roll_type` table needs one new case:
`&["d8+3d6+2"]` gives `RollType::Normal` with that notation.

### Output

`format_roll` now takes the scale rather than a modifier, since the signs
and constants travel with the rolled terms:

```rust
fn format_roll(roll: &Roll, scale: Option<Scale>) -> String
```

Rules for a roll line:

- Dice terms show as their dice in brackets, as now.
- Constants show as their value.
- Terms after the first are joined with ` + ` or ` - ` by sign.
- A leading minus shows as `-` directly before the first term.
- With a scale and more than one term, the terms are wrapped in
  parentheses, so the scale visibly applies to the whole sum.
- The total follows ` = `.

| Case                         | Roll                               | Scale | Output                          |
| ---------------------------- | ---------------------------------- | ----- | ------------------------------- |
| Single dice term             | `[4, 2]`                           | none  | `[4, 2] = 6`                    |
| Dice and constant            | `[4, 2]`, `+3`                     | none  | `[4, 2] + 3 = 9`                |
| Several dice terms           | `[5]`, `+[3, 1, 6]`, `+2`          | none  | `[5] + [3, 1, 6] + 2 = 17`      |
| Subtracted constant          | `[5]`, `-2`                        | none  | `[5] - 2 = 3`                   |
| Subtracted dice              | `[5]`, `-[2]`                      | none  | `[5] - [2] = 3`                 |
| Leading minus                | `-2`, `+[5]`                       | none  | `-2 + [5] = 3`                  |
| Clamped total                | `[1]`, `-5`                        | none  | `[1] - 5 = 0`                   |
| Scale on one term            | `[4, 2]`                           | `*2`  | `[4, 2] * 2 = 12`               |
| Scale on several terms       | `[4, 2]`, `+3`                     | `*2`  | `([4, 2] + 3) * 2 = 18`         |

`format_rolls` passes `notation.scale()` where it passed
`notation.modifier()`. Its existing tests need their `Roll` values updating
to the new shape; add one compound case, such as `d8+3d6+2` giving
`d8+3d6+2: [5] + [3, 1, 6] + 2 = 17`.

`format_ability_score` unpacks the single dice term with the slice pattern
from step 8. Its expected output doesn't change.

### Help text

Update `NOTATION_HELP` to describe the new grammar: a chain of dice and
constants joined by `+` or `-`, with an optional `*` or `/` at the end that
applies to the whole total. Include an example such as `d8+3d6+2`.

### End-to-end tests

| Case                        | Arguments           | Expected                                      |
| --------------------------- | ------------------- | --------------------------------------------- |
| Compound notation           | `"d8 + 3d6 + 2"`    | Success; stdout starts with `d8+3d6+2: [`     |
| Constants only              | `3+2`               | Exit code 2; stderr contains `no dice`        |
| Scale not at the end        | `d6*2+3`            | Exit code 2                                   |
| Compound critical           | `-c d8+2d6+3`       | Success; stdout contains `(Critical!)`        |
| Help describes the grammar  | `--help`            | Stdout contains `d8+3d6+2`                    |

## Order of work

1. Errors: add the new variants and `#[non_exhaustive]`.
2. Tokeniser, with its tests.
3. `Sign`, `TermKind`, `Term`; rename `Modifier` to `Scale`.
4. The new `Notation` structure and `try_new`, with the `Copy` removal.
   Expect a long list of compiler errors across the crate; fixing them is
   mostly adding `&` and `.clone()`.
5. Parser, then rewire `from_str`; update and extend the `from_str` tests.
6. `Display` and the round-trip test.
7. Roller: `RolledTerm`, the new arithmetic, `&RollType`.
8. CLI: arguments, output, help text, end-to-end tests.
9. Remove `regex`, run `cargo clippy --all-targets`, update the README.

## Releasing

`Modifier`, `Operator`, `Roll` and several method signatures change, so
this is a breaking change to the library. On a 0.x version, signal that by
bumping the minor version: `0.1.0` becomes `0.2.0`. See
[Semantic Versioning](https://semver.org/#spec-item-4) for the 0.x rules.

Record the changes in a `CHANGELOG.md` before tagging; the
[Keep a Changelog](https://keepachangelog.com/) format is common in Rust
projects. Then commit, tag `v0.2.0`, and run `cargo publish` as before.

## Further reading

- [The Rust Programming Language](https://doc.rust-lang.org/book/), chapters
  4 (ownership), 6 (enums and patterns) and 13 (iterators and closures).
- [`Peekable`](https://doc.rust-lang.org/std/iter/struct.Peekable.html) and
  the [`Iterator`](https://doc.rust-lang.org/std/iter/trait.Iterator.html)
  trait's method list.
- [Patterns and matching](https://doc.rust-lang.org/book/ch19-00-patterns.html),
  covering range patterns, `..` in struct patterns and slice patterns.
- Crafting Interpreters on
  [scanning](https://craftinginterpreters.com/scanning.html) and
  [parsing](https://craftinginterpreters.com/parsing-expressions.html), the
  tokeniser and parser techniques in depth.