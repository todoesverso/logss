use assert_cmd::Command;
use predicates::prelude::*;
use std::path::Path;

#[test]
fn non_valid_arg() {
    let mut cmd = Command::cargo_bin("logss").unwrap();
    cmd.arg("-e").arg("--non-valid-arg");

    cmd.assert()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("Error: non valid arguments"));
}

#[test]
fn show_help() {
    let mut cmd = Command::cargo_bin("logss").unwrap();
    cmd.arg("-h");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains(
            "Simple CLI command to display logs in a user-friendly way

Usage: logss [OPTIONS]

Options:
  -c <CONTAINERS>  Specify substrings (regex patterns)
  -e               Exit on empty input [default: false]
  -s               Start in single view mode [default: false]
  -C <COMMAND>     Get input from a command
  -f <FILE>        Input configuration file (overrides CLI arguments)
  -o <OUTPUT_PATH> Specify the output path for matched patterns
  -r <RENDER>      Define render speed in milliseconds [default: 100]
  -V               Start in vertical view mode
  -h               Print help
",
        ))
        .stderr(predicate::str::is_empty());
}

#[test]
#[ignore]
fn simple_piped_run() {
    let mut cmd = Command::cargo_bin("logss").unwrap();
    let c_path = Path::new("README.md");
    cmd.pipe_stdin(c_path).unwrap();
    cmd.arg("-e")
        .arg("-c")
        .arg("version")
        .arg("-c")
        .arg("package")
        .arg("-r")
        .arg("25");

    cmd.assert()
        .success()
        .stderr(predicate::str::contains("package"))
        .stderr(predicate::str::contains("version"))
        .stderr(predicate::str::contains("name").not())
        .stdout(predicate::str::is_empty());
}

#[test]
#[ignore]
fn simple_command_run() {
    let mut cmd = Command::cargo_bin("logss").unwrap();
    cmd.arg("-e")
        .arg("-c")
        .arg("version")
        .arg("-c")
        .arg("package")
        .arg("-C")
        .arg("cat Cargo.toml")
        .arg("-r")
        .arg("25");

    cmd.assert()
        .success()
        .stderr(predicate::str::contains("package"))
        .stderr(predicate::str::contains("version"))
        .stderr(predicate::str::contains("name").not())
        .stdout(predicate::str::is_empty());
}
