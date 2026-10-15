use rand::{Rng, RngExt};
use std::cmp::Reverse;

use crate::notation::Notation;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RollType {
    Normal(Notation),
    Advantage(Notation),
    Disadvantage(Notation),
    Critical(Notation),
    AbilityScores,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Roll {
    pub dice: Vec<u32>,
    pub total: u32,
}

fn roll_dice(count: u32, sides: u32, rng: &mut impl Rng) -> Vec<u32> {
    (0..count).map(|_| rng.random_range(1..=sides)).collect()
}

fn roll_once(notation: Notation, rng: &mut impl Rng) -> Roll {
    let dice = roll_dice(notation.count, notation.sides, rng);
    let sum: u32 = dice.iter().sum();
    let total = notation.modifier.map_or(sum, |m| m.apply(sum));

    Roll { dice, total }
}

fn roll_ability_score(rng: &mut impl Rng) -> Roll {
    let dice = roll_dice(4, 6, rng);
    let sum: u32 = dice.iter().sum();
    let lowest = dice.iter().min().copied().unwrap_or(0);
    let total = sum - lowest;

    Roll { dice, total }
}

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

    use crate::notation::{Modifier, Operator};

    const RANDOM_SEED: u64 = 42;

    // roll_dice
    #[test]
    fn roll_dice_returns_vector_with_length_matching_count() {
        // Arrange
        let count = 3;
        let sides = 6;
        let mut rng = StdRng::seed_from_u64(RANDOM_SEED);

        // Act
        let result = roll_dice(count, sides, &mut rng);

        // Assert
        assert_eq!(result.len(), count as usize);
    }

    #[test]
    fn roll_dice_every_die_is_within_range_of_sides() {
        // Arrange
        let count = 100;
        let sides = 10;
        let mut rng = StdRng::seed_from_u64(RANDOM_SEED);

        // Act
        let result = roll_dice(count, sides, &mut rng);

        // Assert
        assert!(
            result.iter().all(|die| (1..=sides).contains(die)),
            "a die was out of range: {result:?}"
        )
    }

    #[test]
    fn roll_dice_every_possible_face_can_come_up() {
        // Arrange
        let count = 1_000;
        let sides = 6;
        let mut rng = StdRng::seed_from_u64(RANDOM_SEED);

        // Act
        let result = roll_dice(count, sides, &mut rng);

        // Assert
        let missing: Vec<u32> = (1..6).filter(|face| !result.contains(face)).collect();
        assert!(missing.is_empty(), "faces never rolled: {missing:?}");
    }

    // roll_once
    #[test]
    fn roll_once_rolls_count_dice() {
        // Arrange
        let notation = Notation {
            count: 2,
            sides: 6,
            modifier: None,
        };
        let mut rng = StdRng::seed_from_u64(RANDOM_SEED);

        // Act
        let result = roll_once(notation, &mut rng);

        // Assert
        assert_eq!(result.dice.len(), 2);
    }

    #[test]
    fn roll_once_applies_modifier_to_total() {
        // Arrange
        let notation = Notation {
            count: 3,
            sides: 1,
            modifier: Some(Modifier {
                value: 2,
                operator: Operator::Add,
            }),
        };
        let mut rng = StdRng::seed_from_u64(RANDOM_SEED);

        // Act
        let result = roll_once(notation, &mut rng);

        // Assert
        assert_eq!(result.total, 5);
    }

    #[test]
    fn roll_once_total_is_sum_of_dice_with_no_modifier() {
        // Arrange
        let notation = Notation {
            count: 2,
            sides: 6,
            modifier: None,
        };
        let mut rng = StdRng::seed_from_u64(RANDOM_SEED);

        // Act
        let result = roll_once(notation, &mut rng);

        // Assert
        assert_eq!(result.total, result.dice.iter().sum::<u32>());
    }

    #[test]
    fn roll_once_leaves_dice_unmodified_when_applying_modifier() {
        // Arrange
        let notation = Notation {
            count: 1,
            sides: 1,
            modifier: Some(Modifier {
                operator: Operator::Add,
                value: 5,
            }),
        };
        let mut rng = StdRng::seed_from_u64(RANDOM_SEED);

        // Act
        let result = roll_once(notation, &mut rng);

        // Assert
        assert_eq!(result.dice, vec![1]);
        assert_eq!(result.total, 6);
    }

    #[test]
    fn roll_once_bottoms_out_subtraction_at_zero() {
        // Arrange
        let notation = Notation {
            count: 1,
            sides: 1,
            modifier: Some(Modifier {
                operator: Operator::Sub,
                value: 5,
            }),
        };
        let mut rng = StdRng::seed_from_u64(RANDOM_SEED);

        // Act
        let result = roll_once(notation, &mut rng);

        // Assert
        assert_eq!(result.total, 0);
    }

    // roll_ability_score
    #[test]
    fn ability_score_rolls_four_dice() {
        // Arrange
        let mut rng = StdRng::seed_from_u64(42);

        // Act
        let result = roll_ability_score(&mut rng);

        // Assert
        assert_eq!(result.dice.len(), 4);
    }

    #[test]
    fn ability_score_dice_are_d6() {
        // Arrange
        let mut rng = StdRng::seed_from_u64(42);

        // Act
        let result = roll_ability_score(&mut rng);

        // Assert
        assert!(
            result.dice.iter().all(|die| (1..=6).contains(die)),
            "a die was outside 1..=6: {:?}",
            result.dice
        );
    }

    #[test]
    fn ability_score_total_drops_the_lowest_die() {
        // Arrange
        let mut rng = StdRng::seed_from_u64(42);

        // Act
        let result = roll_ability_score(&mut rng);

        // Assert
        let sum: u32 = result.dice.iter().sum();
        let lowest = *result.dice.iter().min().unwrap();
        assert_eq!(result.total, sum - lowest);
    }

    #[test]
    fn ability_score_total_is_between_3_and_18() {
        // Arrange
        let mut rng = StdRng::seed_from_u64(42);

        // Act
        let result = roll_ability_score(&mut rng);

        // Assert
        assert!(
            (3..=18).contains(&result.total),
            "total out of range: {}",
            result.total
        );
    }
    // roll Normal
    #[test]
    fn roll_normal_returns_a_vector_with_one_roll_when() {
        // Arrange
        let roll_type = RollType::Normal(Notation {
            count: 2,
            sides: 6,
            modifier: None,
        });
        let mut rng = StdRng::seed_from_u64(RANDOM_SEED);

        // Act
        let result = roll(roll_type, &mut rng);

        // Assert
        assert_eq!(result.len(), 1);
        let roll = &result[0];
        assert_eq!(roll.dice.len(), 2);
        assert!((2..=12).contains(&roll.total));
    }

    // roll Critical
    #[test]
    fn roll_critical_doubles_original_dice() {
        // Arrange
        let roll_type = RollType::Critical(Notation {
            count: 1,
            sides: 8,
            modifier: Some(Modifier {
                operator: Operator::Add,
                value: 2,
            }),
        });
        let mut rng = StdRng::seed_from_u64(RANDOM_SEED);

        // Act
        let result = roll(roll_type, &mut rng);

        // Assert
        assert_eq!(result.len(), 1);
        let roll = &result[0];
        assert_eq!(roll.dice.len(), 2);
    }

    #[test]
    fn roll_critical_applies_modifier_once() {
        // Arrange
        let roll_type = RollType::Critical(Notation {
            count: 1,
            sides: 1,
            modifier: Some(Modifier {
                operator: Operator::Add,
                value: 2,
            }),
        });

        let mut rng = StdRng::seed_from_u64(RANDOM_SEED);

        // Act
        let result = roll(roll_type, &mut rng);

        // Assert
        assert_eq!(result[0].total, 4);
    }

    // roll Advantage
    #[test]
    fn roll_advantage_returns_two_rolls_of_count_dice() {
        // Arrange
        let roll_type = RollType::Advantage(Notation {
            count: 1,
            sides: 20,
            modifier: None,
        });
        let mut rng = StdRng::seed_from_u64(RANDOM_SEED);

        // Act
        let result = roll(roll_type, &mut rng);

        // Assert
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].dice.len(), 1);
        assert_eq!(result[0].dice.len(), 1);
    }

    #[test]
    fn roll_advantage_returns_rolls_sorted_highest_to_lowest() {
        // Arrange
        let roll_type = RollType::Advantage(Notation {
            count: 1,
            sides: 20,
            modifier: Some(Modifier {
                operator: Operator::Add,
                value: 3,
            }),
        });
        let mut rng = StdRng::seed_from_u64(RANDOM_SEED);

        // Act
        let result = roll(roll_type, &mut rng);

        // Assert
        assert!(result[0].total >= result[1].total)
    }

    // roll Disadvantage
    #[test]
    fn roll_disadvantage_returns_two_rolls_of_count_dice() {
        // Arrange
        let roll_type = RollType::Disadvantage(Notation {
            count: 1,
            sides: 20,
            modifier: None,
        });
        let mut rng = StdRng::seed_from_u64(RANDOM_SEED);

        // Act
        let result = roll(roll_type, &mut rng);

        // Assert
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].dice.len(), 1);
        assert_eq!(result[0].dice.len(), 1);
    }

    #[test]
    fn roll_disadvantage_returns_rolls_sorted_highest_to_lowest() {
        // Arrange
        let roll_type = RollType::Disadvantage(Notation {
            count: 1,
            sides: 20,
            modifier: Some(Modifier {
                operator: Operator::Add,
                value: 3,
            }),
        });
        let mut rng = StdRng::seed_from_u64(RANDOM_SEED);

        // Act
        let result = roll(roll_type, &mut rng);

        // Assert
        assert!(result[0].total <= result[1].total)
    }

    // roll AbilityScores
    #[test]
    fn roll_ability_scores_returns_six_rolls() {
        // Arrange
        let roll_type = RollType::AbilityScores;
        let mut rng = StdRng::seed_from_u64(RANDOM_SEED);

        // Act
        let result = roll(roll_type, &mut rng);

        // Assert
        assert_eq!(result.len(), 6);
    }
}
