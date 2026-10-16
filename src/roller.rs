//! Rolling dice from a [`Notation`], following the D&D 5e (2014) rules.
//!
//! The entry point is [`roll`], which takes a [`RollType`] describing what
//! to roll and how. Randomness comes from a generator passed in by the
//! caller, so results can be made repeatable in tests by using a seeded
//! generator.

use rand::{Rng, RngExt};
use std::cmp::Reverse;

use crate::notation::Notation;

/// What to roll, and how.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RollType {
    /// Roll the notation once.
    Normal(Notation),
    /// Roll the notation twice, returning the results highest first.
    Advantage(Notation),
    /// Roll the notation twice, returning the results lowest first.
    Disadvantage(Notation),
    /// Roll the notation as a critical hit: double the dice, apply the
    /// modifier once. See [`Notation::critical`].
    Critical(Notation),
    /// Roll six ability scores, each 4d6 with the lowest die dropped.
    AbilityScores,
}

/// The result of a single roll.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Roll {
    dice: Vec<u32>,
    total: u32,
}

impl Roll {
    /// Every die rolled, in the order rolled, without the modifier. For
    /// ability scores, this includes the dropped die.
    pub fn dice(&self) -> &[u32] {
        &self.dice
    }

    /// The final result, after applying any modifier or dropping the
    /// lowest die.
    pub fn total(&self) -> &u32 {
        &self.total
    }
}

fn roll_dice(count: u32, sides: u32, rng: &mut impl Rng) -> Vec<u32> {
    (0..count).map(|_| rng.random_range(1..=sides)).collect()
}

fn roll_once(notation: Notation, rng: &mut impl Rng) -> Roll {
    let dice = roll_dice(notation.count(), notation.sides(), rng);
    let sum: u32 = dice.iter().sum();
    let total = notation.modifier().map_or(sum, |m| m.apply(sum));

    Roll { dice, total }
}

fn roll_ability_score(rng: &mut impl Rng) -> Roll {
    let dice = roll_dice(4, 6, rng);
    let sum: u32 = dice.iter().sum();
    let lowest = dice.iter().min().copied().unwrap_or(0);
    let total = sum - lowest;

    Roll { dice, total }
}

/// Rolls dice as described by `roll_type`, using `rng` for randomness.
///
/// Returns one [`Roll`] for normal and critical rolls, two for advantage
/// and disadvantage, and six for ability scores.
///
/// # Examples
///
/// ```
/// use roll::notation::Notation;
/// use roll::roller::{roll, RollType};
///
/// let notation: Notation = "2d6".parse().unwrap();
/// let rolls = roll(RollType::Normal(notation), &mut rand::rng());
///
/// assert_eq!(rolls.len(), 1);
/// assert_eq!(rolls[0].dice().len(), 2);
/// ```
pub fn roll(roll_type: RollType, rng: &mut impl Rng) -> Vec<Roll> {
    match roll_type {
        RollType::Normal(notation) => vec![roll_once(notation, rng)],
        RollType::Critical(notation) => vec![roll_once(notation.critical(), rng)],
        RollType::Advantage(notation) => {
            let mut rolls = vec![roll_once(notation, rng), roll_once(notation, rng)];
            rolls.sort_by_key(|roll| Reverse(roll.total));

            rolls
        }
        RollType::Disadvantage(notation) => {
            let mut rolls = vec![roll_once(notation, rng), roll_once(notation, rng)];
            rolls.sort_by_key(|roll| roll.total);

            rolls
        }
        RollType::AbilityScores => (0..6).map(|_| roll_ability_score(rng)).collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{SeedableRng, rngs::StdRng};
    use rstest::{fixture, rstest};

    use crate::notation::Operator;
    use crate::test_helpers::{modifier, notation};

    const RANDOM_SEED: u64 = 42;

    #[fixture]
    fn rng() -> StdRng {
        StdRng::seed_from_u64(RANDOM_SEED)
    }

    mod roll_dice {
        use super::*;

        #[rstest]
        fn returns_count_dice(mut rng: StdRng) {
            let count = 3;
            let sides = 6;

            let result = roll_dice(count, sides, &mut rng);

            assert_eq!(result.len(), count as usize);
        }

        #[rstest]
        fn every_die_is_within_range_of_sides(mut rng: StdRng) {
            let count = 100;
            let sides = 10;

            let result = roll_dice(count, sides, &mut rng);

            assert!(
                result.iter().all(|die| (1..=sides).contains(die)),
                "a die was out of range: {result:?}"
            )
        }

        #[rstest]
        fn every_possible_face_can_come_up(mut rng: StdRng) {
            let count = 1_000;
            let sides = 6;

            let result = roll_dice(count, sides, &mut rng);

            let missing: Vec<u32> = (1..=6).filter(|face| !result.contains(face)).collect();
            assert!(missing.is_empty(), "faces never rolled: {missing:?}");
        }
    }

    mod roll_once {
        use super::*;

        #[rstest]
        fn rolls_count_dice(mut rng: StdRng) {
            let notation = notation(2, 6, None);

            let result = roll_once(notation, &mut rng);

            assert_eq!(result.dice.len(), 2);
        }

        #[rstest]
        fn applies_modifier_to_total(mut rng: StdRng) {
            let notation = notation(3, 1, Some(modifier(Operator::Add, 2)));

            let result = roll_once(notation, &mut rng);

            assert_eq!(result.total, 5);
        }

        #[rstest]
        fn total_is_sum_of_dice_with_no_modifier(mut rng: StdRng) {
            let notation = notation(2, 6, None);

            let result = roll_once(notation, &mut rng);

            assert_eq!(result.total, result.dice.iter().sum::<u32>());
        }

        #[rstest]
        fn leaves_dice_unmodified_when_applying_modifier(mut rng: StdRng) {
            let notation = notation(1, 1, Some(modifier(Operator::Add, 5)));

            let result = roll_once(notation, &mut rng);

            assert_eq!(result.dice, vec![1]);
            assert_eq!(result.total, 6);
        }
    }

    mod roll_ability_score {
        use super::*;
        #[rstest]
        fn rolls_four_dice(mut rng: StdRng) {
            let result = roll_ability_score(&mut rng);

            assert_eq!(result.dice.len(), 4);
        }

        #[rstest]
        fn dice_have_six_sides(mut rng: StdRng) {
            let result = roll_ability_score(&mut rng);

            assert!(
                result.dice.iter().all(|die| (1..=6).contains(die)),
                "a die was outside 1..=6: {:?}",
                result.dice
            );
        }

        #[rstest]
        fn total_drops_the_lowest_die(mut rng: StdRng) {
            let result = roll_ability_score(&mut rng);

            let sum: u32 = result.dice.iter().sum();
            let lowest = *result.dice.iter().min().unwrap();
            assert_eq!(result.total, sum - lowest);
        }

        #[rstest]
        fn total_is_between_3_and_18(mut rng: StdRng) {
            let result = roll_ability_score(&mut rng);

            assert!(
                (3..=18).contains(&result.total),
                "total out of range: {}",
                result.total
            );
        }
    }

    mod roll {
        use super::*;

        #[rstest]
        fn normal_returns_one_roll(mut rng: StdRng) {
            let roll_type = RollType::Normal(notation(2, 6, None));

            let result = roll(roll_type, &mut rng);

            assert_eq!(result.len(), 1);
            let roll = &result[0];
            assert_eq!(roll.dice.len(), 2);
            assert!((2..=12).contains(&roll.total));
        }

        #[rstest]
        fn critical_doubles_original_dice(mut rng: StdRng) {
            let roll_type = RollType::Critical(notation(1, 8, Some(modifier(Operator::Add, 2))));

            let result = roll(roll_type, &mut rng);

            assert_eq!(result.len(), 1);
            let roll = &result[0];
            assert_eq!(roll.dice.len(), 2);
        }

        #[rstest]
        fn critical_applies_modifier_once(mut rng: StdRng) {
            let roll_type = RollType::Critical(notation(1, 1, Some(modifier(Operator::Add, 2))));

            let result = roll(roll_type, &mut rng);

            assert_eq!(result[0].total, 4);
        }

        #[rstest]
        fn advantage_returns_two_rolls_of_count_dice(mut rng: StdRng) {
            let roll_type = RollType::Advantage(notation(1, 20, None));
            let result = roll(roll_type, &mut rng);

            assert_eq!(result.len(), 2);
            assert_eq!(result[0].dice.len(), 1);
            assert_eq!(result[1].dice.len(), 1);
        }

        #[rstest]
        fn advantage_returns_rolls_sorted_highest_to_lowest(mut rng: StdRng) {
            let roll_type = RollType::Advantage(notation(1, 20, Some(modifier(Operator::Add, 2))));

            let result = roll(roll_type, &mut rng);

            assert!(result[0].total >= result[1].total)
        }

        #[rstest]
        fn disadvantage_returns_two_rolls_of_count_dice(mut rng: StdRng) {
            let roll_type = RollType::Disadvantage(notation(1, 20, None));

            let result = roll(roll_type, &mut rng);

            assert_eq!(result.len(), 2);
            assert_eq!(result[0].dice.len(), 1);
            assert_eq!(result[1].dice.len(), 1);
        }

        #[rstest]
        fn disadvantage_returns_rolls_sorted_lowest_to_highest(mut rng: StdRng) {
            let roll_type =
                RollType::Disadvantage(notation(1, 20, Some(modifier(Operator::Add, 2))));

            let result = roll(roll_type, &mut rng);

            assert!(result[0].total <= result[1].total)
        }

        #[rstest]
        fn ability_scores_returns_six_rolls(mut rng: StdRng) {
            let result = roll(RollType::AbilityScores, &mut rng);

            assert_eq!(result.len(), 6);
        }
    }
}
