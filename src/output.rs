//! Formatting of command line output.

use rust_roll::notation::Modifier;
use rust_roll::roller::{Roll, RollType};
use std::cmp::Reverse;
use std::string::ToString;

fn join_dice(dice: &[u32]) -> String {
    dice.iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(", ")
}

fn format_roll(roll: &Roll, modifier: Option<Modifier>) -> String {
    let dice = join_dice(&roll.dice);

    let modifier = modifier
        .map(|m| format!(" {} {}", m.operator(), m.value()))
        .unwrap_or_default();

    format!("[{dice}]{modifier} = {}", roll.total)
}

fn format_ability_score(roll: &Roll) -> String {
    let mut dice = roll.dice.clone();
    dice.sort_by_key(|&die| Reverse(die));
    let (lowest, rest) = dice.split_last().expect("an ability score always has dice");

    let rest = join_dice(rest);

    format!("[{rest}, ({lowest})] = {}", roll.total)
}

fn format_pair(kept: &Roll, dropped: &Roll, modifier: Option<Modifier>) -> String {
    let kept_string = format_roll(kept, modifier);
    let dropped_string = format_roll(dropped, modifier);

    format!("{kept_string}, dropped {dropped_string}")
}

/// Returns a formatted string of the given `RollType` and rolls.
///
/// # Examples
///
/// ```
/// use crate::notation::RollType;
/// use crate::roller::Roll;
///
/// let roll_type = RollType::Normal("d20+2".parse().unwrap())
/// let rolls = vec![Roll { dice: vec![15], total: 17 }]
///
/// let result = format_rolls(roll_type, rolls);
///
/// assert_eq!(result, "d20+2: [15] + 2 = 17");
/// ```
pub fn format_rolls(roll_type: &RollType, rolls: &[Roll]) -> String {
    match (roll_type, rolls) {
        (RollType::Normal(notation), [roll]) => {
            format!("{notation}: {}", format_roll(roll, notation.modifier()))
        }
        (RollType::Critical(notation), [roll]) => {
            format!(
                "{notation} (Critical!): {}",
                format_roll(roll, notation.modifier())
            )
        }
        (RollType::Advantage(notation), [kept, dropped]) => {
            format!(
                "{notation} (Advantage): {}",
                format_pair(kept, dropped, notation.modifier())
            )
        }
        (RollType::Disadvantage(notation), [kept, dropped]) => {
            format!(
                "{notation} (Disadvantage): {}",
                format_pair(kept, dropped, notation.modifier())
            )
        }
        (RollType::AbilityScores, scores) => scores
            .iter()
            .map(format_ability_score)
            .collect::<Vec<_>>()
            .join("\n"),
        _ => unreachable!("roll type and number of rolls do not match"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
    use rust_roll::notation::Operator;

    mod format_roll {
        use super::*;

        #[rstest]
        #[case::no_modifier(Roll {dice: vec![4, 2], total: 6}, None, "[4, 2] = 6")]
        #[case::with_modifier(Roll {dice: vec![19], total: 22}, Some(Modifier::try_new(Operator::Add, 3).unwrap()), "[19] + 3 = 22")]
        fn returns_expected_string(
            #[case] roll: Roll,
            #[case] modifier: Option<Modifier>,
            #[case] expected: &str,
        ) {
            let result = format_roll(&roll, modifier);

            assert_eq!(result, expected);
        }
    }

    mod format_ability_score {
        use super::*;

        #[test]
        fn shows_dice_highest_first_with_lowest_marked() {
            let roll = Roll {
                dice: vec![2, 1, 4, 5],
                total: 11,
            };

            let result = format_ability_score(&roll);

            assert_eq!(result, "[5, 4, 2, (1)] = 11");
        }

        #[test]
        fn ties_mark_exactly_one_die() {
            let roll = Roll {
                dice: vec![3, 3, 5, 6],
                total: 14,
            };

            let result = format_ability_score(&roll);

            assert_eq!(result, "[6, 5, 3, (3)] = 14");
        }
    }

    mod format_pair {
        use super::*;

        #[rstest]
        fn returns_expected_string() {
            let kept = Roll {
                dice: vec![19],
                total: 21,
            };
            let dropped = Roll {
                dice: vec![16],
                total: 18,
            };
            let modifier = Modifier::try_new(Operator::Add, 2).unwrap();

            let result = format_pair(&kept, &dropped, Some(modifier));

            assert_eq!(result, "[19] + 2 = 21, dropped [16] + 2 = 18");
        }
    }

    mod format_rolls {
        use super::*;

        #[test]
        fn normal_displays_one_line_prefixed_by_notation() {
            let roll_type = RollType::Normal("3d4+3".parse().unwrap());
            let rolls = vec![Roll {
                dice: vec![2, 3, 1],
                total: 9,
            }];

            let result = format_rolls(&roll_type, &rolls);

            assert_eq!(result, "3d4+3: [2, 3, 1] + 3 = 9");
        }

        #[test]
        fn critical_labelled_showing_original_notation() {
            let roll_type = RollType::Critical("2d6+2".parse().unwrap());
            let rolls = vec![Roll {
                dice: vec![4, 6, 2, 3],
                total: 17,
            }];

            let result = format_rolls(&roll_type, &rolls);

            assert_eq!(result, "2d6+2 (Critical!): [4, 6, 2, 3] + 2 = 17");
        }

        #[test]
        fn advantage_shows_both_rolls_labelled_with_kept_first() {
            let roll_type = RollType::Advantage("d20+5".parse().unwrap());
            let rolls = vec![
                Roll {
                    dice: vec![12],
                    total: 17,
                },
                Roll {
                    dice: vec![4],
                    total: 9,
                },
            ];

            let result = format_rolls(&roll_type, &rolls);

            assert_eq!(
                result,
                "d20+5 (Advantage): [12] + 5 = 17, dropped [4] + 5 = 9"
            );
        }

        #[test]
        fn disadvantage_shows_both_rolls_labelled_with_kept_first() {
            let roll_type = RollType::Disadvantage("d20+3".parse().unwrap());
            let rolls = vec![
                Roll {
                    dice: vec![7],
                    total: 10,
                },
                Roll {
                    dice: vec![15],
                    total: 18,
                },
            ];

            let result = format_rolls(&roll_type, &rolls);

            assert_eq!(
                result,
                "d20+3 (Disadvantage): [7] + 3 = 10, dropped [15] + 3 = 18"
            );
        }

        #[test]
        fn scores_displays_six_lines() {
            let roll_type = RollType::AbilityScores;
            let rolls = vec![
                Roll {
                    dice: vec![2, 5, 3, 4],
                    total: 12,
                },
                Roll {
                    dice: vec![6, 1, 2, 2],
                    total: 10,
                },
                Roll {
                    dice: vec![1, 1, 1, 2],
                    total: 4,
                },
                Roll {
                    dice: vec![5, 4, 1, 1],
                    total: 10,
                },
                Roll {
                    dice: vec![6, 6, 6, 6],
                    total: 18,
                },
                Roll {
                    dice: vec![5, 2, 4, 3],
                    total: 12,
                },
            ];

            let result = format_rolls(&roll_type, &rolls);

            let expected = [
                "[5, 4, 3, (2)] = 12",
                "[6, 2, 2, (1)] = 10",
                "[2, 1, 1, (1)] = 4",
                "[5, 4, 1, (1)] = 10",
                "[6, 6, 6, (6)] = 18",
                "[5, 4, 3, (2)] = 12",
            ]
            .join("\n");

            assert_eq!(result, expected);
        }
    }
}
