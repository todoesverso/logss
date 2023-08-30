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
            "Simple cli command to show logs in a friendly way

Usage: logss [OPTIONS]

Options:
  -c <CONTAINERS>  Finds the substring (regexp)
  -e               Exit on empty input [default: false]
  -C <COMMAND>     Gets input from this command
  -f <FILE>        Input config file (overrides cli arguments)
  -o <OUTPUT_PATH> If defined, files with matched patters will be created
  -r <RENDER>      Defines render speed in milliseconds [default: 100]
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
