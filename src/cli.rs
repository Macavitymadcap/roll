//! Parsing of command line arguments and formatting output

use clap::{ArgGroup, Parser};
use roll::notation::{Modifier, Notation, Operator};
use roll::roller::{Roll, RollType};

/// Roll dice using tabletop RPG notation, following the D&D 5e (2014) rules.
#[derive(Parser, Debug)]
#[command(version, about)]
#[command(group(
  ArgGroup::new("mode").args(["advantage", "critical", "disadvantage", "scores"])
))]
pub struct Cli {
    /// Roll at advantage: twice, highest first
    #[arg(short, long)]
    pub advantage: bool,

    /// Roll a critical hit: double the dice, apply the modifier once
    #[arg(short, long, requires = "notation")]
    pub critical: bool,

    /// Roll at disadvantage: twice, lowest first
    #[arg(short, long)]
    pub disadvantage: bool,

    /// Roll six ability scores: 4d6 each, dropping the lowest die
    #[arg(short, long, conflicts_with = "notation")]
    pub scores: bool,

    /// Dice notation, e.g 2d6+3. Quote if it contains whitespace: "2d6 + 3"
    #[arg(value_name = "NOTATION")]
    pub notation: Option<Notation>,
}

impl Cli {
    pub fn roll_type(&self) -> RollType {
        let notation = self.notation.unwrap_or(Notation::d20(None));
        if self.advantage {
            return RollType::Advantage(notation);
        } else if self.disadvantage {
            return RollType::Disadvantage(notation);
        } else if self.critical {
            return RollType::Critical(notation);
        } else if self.scores {
            return RollType::AbilityScores;
        }

        RollType::Normal(notation)
    }
}

fn format_roll(roll: Roll, modifier: Option<Modifier>) -> String {
    let mut output = String::new();

    let dice = roll
        .dice
        .iter()
        .map(|die| die.to_string())
        .collect::<Vec<_>>()
        .join(", ");

    output.push_str(format!("[{}]", dice).as_str());

    if let Some(m) = modifier {
        output.push_str(format!(" {} {}", m.operator(), m.value()).as_str());
    }

    output.push_str(format!(" = {}", roll.total).as_str());

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;
    use clap::error::{Error, ErrorKind, RichFormatter};
    use rstest::rstest;

    fn parse(args: &[&str]) -> Result<Cli, Error<RichFormatter>> {
        Cli::try_parse_from(std::iter::once("roll").chain(args.iter().copied()))
    }

    #[test]
    fn cli_definition_is_valid() {
        Cli::command().debug_assert()
    }
    mod parse {
        use super::*;

        #[rstest]
        #[case::advantage_and_disadvantage(&["-a", "-d"], ErrorKind::ArgumentConflict)]
        #[case::scores_with_notation(&["-s", "2d6"], ErrorKind::ArgumentConflict)]
        #[case::critical_without_notation(&["-c"], ErrorKind::MissingRequiredArgument)]
        #[case::invalid_notation(&["banana"], ErrorKind::ValueValidation)]
        #[case::unknown_flag(&["-x"], ErrorKind::UnknownArgument)]
        #[case::unquoted_notation(&["2d6", "+", "3"], ErrorKind::UnknownArgument)]
        fn rejects_invalid_arguments(#[case] args: &[&str], #[case] expected: ErrorKind) {
            let result = parse(args).unwrap_err().kind();

            assert_eq!(result, expected);
        }

        #[test]
        fn validation_error_includes_notation_error_message() {
            let args = &["not a roll"];

            let result = parse(args).unwrap_err().to_string();

            assert!(result.contains("invalid dice notation"));
        }

        #[rstest]
        #[case::help(&["-h"], ErrorKind::DisplayHelp)]
        #[case::version(&["-V"], ErrorKind::DisplayVersion)]
        fn error_to_display_other_text(#[case] args: &[&str], #[case] expected: ErrorKind) {
            let result = parse(args).unwrap_err().kind();

            assert_eq!(result, expected);
        }
    }

    mod roll_type {
        use super::*;

        #[rstest]
        #[case::normal(&[], RollType::Normal(Notation::d20(None)))]
        #[case::normal_with_notation(&["2d6+3"], RollType::Normal("2d6+3".parse().unwrap()))]
        #[case::normal_with_whitespace(&["2 d6 + 3"], RollType::Normal("2d6+3".parse().unwrap()))]
        #[case::advantage(&["-a"], RollType::Advantage(Notation::d20(None)))]
        #[case::advantage_long(&["--advantage"], RollType::Advantage(Notation::d20(None)))]
        #[case::disadvantage(&["-d", "d20+5"], RollType::Disadvantage("d20+5".parse().unwrap()))]
        #[case::disadvantage_long(&["--disadvantage", "d20+5"], RollType::Disadvantage("d20+5".parse().unwrap()))]
        #[case::critical(&["-c", "1d8+2"], RollType::Critical("1d8+2".parse().unwrap()))]
        #[case::critical_long(&["--critical", "1d8+2"], RollType::Critical("1d8+2".parse().unwrap()))]
        #[case::scores(&["-s"], RollType::AbilityScores)]
        #[case::scores_long(&["--scores"], RollType::AbilityScores)]
        fn maps_arguments_to_roll_type(#[case] args: &[&str], #[case] expected: RollType) {
            let result = parse(args).unwrap().roll_type();

            assert_eq!(result, expected);
        }
    }

    mod format_roll {
        use super::*;

        #[rstest]
        #[case::no_modifier(Roll {dice: vec![4, 2], total: 6}, None, String::from("[4, 2] = 6"))]
        #[case::with_modifier(Roll {dice: vec![19], total: 22}, Some(Modifier::try_new(Operator::Add, 3).unwrap()), "[19] + 3 = 22")]
        fn returns_expected_string(
            #[case] roll: Roll,
            #[case] modifier: Option<Modifier>,
            #[case] expected: String,
        ) {
            let result = format_roll(roll, modifier);

            assert_eq!(result, expected);
        }
    }
}
