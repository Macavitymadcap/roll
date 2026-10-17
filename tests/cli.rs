use assert_cmd::Command;
use predicates::prelude::*;

fn roll() -> Command {
    Command::cargo_bin("roll").unwrap()
}

#[test]
fn rolls_the_given_notation() {
    roll()
        .arg("2d6+3")
        .assert()
        .success()
        .stdout(predicate::str::starts_with("2d6+3: ["));
}

#[test]
fn rolls_with_default_dice() {
    roll()
        .assert()
        .success()
        .stdout(predicate::str::starts_with("d20: ["));
}

#[test]
fn rolls_the_passed_mode() {
    roll()
        .args(["-a", "d20+3"])
        .assert()
        .success()
        .stdout(predicate::str::starts_with("d20+3 (Advantage): ["));
}

#[test]
fn rolls_ability_scores() {
    let output = roll()
        .arg("-s")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let stdout = String::from_utf8(output).unwrap();

    assert_eq!(stdout.lines().count(), 6);
}

#[test]
fn quoted_notation_with_spaces_succeeds() {
    roll()
        .arg("4 d10 - 2")
        .assert()
        .success()
        .stdout(predicate::str::starts_with("4d10-2: ["));
}

#[test]
fn ivalid_notation_fails() {
    roll()
        .arg("foobar")
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid dice notation"))
        .code(predicate::eq(2));
}

#[test]
fn unquoted_notation_fails() {
    roll()
        .args(["d20", "+", "2"])
        .assert()
        .failure()
        .code(predicate::eq(2));
}

#[test]
fn conflicting_arguments_fail() {
    roll()
        .args(["-a", "-d"])
        .assert()
        .failure()
        .code(predicate::eq(2));
}

#[test]
fn help_flag_succeeds() {
    roll()
        .arg("-h")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Roll dice using tabletop RPG notation",
        ));
}

#[test]
fn version_flag_succeeds() {
    let expected_version = env!("CARGO_PKG_VERSION");
    roll()
        .arg("-V")
        .assert()
        .success()
        .stdout(predicate::str::contains(expected_version));
}
