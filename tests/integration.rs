use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_cli_workflow() {

    // Test: Add a bookmark
    let assert = Command::cargo_bin("arx")
        .expect("Failed to find arx binary")
        .arg("add")
        .arg("The C Programming Language")
        .arg("--category")
        .arg("book")
        .assert();
    assert
        .success()
        .stdout(predicate::str::contains("Bookmark with ID #1 successfully added!"));

    // Test: List bookmarks
    let assert = Command::cargo_bin("arx")
        .expect("Failed to find arx binary")
        .arg("ls")
        .assert();
    assert
        .success()
        .stdout(predicate::str::contains("The C Programming Language"));

    // Test: Mark bookmark as done
    let assert = Command::cargo_bin("arx")
        .expect("Failed to find arx binary")
        .arg("done")
        .arg("1")
        .assert();
    assert.success();
}
