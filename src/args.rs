//! Parsing of command line arguments.

use clap::{ArgGroup, Parser};
use rust_roll::notation::Notation;
use rust_roll::roller::RollType;

const NOTATION_HELP: &str = "\
Notation:
  XdY?Z, where:
    X   number of dice; optional, defaults to 1
    dY  a die with Y sides; d or D
    ?   operator: +, -, * or /; optional
    Z   value the operator applies to the total

  The modifier applies to the total, not to each die. Division rounds down
  and totals never go below zero.

  Whitespace between parts is allowed. Quote the notation so the shell
  passes it as one argument: roll \"2d6 + 3\"";

/// Roll dice using tabletop RPG notation, following the D&D 5e (2014) rules.
#[expect(
    clippy::struct_excessive_bools,
    reason = "one clap flag per mode; ArgGroup makes them exclusive and roll_type converts them to RollType"
)]
#[derive(Parser, Debug)]
#[command(version, after_help = "Run with --help for details of dice notation", after_long_help = NOTATION_HELP)]
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
    /// Maps the input flag to the `RollType` enum.
    #[must_use]
    pub fn roll_type(&self) -> RollType {
        let notation = self.notation.unwrap_or(Notation::d20(None));
        if self.advantage {
            RollType::Advantage(notation)
        } else if self.disadvantage {
            RollType::Disadvantage(notation)
        } else if self.critical {
            RollType::Critical(notation)
        } else if self.scores {
            RollType::AbilityScores
        } else {
            RollType::Normal(notation)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;
    use clap::error::ErrorKind;
    use rstest::rstest;

    fn parse_args(args: &[&str]) -> Result<Cli, clap::Error> {
        Cli::try_parse_from(std::iter::once("roll").chain(args.iter().copied()))
    }

    #[test]
    fn cli_definition_is_valid() {
        Cli::command().debug_assert();
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
            let result = parse_args(args).unwrap_err().kind();

            assert_eq!(result, expected);
        }

        #[test]
        fn validation_error_includes_notation_error_message() {
            let args = &["not a roll"];

            let result = parse_args(args).unwrap_err().to_string();

            assert!(result.contains("invalid dice notation"));
        }

        #[rstest]
        #[case::help(&["-h"], ErrorKind::DisplayHelp)]
        #[case::help_long(&["--help"], ErrorKind::DisplayHelp)]
        #[case::version(&["-V"], ErrorKind::DisplayVersion)]
        #[case::version_long(&["--version"], ErrorKind::DisplayVersion)]
        fn help_and_version_stop_parsing(#[case] args: &[&str], #[case] expected: ErrorKind) {
            let result = parse_args(args).unwrap_err().kind();

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
            let result = parse_args(args).unwrap().roll_type();

            assert_eq!(result, expected);
        }
    }
}
