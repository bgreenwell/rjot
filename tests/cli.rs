use assert_cmd::Command;
use chrono::{Datelike, Local};
use predicates::prelude::*;
use std::fs;
use std::path::PathBuf;
use tempfile::{tempdir, TempDir};

type TestResult = Result<(), Box<dyn std::error::Error>>;

// Helper function to set up a test environment
fn setup() -> (TempDir, PathBuf) {
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let rjot_dir = temp_dir.path().to_path_buf();
    // Ensure the base directory and entries subdir exist for tests
    fs::create_dir_all(rjot_dir.join("entries")).expect("Failed to create entries dir");
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
        .args(["--tags", "rust,project"])
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
        .args(["tag", "add", "--last=1", "rust", "testing"])
        .env("RJOT_DIR", &rjot_dir)
        .assert()
        .success();

    Command::cargo_bin("rjot")?
        .args(["show", "--last"])
        .env("RJOT_DIR", &rjot_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("- rust"))
        .stdout(predicate::str::contains("- testing"));

    Command::cargo_bin("rjot")?
        .args(["tag", "rm", "--last=1", "testing"])
        .env("RJOT_DIR", &rjot_dir)
        .assert()
        .success();

    Command::cargo_bin("rjot")?
        .args(["show", "--last"])
        .env("RJOT_DIR", &rjot_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("- rust"))
        .stdout(predicate::str::contains("testing").not());

    Command::cargo_bin("rjot")?
        .args(["tag", "set", "--last=1", "final,done"])
        .env("RJOT_DIR", &rjot_dir)
        .assert()
        .success();

    Command::cargo_bin("rjot")?
        .args(["show", "--last"])
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

    // Create a note for today
    let today = Local::now().date_naive();
    fs::write(
        entries_dir.join(format!("{}-120000.md", today.format("%Y-%m-%d"))),
        "note for today",
    )?;

    // Create a note for the first day of this week
    let week_start = today - chrono::Duration::days(today.weekday().num_days_from_sunday() as i64);
    // Ensure the start of the week is not the same as today, unless today is Sunday
    if week_start != today {
        fs::write(
            entries_dir.join(format!("{}-120000.md", week_start.format("%Y-%m-%d"))),
            "note from start of week",
        )?;
    }

    // Create a note for a week ago that is NOT in the current week
    let week_ago = today - chrono::Duration::days(7);
    fs::write(
        entries_dir.join(format!("{}-120000.md", week_ago.format("%Y-%m-%d"))),
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

    // Test `week` and `--compile`
    let week_command = Command::cargo_bin("rjot")?
        .arg("week")
        .arg("--compile")
        .env("RJOT_DIR", &rjot_dir)
        .assert()
        .success();

    // Assert that both notes from this week are present
    week_command
        .stdout(predicate::str::contains("note for today"))
        .stdout(predicate::str::contains("note from start of week"));

    Ok(())
}

#[test]
fn test_new_with_template() -> TestResult {
    let (_temp_dir, rjot_dir) = setup();
    let templates_dir = rjot_dir.join("templates");
    fs::create_dir(&templates_dir)?;
    fs::write(templates_dir.join("daily.md"), "tags:\n  - daily")?;

    Command::cargo_bin("rjot")?
        .args(["new", "--template", "daily.md"])
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

#[test]
fn test_git_init_and_sync() -> TestResult {
    let (_temp_dir, rjot_dir) = setup();

    // 1. Init with git
    Command::cargo_bin("rjot")?
        .args(&["init", "--git"])
        .env("RJOT_DIR", &rjot_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("Initialized a new Git repository"));

    assert!(rjot_dir.join(".git").exists());
    assert!(rjot_dir.join(".gitignore").exists());

    // 2. Set up a bare repository to act as a remote
    let remote_dir = tempdir()?;
    git2::Repository::init_bare(remote_dir.path())?;
    let local_repo = git2::Repository::open(&rjot_dir)?;
    local_repo.remote("origin", remote_dir.path().to_str().unwrap())?;

    // 3. Create a note and sync
    Command::cargo_bin("rjot")?
        .arg("a note to be synced")
        .env("RJOT_DIR", &rjot_dir)
        .assert()
        .success();

    Command::cargo_bin("rjot")?
        .arg("sync")
        .env("RJOT_DIR", &rjot_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("Sync complete."));

    // 4. Verify the commit exists in the "remote" repo
    let remote_repo = git2::Repository::open_bare(remote_dir.path())?;
    let head = remote_repo.head()?.peel_to_commit()?;
    assert!(head.message().unwrap().contains("rjot sync"));

    Ok(())
}

#[test]
fn test_encryption_and_decryption_lifecycle() -> TestResult {
    let (_temp_dir, rjot_dir) = setup();
    let entries_dir = rjot_dir.join("entries");

    // 1. Init with encryption
    Command::cargo_bin("rjot")?
        .args(&["init", "--encrypt"])
        .env("RJOT_DIR", &rjot_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Generated new encryption identity",
        ));

    assert!(rjot_dir.join("identity.txt").exists());
    assert!(rjot_dir.join("config.toml").exists());

    // 2. Create an encrypted note
    Command::cargo_bin("rjot")?
        .arg("this is a secret")
        .env("RJOT_DIR", &rjot_dir)
        .assert()
        .success();

    // 3. Verify the file on disk is encrypted
    let entry_path = fs::read_dir(&entries_dir)?.next().unwrap()?.path();
    let raw_content = fs::read(&entry_path)?;
    assert!(raw_content.starts_with(b"age-encryption.org"));

    // 4. Verify rjot can read it transparently
    Command::cargo_bin("rjot")?
        .arg("show")
        .arg("--last")
        .env("RJOT_DIR", &rjot_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("this is a secret"));

    // 5. Decrypt the journal
    Command::cargo_bin("rjot")?
        .args(&["decrypt", "--force"])
        .env("RJOT_DIR", &rjot_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("Successfully decrypted journal"));

    // 6. Verify the file on disk is now plaintext
    let raw_content_after = fs::read_to_string(&entry_path)?;
    assert_eq!(raw_content_after, "this is a secret");

    // 7. Verify the identity and config files were removed
    assert!(!rjot_dir.join("identity.txt").exists());
    assert!(!rjot_dir.join("config.toml").exists());

    Ok(())
}

#[test]
fn test_list_count_override() -> TestResult {
    let (_temp_dir, rjot_dir) = setup();

    for i in 0..12 {
        Command::cargo_bin("rjot")?
            .arg(format!("note {}", i))
            .env("RJOT_DIR", &rjot_dir)
            .assert()
            .success();
        std::thread::sleep(std::time::Duration::from_millis(1200)); // Ensure different timestamps
    }

    // Default list should show 10
    let output = Command::cargo_bin("rjot")?
        .arg("list")
        .env("RJOT_DIR", &rjot_dir)
        .assert()
        .success();

    let stdout = String::from_utf8(output.get_output().stdout.clone())?;
    assert_eq!(
        stdout.lines().count(),
        13,
        "Expected 10 notes + 2 header lines + 1 space"
    );

    // Override to show 5
    let output_5 = Command::cargo_bin("rjot")?
        .arg("list")
        .arg("5")
        .env("RJOT_DIR", &rjot_dir)
        .assert()
        .success();

    let stdout_5 = String::from_utf8(output_5.get_output().stdout.clone())?;
    assert_eq!(
        stdout_5.lines().count(),
        8,
        "Expected 5 notes + 2 header lines + 1 space"
    );

    Ok(())
}
