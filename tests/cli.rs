use assert_cmd::Command;
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
    let (_temp_dir, rjot_dir) = setup();

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
    cmd.arg("lisy")
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

    Command::cargo_bin("rjot")?
        .arg("note about a unique_keyword")
        .env("RJOT_DIR", &rjot_dir)
        .assert()
        .success();

    Command::cargo_bin("rjot")?
        .arg("list")
        .env("RJOT_DIR", &rjot_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("unique_keyword"));

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
fn test_show_edit_delete() -> TestResult {
    let (temp_dir, rjot_dir) = setup();

    // Create notes and get the first one's ID
    Command::cargo_bin("rjot")?
        .arg("first note")
        .env("RJOT_DIR", &rjot_dir)
        .assert()
        .success();
    std::thread::sleep(std::time::Duration::from_secs(1));
    Command::cargo_bin("rjot")?
        .arg("second note")
        .env("RJOT_DIR", &rjot_dir)
        .assert()
        .success();

    let mut entries: Vec<_> = fs::read_dir(rjot_dir.join("entries"))?
        .map(|r| r.unwrap().path())
        .collect();
    entries.sort();
    let first_note_id = entries[0].file_stem().unwrap().to_str().unwrap();

    // Test edit with ID prefix
    let script_path;
    #[cfg(unix)]
    {
        script_path = temp_dir.path().join("editor.sh");
        fs::write(&script_path, "#!/bin/sh\necho 'edited content' > \"$1\"")?;
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&script_path, fs::Permissions::from_mode(0o755))?;
    }
    #[cfg(windows)]
    {
        script_path = temp_dir.path().join("editor.bat");
        fs::write(&script_path, "@echo edited content > %1")?;
    }

    Command::cargo_bin("rjot")?
        .arg("edit")
        .arg(first_note_id)
        .env("RJOT_DIR", &rjot_dir)
        .env("EDITOR", &script_path)
        .assert()
        .success();

    // Verify edit
    let first_note_content = fs::read_to_string(&entries[0])?;
    assert!(first_note_content.contains("edited content"));

    // Test delete with --last
    Command::cargo_bin("rjot")?
        .arg("delete")
        .arg("--last")
        .arg("--force")
        .env("RJOT_DIR", &rjot_dir)
        .assert()
        .success();

    assert_eq!(
        fs::read_dir(rjot_dir.join("entries"))?.count(),
        1,
        "Expected one jot to remain."
    );

    Ok(())
}

#[test]
fn test_info_command() -> TestResult {
    let (_temp_dir, rjot_dir) = setup();

    Command::cargo_bin("rjot")?
        .arg("info")
        .arg("--paths")
        .env("RJOT_DIR", &rjot_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("Entries:"))
        .stdout(predicate::str::contains("Templates:"));

    Command::cargo_bin("rjot")?
        .arg("info")
        .arg("--stats")
        .env("RJOT_DIR", &rjot_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("Total jots: 0"));

    Ok(())
}

#[test]
fn test_tag_management() -> TestResult {
    let (_temp_dir, rjot_dir) = setup();

    Command::cargo_bin("rjot")?
        .arg("note for tags")
        .env("RJOT_DIR", &rjot_dir)
        .assert()
        .success();

    Command::cargo_bin("rjot")?
        .args(&["tag", "add", "--last=1", "rust", "testing"])
        .env("RJOT_DIR", &rjot_dir)
        .assert()
        .success();

    Command::cargo_bin("rjot")?
        .args(&["show", "--last"])
        .env("RJOT_DIR", &rjot_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("- rust"))
        .stdout(predicate::str::contains("- testing"));

    Command::cargo_bin("rjot")?
        .args(&["tag", "rm", "--last=1", "testing"])
        .env("RJOT_DIR", &rjot_dir)
        .assert()
        .success();

    Command::cargo_bin("rjot")?
        .args(&["show", "--last"])
        .env("RJOT_DIR", &rjot_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("- rust"))
        .stdout(predicate::str::contains("testing").not());

    Command::cargo_bin("rjot")?
        .args(&["tag", "set", "--last=1", "final,done"])
        .env("RJOT_DIR", &rjot_dir)
        .assert()
        .success();

    Command::cargo_bin("rjot")?
        .args(&["show", "--last"])
        .env("RJOT_DIR", &rjot_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("- final"))
        .stdout(predicate::str::contains("- done"))
        .stdout(predicate::str::contains("rust").not());

    Ok(())
}

#[test]
fn test_editor_fallback() -> TestResult {
    let (_temp_dir, rjot_dir) = setup();

    Command::cargo_bin("rjot")?
        .arg("new")
        .env("RJOT_DIR", &rjot_dir)
        .env_remove("EDITOR")
        .env("PATH", "")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Could not find a default editor"));

    Ok(())
}

#[test]
fn test_time_based_commands_and_compile() -> TestResult {
    let (_temp_dir, rjot_dir) = setup();
    let entries_dir = rjot_dir.join("entries");
    fs::create_dir_all(&entries_dir)?; // Ensure the entries directory exists

    // Create a note for today and a note for a week ago
    let today_str = chrono::Local::now().format("%Y-%m-%d").to_string();
    fs::write(
        entries_dir.join(format!("{}-120000.md", today_str)),
        "note for today",
    )?;

    let week_ago = chrono::Local::now().date_naive() - chrono::Duration::days(7);
    let week_ago_str = week_ago.format("%Y-%m-%d").to_string();
    fs::write(
        entries_dir.join(format!("{}-120000.md", week_ago_str)),
        "note from a week ago",
    )?;

    // Test `today`
    Command::cargo_bin("rjot")?
        .arg("today")
        .env("RJOT_DIR", &rjot_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("note for today"))
        .stdout(predicate::str::contains("note from a week ago").not());

    // Test `on` with a specific date
    Command::cargo_bin("rjot")?
        .arg("on")
        .arg(&week_ago_str)
        .env("RJOT_DIR", &rjot_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("note from a week ago"));

    // Test `week` and `--compile`
    Command::cargo_bin("rjot")?
        .arg("week")
        .arg("--compile")
        .env("RJOT_DIR", &rjot_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("#").count(1)) // Should contain one compiled note
        .stdout(predicate::str::contains("note for today"));

    Ok(())
}

#[test]
fn test_new_with_template() -> TestResult {
    let (_temp_dir, rjot_dir) = setup();
    let templates_dir = rjot_dir.join("templates");
    fs::create_dir(&templates_dir)?;
    fs::write(templates_dir.join("daily.md"), "tags:\n  - daily")?;

    Command::cargo_bin("rjot")?
        .args(&["new", "--template", "daily.md"])
        .env("RJOT_DIR", &rjot_dir)
        .env("EDITOR", "true") // Use `true` as a no-op editor
        .assert()
        .success();

    let entries_dir = rjot_dir.join("entries");
    let entry_path = fs::read_dir(entries_dir)?.next().unwrap()?.path();
    let content = fs::read_to_string(entry_path)?;
    assert!(content.contains("- daily"));

    Ok(())
}
