use assert_cmd::Command; // Brings in `CommandCargoExt` for `main_binary`
use predicates::prelude::*;
use std::fs;
use std::path::PathBuf;
use tempfile::{tempdir, TempDir};

type TestResult = Result<(), Box<dyn std::error::Error>>;

// Helper function to set up a test environment
fn setup() -> (TempDir, PathBuf) {
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let rjot_dir = temp_dir.path().to_path_buf();
    (temp_dir, rjot_dir)
}

#[test]
fn test_default_jot_creation() -> TestResult {
    let (_temp_dir, rjot_dir) = setup(); // _temp_dir ensures the dir is cleaned up

    let mut cmd = Command::cargo_bin("rjot")?;
    cmd.arg("a default note")
        .env("RJOT_DIR", &rjot_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("Jotting down:"));

    let entries_dir = rjot_dir.join("entries");
    assert_eq!(fs::read_dir(entries_dir)?.count(), 1);
    Ok(())
}

#[test]
fn test_misspelled_command_is_a_note() -> TestResult {
    let (_temp_dir, rjot_dir) = setup();

    let mut cmd = Command::cargo_bin("rjot")?;
    cmd.arg("lisy") // A typo for "list"
        .env("RJOT_DIR", &rjot_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("Jotting down: \"lisy\""));

    let entries_dir = rjot_dir.join("entries");
    assert_eq!(
        fs::read_dir(entries_dir)?.count(),
        1,
        "Expected a note to be created from the typo"
    );

    Ok(())
}

#[test]
fn test_tagged_jot_creation() -> TestResult {
    let (_temp_dir, rjot_dir) = setup();

    let mut cmd = Command::cargo_bin("rjot")?;
    cmd.arg("a tagged note")
        .args(&["--tags", "rust,project"])
        .env("RJOT_DIR", &rjot_dir)
        .assert()
        .success();

    let entries_dir = rjot_dir.join("entries");
    let entry_path = fs::read_dir(entries_dir)?.next().unwrap()?.path();
    let content = fs::read_to_string(entry_path)?;

    assert!(content.contains("tags:"));
    assert!(content.contains("- rust"));
    assert!(content.contains("- project"));
    Ok(())
}

#[test]
fn test_list_and_find() -> TestResult {
    let (_temp_dir, rjot_dir) = setup();

    // Create a note
    Command::cargo_bin("rjot")?
        .arg("note about a unique_keyword")
        .env("RJOT_DIR", &rjot_dir)
        .assert()
        .success();

    // Test list
    Command::cargo_bin("rjot")?
        .arg("list")
        .env("RJOT_DIR", &rjot_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("unique_keyword"));

    // Test find
    Command::cargo_bin("rjot")?
        .arg("find")
        .arg("unique_keyword")
        .env("RJOT_DIR", &rjot_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("unique_keyword"));

    Ok(())
}

#[test]
fn test_show_edit_delete_last() -> TestResult {
    let (temp_dir, rjot_dir) = setup();

    // Create two notes
    Command::cargo_bin("rjot")?
        .arg("first note")
        .env("RJOT_DIR", &rjot_dir)
        .assert()
        .success();
    std::thread::sleep(std::time::Duration::from_secs(1)); // Ensure different timestamp
    Command::cargo_bin("rjot")?
        .arg("second note")
        .env("RJOT_DIR", &rjot_dir)
        .assert()
        .success();

    // Test show --last
    Command::cargo_bin("rjot")?
        .arg("show")
        .arg("--last")
        .env("RJOT_DIR", &rjot_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("second note"));

    // Test edit --last=2
    let editor_script_content = "#!/bin/sh\necho 'edited content' > \"$1\"";
    let script_path = temp_dir.path().join("editor.sh");
    fs::write(&script_path, editor_script_content)?;
    fs::set_permissions(
        &script_path,
        std::os::unix::fs::PermissionsExt::from_mode(0o755),
    )?;

    Command::cargo_bin("rjot")?
        .arg("edit")
        .arg("--last=2")
        .env("RJOT_DIR", &rjot_dir)
        .env("EDITOR", &script_path)
        .assert()
        .success();

    let entries_dir = rjot_dir.join("entries");
    let mut entries: Vec<_> = fs::read_dir(&entries_dir)?
        .map(|r| r.unwrap().path())
        .collect();
    entries.sort();
    let first_note_content = fs::read_to_string(&entries[0])?;
    assert!(first_note_content.contains("edited content"));

    // Test delete --last
    Command::cargo_bin("rjot")?
        .arg("delete")
        .arg("--last")
        .arg("--force")
        .env("RJOT_DIR", &rjot_dir)
        .assert()
        .success();

    let entries_dir = rjot_dir.join("entries");
    assert_eq!(
        fs::read_dir(entries_dir)?.count(),
        1,
        "Expected one jot to remain."
    );

    Ok(())
}

#[test]
fn test_info_command() -> TestResult {
    let (_temp_dir, rjot_dir) = setup();

    // Test info --paths
    Command::cargo_bin("rjot")?
        .arg("info")
        .arg("--paths")
        .env("RJOT_DIR", &rjot_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("Entries:"))
        .stdout(predicate::str::contains("Templates:"));

    // Test info --stats
    Command::cargo_bin("rjot")?
        .arg("info")
        .arg("--stats")
        .env("RJOT_DIR", &rjot_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("Total jots: 0"));

    Ok(())
}
