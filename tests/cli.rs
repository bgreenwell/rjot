use assert_cmd::Command;
use chrono::Local;
use predicates::prelude::*;
use std::fs;
use std::path::PathBuf;
use tempfile::{tempdir, TempDir};

type TestResult = Result<(), Box<dyn std::error::Error>>;

// Helper function to set up a test environment
// This creates the `notebooks/default` structure to ensure all
// existing tests run in the default notebook context.
fn setup() -> (TempDir, PathBuf) {
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let rjot_dir = temp_dir.path().to_path_buf();
    // All tests will now run inside the 'default' notebook by default.
    fs::create_dir_all(rjot_dir.join("notebooks").join("default"))
        .expect("Failed to create default notebook dir");
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

    // Verify the note was created in the `default` notebook.
    let entries_dir = rjot_dir.join("notebooks").join("default");
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

    let entries_dir = rjot_dir.join("notebooks").join("default");
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

    let entries_dir = rjot_dir.join("notebooks").join("default");
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

    let mut entries: Vec<_> = fs::read_dir(rjot_dir.join("notebooks").join("default"))?
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
        fs::read_dir(rjot_dir.join("notebooks").join("default"))?.count(),
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
        .stdout(predicate::str::contains("Active Notebook:  default"))
        .stdout(predicate::str::contains("Entries:"));

    Command::cargo_bin("rjot")?
        .arg("info")
        .arg("--stats")
        .env("RJOT_DIR", &rjot_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Stats for active notebook: 'default'",
        ))
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

    Ok(())
}

#[test]
fn test_time_based_commands_and_compile() -> TestResult {
    let (_temp_dir, rjot_dir) = setup();
    let entries_dir = rjot_dir.join("notebooks").join("default");

    // Create a note for today
    let today = Local::now().date_naive();
    fs::write(
        entries_dir.join(format!("{}-120000.md", today.format("%Y-%m-%d"))),
        "note for today",
    )?;

    // Test `today`
    Command::cargo_bin("rjot")?
        .arg("today")
        .env("RJOT_DIR", &rjot_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("note for today"));

    Ok(())
}

#[test]
fn test_git_init_and_sync() -> TestResult {
    let (_temp_dir, rjot_dir) = setup();

    // 1. Init with git
    Command::cargo_bin("rjot")?
        .args(["init", "--git"])
        .env("RJOT_DIR", &rjot_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("Initialized a new Git repository"));

    assert!(rjot_dir.join(".git").exists());
    assert!(rjot_dir.join(".gitignore").exists());

    Ok(())
}

// Test module for notebooks
#[cfg(test)]
mod notebooks {
    use super::*;

    #[test]
    fn test_notebook_creation_and_list() -> TestResult {
        let (_temp_dir, rjot_dir) = setup();

        // Create a new notebook
        Command::cargo_bin("rjot")?
            .args(["notebook", "new", "work"])
            .env("RJOT_DIR", &rjot_dir)
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "Successfully created new notebook: 'work'",
            ));

        assert!(rjot_dir.join("notebooks").join("work").exists());

        // List notebooks
        Command::cargo_bin("rjot")?
            .args(["notebook", "list"])
            .env("RJOT_DIR", &rjot_dir)
            .assert()
            .success()
            .stdout(predicate::str::contains("* default"))
            .stdout(predicate::str::contains("  work"));

        Ok(())
    }

    #[test]
    fn test_notebook_status_and_use() -> TestResult {
        let (_temp_dir, rjot_dir) = setup();

        // Default status
        Command::cargo_bin("rjot")?
            .args(["notebook", "status"])
            .env("RJOT_DIR", &rjot_dir)
            .assert()
            .success()
            .stdout(predicate::str::contains("Active notebook: default"));

        // Status with env var
        Command::cargo_bin("rjot")?
            .args(["notebook", "status"])
            .env("RJOT_DIR", &rjot_dir)
            .env("RJOT_ACTIVE_NOTEBOOK", "personal")
            .assert()
            .success()
            .stdout(predicate::str::contains("Active notebook: personal"));

        // `use` command should print the export command
        Command::cargo_bin("rjot")?
            .args(["notebook", "new", "project-x"])
            .env("RJOT_DIR", &rjot_dir)
            .assert()
            .success();

        Command::cargo_bin("rjot")?
            .args(["notebook", "use", "project-x"])
            .env("RJOT_DIR", &rjot_dir)
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "export RJOT_ACTIVE_NOTEBOOK=\"project-x\"",
            ));

        Ok(())
    }

    #[test]
    fn test_jotting_in_different_notebooks() -> TestResult {
        let (_temp_dir, rjot_dir) = setup();
        Command::cargo_bin("rjot")?
            .args(["notebook", "new", "work"])
            .env("RJOT_DIR", &rjot_dir)
            .assert()
            .success();

        // Jot in default
        Command::cargo_bin("rjot")?
            .arg("a personal note")
            .env("RJOT_DIR", &rjot_dir)
            .assert()
            .success();

        // Added sleep to prevent filename collision due to second-level timestamp resolution.
        std::thread::sleep(std::time::Duration::from_millis(1200));

        // Jot in work notebook using --notebook flag
        Command::cargo_bin("rjot")?
            .arg("a work note")
            .args(["--notebook", "work"])
            .env("RJOT_DIR", &rjot_dir)
            .assert()
            .success();

        std::thread::sleep(std::time::Duration::from_millis(1200));

        // Jot in work notebook using env var
        Command::cargo_bin("rjot")?
            .arg("another work note")
            .env("RJOT_DIR", &rjot_dir)
            .env("RJOT_ACTIVE_NOTEBOOK", "work")
            .assert()
            .success();

        // Verify counts
        assert_eq!(
            fs::read_dir(rjot_dir.join("notebooks").join("default"))?.count(),
            1
        );
        assert_eq!(
            fs::read_dir(rjot_dir.join("notebooks").join("work"))?.count(),
            2
        );

        // Verify `list` is scoped correctly
        Command::cargo_bin("rjot")?
            .arg("list")
            .env("RJOT_DIR", &rjot_dir)
            .env("RJOT_ACTIVE_NOTEBOOK", "work")
            .assert()
            .success()
            .stdout(predicate::str::contains("a work note"))
            .stdout(predicate::str::contains("another work note"))
            .stdout(predicate::str::contains("a personal note").not());

        Ok(())
    }

    #[test]
    fn test_legacy_migration() -> TestResult {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let rjot_dir = temp_dir.path().to_path_buf();

        // 1. Create the legacy `entries` directory structure
        let legacy_entries = rjot_dir.join("entries");
        fs::create_dir_all(&legacy_entries)?;
        fs::write(legacy_entries.join("legacy_note.md"), "old note")?;

        // 2. Run any rjot command, which should trigger the migration
        Command::cargo_bin("rjot")?
            .arg("info")
            .arg("--paths")
            .env("RJOT_DIR", &rjot_dir)
            .assert()
            .success()
            .stdout(predicate::str::contains("Migrating your existing notes"));

        // 3. Verify the new structure
        assert!(
            !rjot_dir.join("entries").exists(),
            "Legacy entries dir should be gone"
        );
        let default_notebook = rjot_dir.join("notebooks").join("default");
        assert!(
            default_notebook.exists(),
            "Default notebook should be created"
        );
        assert!(
            default_notebook.join("legacy_note.md").exists(),
            "Legacy note should be moved"
        );

        Ok(())
    }

    #[test]
    fn test_info_stats_all_notebooks() -> TestResult {
        let (_temp_dir, rjot_dir) = setup();
        Command::cargo_bin("rjot")?
            .args(["notebook", "new", "work"])
            .env("RJOT_DIR", &rjot_dir)
            .assert()
            .success();

        // Add one note to default, two to work
        Command::cargo_bin("rjot")?
            .arg("note 1")
            .env("RJOT_DIR", &rjot_dir)
            .assert()
            .success();
        std::thread::sleep(std::time::Duration::from_millis(1200));
        Command::cargo_bin("rjot")?
            .arg("note 2")
            .args(["--notebook", "work"])
            .env("RJOT_DIR", &rjot_dir)
            .assert()
            .success();
        std::thread::sleep(std::time::Duration::from_millis(1200));
        Command::cargo_bin("rjot")?
            .arg("note 3")
            .args(["--notebook", "work"])
            .env("RJOT_DIR", &rjot_dir)
            .assert()
            .success();

        // Check stats for all
        Command::cargo_bin("rjot")?
            .args(["info", "--stats", "--all"])
            .env("RJOT_DIR", &rjot_dir)
            .assert()
            .success()
            .stdout(predicate::str::contains("Stats for all notebooks combined"))
            .stdout(predicate::str::contains("Total jots: 3"));

        Ok(())
    }

    #[test]
    fn test_tags_filter_is_scoped() -> TestResult {
        let (_temp_dir, rjot_dir) = setup();
        Command::cargo_bin("rjot")?
            .args(["notebook", "new", "work"])
            .env("RJOT_DIR", &rjot_dir)
            .assert()
            .success();

        // Create two notes with the same tag in different notebooks
        Command::cargo_bin("rjot")?
            .arg("personal task")
            .args(["--tags", "todo"])
            .env("RJOT_DIR", &rjot_dir)
            .assert()
            .success();
        std::thread::sleep(std::time::Duration::from_millis(1200));
        Command::cargo_bin("rjot")?
            .arg("work task")
            .args(["--tags", "todo", "--notebook", "work"])
            .env("RJOT_DIR", &rjot_dir)
            .assert()
            .success();

        // Filter in the 'work' notebook
        Command::cargo_bin("rjot")?
            .args(["tags", "todo", "--notebook", "work"])
            .env("RJOT_DIR", &rjot_dir)
            .assert()
            .success()
            .stdout(predicate::str::contains("work task"))
            .stdout(predicate::str::contains("personal task").not());

        Ok(())
    }

    #[test]
    fn test_new_with_template_in_notebook() -> TestResult {
        let (_temp_dir, rjot_dir) = setup();
        let templates_dir = rjot_dir.join("templates");
        fs::create_dir(&templates_dir)?;
        fs::write(templates_dir.join("meeting.md"), "## Meeting Notes")?;
        Command::cargo_bin("rjot")?
            .args(["notebook", "new", "work"])
            .env("RJOT_DIR", &rjot_dir)
            .assert()
            .success();

        Command::cargo_bin("rjot")?
            .args(["new", "--template", "meeting.md", "--notebook", "work"])
            .env("RJOT_DIR", &rjot_dir)
            .env("EDITOR", "true") // Use `true` as a no-op editor
            .assert()
            .success();

        let work_notebook = rjot_dir.join("notebooks").join("work");
        let entry_path = fs::read_dir(work_notebook)?.next().unwrap()?.path();
        let content = fs::read_to_string(entry_path)?;
        assert!(content.contains("## Meeting Notes"));

        Ok(())
    }

    #[test]
    fn test_find_is_scoped_and_global_search_works() -> TestResult {
        let (_temp_dir, rjot_dir) = setup();

        // 1. Create a couple of new notebooks
        Command::cargo_bin("rjot")?
            .args(["notebook", "new", "work"])
            .env("RJOT_DIR", &rjot_dir)
            .assert()
            .success();

        Command::cargo_bin("rjot")?
            .args(["notebook", "new", "personal"])
            .env("RJOT_DIR", &rjot_dir)
            .assert()
            .success();

        // 2. Create notes with a shared keyword in different notebooks
        // Note in 'default'
        Command::cargo_bin("rjot")?
            .arg("A note about a database_migration in default.")
            .env("RJOT_DIR", &rjot_dir)
            .assert()
            .success();
        std::thread::sleep(std::time::Duration::from_millis(1200));

        // Note in 'work'
        Command::cargo_bin("rjot")?
            .arg("Work note on the database_migration plan.")
            .args(["--notebook", "work"])
            .env("RJOT_DIR", &rjot_dir)
            .assert()
            .success();
        std::thread::sleep(std::time::Duration::from_millis(1200));

        // A note without the keyword
        Command::cargo_bin("rjot")?
            .arg("A personal note about something else.")
            .args(["--notebook", "personal"])
            .env("RJOT_DIR", &rjot_dir)
            .assert()
            .success();

        // 3. Test that a normal find is scoped to the active notebook ('default')
        Command::cargo_bin("rjot")?
            .args(["find", "database_migration"])
            .env("RJOT_DIR", &rjot_dir)
            .assert()
            .success()
            .stdout(predicate::str::contains("in default"))
            .stdout(predicate::str::contains("in work").not());

        // 4. Test that `find --all` searches across all notebooks
        Command::cargo_bin("rjot")?
            .args(["find", "database_migration", "--all"])
            .env("RJOT_DIR", &rjot_dir)
            .assert()
            .success()
            .stdout(predicate::str::contains("NOTEBOOK")) // Check for new header
            .stdout(predicate::str::contains("default"))
            .stdout(predicate::str::contains("work"))
            .stdout(predicate::str::contains("something else").not());

        Ok(())
    }
}

// Test module for error handling
#[cfg(test)]
mod error_handling {
    use super::*;

    #[test]
    fn test_fails_on_invalid_notebook_name() -> TestResult {
        let (_temp_dir, rjot_dir) = setup();
        Command::cargo_bin("rjot")?
            .args(["notebook", "new", "invalid/name"])
            .env("RJOT_DIR", &rjot_dir)
            .assert()
            .failure()
            .stderr(predicate::str::contains("Invalid notebook name"));
        Ok(())
    }

    #[test]
    fn test_fails_on_nonexistent_notebook_use() -> TestResult {
        let (_temp_dir, rjot_dir) = setup();
        Command::cargo_bin("rjot")?
            .args(["notebook", "use", "fake-notebook"])
            .env("RJOT_DIR", &rjot_dir)
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "Notebook 'fake-notebook' not found",
            ));
        Ok(())
    }

    #[test]
    fn test_fails_on_ambiguous_prefix() -> TestResult {
        let (_temp_dir, rjot_dir) = setup();
        let entries = rjot_dir.join("notebooks").join("default");
        fs::write(entries.join("2025-01-01-100000.md"), "note 1")?;
        fs::write(entries.join("2025-01-01-200000.md"), "note 2")?;

        Command::cargo_bin("rjot")?
            .args(["show", "2025-01-01"])
            .env("RJOT_DIR", &rjot_dir)
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "Prefix '2025-01-01' is not unique",
            ));
        Ok(())
    }

    #[test]
    fn test_fails_on_out_of_bounds_last() -> TestResult {
        let (_temp_dir, rjot_dir) = setup();
        Command::cargo_bin("rjot")?
            .arg("a single note")
            .env("RJOT_DIR", &rjot_dir)
            .assert()
            .success();

        Command::cargo_bin("rjot")?
            .args(["show", "--last=5"])
            .env("RJOT_DIR", &rjot_dir)
            .assert()
            .failure()
            .stderr(predicate::str::contains("Index out of bounds"));
        Ok(())
    }
}

// Test for full encryption feature
#[test]
fn test_full_encryption_and_decryption_lifecycle_across_notebooks() -> TestResult {
    let (_temp_dir, rjot_dir) = setup();

    // 1. Init with encryption
    Command::cargo_bin("rjot")?
        .args(["init", "--encrypt"])
        .env("RJOT_DIR", &rjot_dir)
        .assert()
        .success();

    // 2. Create a second notebook
    Command::cargo_bin("rjot")?
        .args(["notebook", "new", "secrets"])
        .env("RJOT_DIR", &rjot_dir)
        .assert()
        .success();

    // 3. Create encrypted notes in both notebooks
    Command::cargo_bin("rjot")?
        .arg("default secret")
        .env("RJOT_DIR", &rjot_dir)
        .assert()
        .success();
    std::thread::sleep(std::time::Duration::from_millis(1200));
    Command::cargo_bin("rjot")?
        .arg("special secret")
        .args(["--notebook", "secrets"])
        .env("RJOT_DIR", &rjot_dir)
        .assert()
        .success();

    // 4. Verify both files on disk are encrypted
    let default_note_path = fs::read_dir(rjot_dir.join("notebooks").join("default"))?
        .next()
        .unwrap()?
        .path();
    let secret_note_path = fs::read_dir(rjot_dir.join("notebooks").join("secrets"))?
        .next()
        .unwrap()?
        .path();
    assert!(fs::read(&default_note_path)?.starts_with(b"age-encryption.org"));
    assert!(fs::read(&secret_note_path)?.starts_with(b"age-encryption.org"));

    // 5. Verify rjot can read them transparently
    Command::cargo_bin("rjot")?
        .args(["show", "--last"])
        .env("RJOT_DIR", &rjot_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("default secret"));
    Command::cargo_bin("rjot")?
        .args(["show", "--last", "--notebook", "secrets"])
        .env("RJOT_DIR", &rjot_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("special secret"));

    // 6. Decrypt the entire journal
    Command::cargo_bin("rjot")?
        .args(["decrypt", "--force"])
        .env("RJOT_DIR", &rjot_dir)
        .assert()
        .success();

    // 7. Verify both files are now plaintext
    assert_eq!(fs::read_to_string(default_note_path)?, "default secret");
    assert_eq!(fs::read_to_string(secret_note_path)?, "special secret");

    // 8. Verify identity/config files are gone
    assert!(!rjot_dir.join("identity.txt").exists());
    assert!(!rjot_dir.join("config.toml").exists());

    Ok(())
}

// Test module for pinning feature.
#[cfg(test)]
mod pinning {
    use super::*;

    /// Tests the complete lifecycle of pinning and unpinning a note
    /// using the `--last` flag and by its ID prefix.
    #[test]
    fn test_pin_and_unpin_lifecycle() -> TestResult {
        let (_temp_dir, rjot_dir) = setup();

        // 1. Create a couple of notes to work with.
        Command::cargo_bin("rjot")?
            .arg("an unimportant note")
            .env("RJOT_DIR", &rjot_dir)
            .assert()
            .success();

        std::thread::sleep(std::time::Duration::from_millis(1200));

        Command::cargo_bin("rjot")?
            .arg("a very important note")
            .env("RJOT_DIR", &rjot_dir)
            .assert()
            .success();

        // 2. Pin the last note created.
        Command::cargo_bin("rjot")?
            .args(["pin", "--last"])
            .env("RJOT_DIR", &rjot_dir)
            .assert()
            .success()
            .stdout(predicate::str::contains("Successfully pinned jot"));

        // 3. Verify the `pinned: true` attribute exists in the file.
        let entries_dir = rjot_dir.join("notebooks").join("default");
        let mut entries: Vec<_> = fs::read_dir(entries_dir)?
            .map(|r| r.unwrap().path())
            .collect();
        entries.sort(); // Sort to get the most recent note last.
        let last_note_path = entries.last().unwrap();
        let last_note_content = fs::read_to_string(last_note_path)?;
        assert!(
            last_note_content.contains("pinned: true"),
            "The note should contain 'pinned: true' after pinning."
        );

        // 4. Unpin the same note using its ID prefix.
        let note_id = last_note_path.file_stem().unwrap().to_str().unwrap();
        Command::cargo_bin("rjot")?
            .args(["unpin", note_id])
            .env("RJOT_DIR", &rjot_dir)
            .assert()
            .success()
            .stdout(predicate::str::contains("Successfully unpinned jot"));

        // 5. Verify the `pinned` attribute is now gone from the file.
        let unpinned_content = fs::read_to_string(last_note_path)?;
        assert!(
            !unpinned_content.contains("pinned:"),
            "The 'pinned' key should be removed after unpinning."
        );

        Ok(())
    }

    /// Tests the `list --pinned` command.
    #[test]
    fn test_list_pinned_jots() -> TestResult {
        let (_temp_dir, rjot_dir) = setup();

        // 1. Create a mix of pinned and unpinned notes.
        Command::cargo_bin("rjot")?
            .arg("unpinned note 1")
            .env("RJOT_DIR", &rjot_dir)
            .assert()
            .success();
        std::thread::sleep(std::time::Duration::from_millis(1200));
        Command::cargo_bin("rjot")?
            .arg("pinned note 1")
            .env("RJOT_DIR", &rjot_dir)
            .assert()
            .success();
        Command::cargo_bin("rjot")?
            .args(["pin", "--last"])
            .env("RJOT_DIR", &rjot_dir)
            .assert()
            .success();
        std::thread::sleep(std::time::Duration::from_millis(1200));
        Command::cargo_bin("rjot")?
            .arg("unpinned note 2")
            .env("RJOT_DIR", &rjot_dir)
            .assert()
            .success();
        std::thread::sleep(std::time::Duration::from_millis(1200));
        Command::cargo_bin("rjot")?
            .arg("pinned note 2")
            .env("RJOT_DIR", &rjot_dir)
            .assert()
            .success();
        Command::cargo_bin("rjot")?
            .args(["pin", "--last"])
            .env("RJOT_DIR", &rjot_dir)
            .assert()
            .success();

        // 2. Run `list --pinned` and verify the output.
        Command::cargo_bin("rjot")?
            .args(["list", "--pinned"])
            .env("RJOT_DIR", &rjot_dir)
            .assert()
            .success()
            .stdout(predicate::str::contains("pinned note 1"))
            .stdout(predicate::str::contains("pinned note 2"))
            .stdout(predicate::str::contains("unpinned note").not());

        Ok(())
    }

    /// Tests that re-pinning an already pinned jot doesn't cause an error.
    #[test]
    fn test_pinning_is_idempotent() -> TestResult {
        let (_temp_dir, rjot_dir) = setup();

        Command::cargo_bin("rjot")?
            .arg("a note to be pinned repeatedly")
            .env("RJOT_DIR", &rjot_dir)
            .assert()
            .success();

        // Pin it once.
        Command::cargo_bin("rjot")?
            .args(["pin", "--last"])
            .env("RJOT_DIR", &rjot_dir)
            .assert()
            .success();

        // **MODIFICATION**: The assertion now correctly checks for the unique part of the confirmation message.
        // Pin it again. Should report that it's already pinned.
        Command::cargo_bin("rjot")?
            .args(["pin", "--last"])
            .env("RJOT_DIR", &rjot_dir)
            .assert()
            .success()
            .stdout(predicate::str::contains("is already pinned."));

        Ok(())
    }
}

// Test module for the task feature.
#[cfg(test)]
mod tasks {
    use super::*;

    /// Tests that the `task`, `todo`, and `t` subcommands all create a
    /// correctly formatted task jot.
    #[test]
    fn test_task_creation_and_aliases() -> TestResult {
        let (_temp_dir, rjot_dir) = setup();

        // 1. Test the main `task` command
        Command::cargo_bin("rjot")?
            .args(["task", "this is the main command"])
            .env("RJOT_DIR", &rjot_dir)
            .assert()
            .success();
        std::thread::sleep(std::time::Duration::from_millis(1200));

        // 2. Test the `todo` alias
        Command::cargo_bin("rjot")?
            .args(["todo", "this is the todo alias"])
            .env("RJOT_DIR", &rjot_dir)
            .assert()
            .success();
        std::thread::sleep(std::time::Duration::from_millis(1200));

        // 3. Test the `t` alias
        Command::cargo_bin("rjot")?
            .args(["t", "this is the t alias"])
            .env("RJOT_DIR", &rjot_dir)
            .assert()
            .success();

        // 4. Verify the contents of the created files
        let entries_dir = rjot_dir.join("notebooks").join("default");
        let mut entries: Vec<_> = fs::read_dir(entries_dir)?
            .map(|r| r.unwrap().path())
            .collect();
        entries.sort();

        assert_eq!(entries.len(), 3, "Expected three task jots to be created");

        let content1 = fs::read_to_string(&entries[0])?;
        let content2 = fs::read_to_string(&entries[1])?;
        let content3 = fs::read_to_string(&entries[2])?;

        assert!(content1.contains("- [ ] this is the main command"));
        assert!(content2.contains("- [ ] this is the todo alias"));
        assert!(content3.contains("- [ ] this is the t alias"));

        Ok(())
    }

    /// Tests the `list --tasks` command to ensure it only shows jots
    /// with incomplete tasks.
    #[test]
    fn test_list_tasks() -> TestResult {
        let (_temp_dir, rjot_dir) = setup();

        // 1. Create a jot with no tasks.
        Command::cargo_bin("rjot")?
            .arg("just a regular note")
            .env("RJOT_DIR", &rjot_dir)
            .assert()
            .success();
        std::thread::sleep(std::time::Duration::from_millis(1200));

        // 2. Create a jot with only completed tasks.
        let completed_task_path = rjot_dir
            .join("notebooks")
            .join("default")
            .join("2025-01-01-100000.md");
        fs::write(completed_task_path, "- [x] This task is done")?;
        std::thread::sleep(std::time::Duration::from_millis(1200));

        // 3. Create a jot with an incomplete task.
        Command::cargo_bin("rjot")?
            .args(["task", "this task is pending"])
            .env("RJOT_DIR", &rjot_dir)
            .assert()
            .success();

        // 4. Run `list --tasks` and verify the output.
        Command::cargo_bin("rjot")?
            .args(["list", "--tasks"])
            .env("RJOT_DIR", &rjot_dir)
            .assert()
            .success()
            .stdout(predicate::str::contains("this task is pending"))
            .stdout(predicate::str::contains("regular note").not())
            .stdout(predicate::str::contains("This task is done").not());

        Ok(())
    }

    /// Tests the `info --stats` command to verify task summary output.
    #[test]
    fn test_info_stats_with_tasks() -> TestResult {
        let (_temp_dir, rjot_dir) = setup();

        // 1. Create notes with a mix of pending and completed tasks.
        let entries_dir = rjot_dir.join("notebooks").join("default");
        fs::write(
            entries_dir.join("tasks1.md"),
            "- [ ] pending 1\n- [x] done 1",
        )?;
        fs::write(
            entries_dir.join("tasks2.md"),
            "- [ ] pending 2\n- [ ] pending 3",
        )?;
        fs::write(entries_dir.join("tasks3.md"), "- [x] done 2")?;

        // 2. Run `info --stats` and check the summary.
        Command::cargo_bin("rjot")?
            .args(["info", "--stats"])
            .env("RJOT_DIR", &rjot_dir)
            .assert()
            .success()
            .stdout(predicate::str::contains("Task Summary:"))
            .stdout(predicate::str::contains("Completed: 2"))
            .stdout(predicate::str::contains("Pending:   3"));

        Ok(())
    }
}

/// Test module for import/export feature
#[cfg(test)]
mod import_export {
    use super::*;

    #[test]
    fn test_export_and_import_zip() -> TestResult {
        let (_temp_dir, rjot_dir) = setup();
        let notebook_name = "zip-test-notebook";
        let output_zip = rjot_dir.join("export.zip");

        // 1. Create a notebook and a note
        Command::cargo_bin("rjot")?
            .args(["notebook", "new", notebook_name])
            .env("RJOT_DIR", &rjot_dir)
            .assert()
            .success();
        Command::cargo_bin("rjot")?
            .arg("a note for zip export")
            .args(["--notebook", notebook_name])
            .env("RJOT_DIR", &rjot_dir)
            .assert()
            .success();

        // 2. Export the notebook to a zip file
        Command::cargo_bin("rjot")?
            .args([
                "export",
                notebook_name,
                "--output",
                output_zip.to_str().unwrap(),
            ])
            .env("RJOT_DIR", &rjot_dir)
            .assert()
            .success();

        assert!(output_zip.exists());

        // 3. Import the notebook from the zip file
        Command::cargo_bin("rjot")?
            .args(["import", output_zip.to_str().unwrap()])
            .env("RJOT_DIR", &rjot_dir)
            .assert()
            .success();

        // 4. Verify the imported notebook and its content
        let imported_notebook_path = rjot_dir.join("notebooks").join("export"); // "export" from file stem
        assert!(imported_notebook_path.exists());
        assert_eq!(fs::read_dir(&imported_notebook_path)?.count(), 1);

        Ok(())
    }

    #[test]
    fn test_export_and_import_json() -> TestResult {
        let (_temp_dir, rjot_dir) = setup();
        let notebook_name = "json-test-notebook";
        let notebook_path = rjot_dir.join("notebooks").join(notebook_name); // Path to the original notebook
        let output_json = rjot_dir.join("export.json");

        // 1. Create a notebook and a note
        Command::cargo_bin("rjot")?
            .args(["notebook", "new", notebook_name])
            .env("RJOT_DIR", &rjot_dir)
            .assert()
            .success();
        Command::cargo_bin("rjot")?
            .arg("a note for json export")
            .args(["--notebook", notebook_name])
            .env("RJOT_DIR", &rjot_dir)
            .assert()
            .success();

        // 2. Export the notebook to a json file
        Command::cargo_bin("rjot")?
            .args([
                "export",
                notebook_name,
                "--format",
                "json",
                "--output",
                output_json.to_str().unwrap(),
            ])
            .env("RJOT_DIR", &rjot_dir)
            .assert()
            .success();

        assert!(output_json.exists());

        // 3. ✅ REMOVE the original notebook to simulate a restore
        fs::remove_dir_all(&notebook_path)?;
        assert!(
            !notebook_path.exists(),
            "Original notebook should be deleted before import"
        );

        // 4. Import the notebook from the json file. This should now succeed.
        Command::cargo_bin("rjot")?
            .args(["import", output_json.to_str().unwrap()])
            .env("RJOT_DIR", &rjot_dir)
            .assert()
            .success();

        // 5. Verify the imported notebook
        let imported_notebook_path = rjot_dir.join("notebooks").join(notebook_name);
        assert!(imported_notebook_path.exists());
        assert_eq!(fs::read_dir(&imported_notebook_path)?.count(), 1);

        Ok(())
    }
}

// Test module for the templating feature.
#[cfg(test)]
mod templating {
    use super::*;
    use git2::{Repository, Signature};
    use predicates::str::is_match;

    /// Tests the replacement of built-in variables like `{{date}}`,
    /// `{{branch}}`, `{{project_dir}}`, and `{{uuid}}`.
    #[test]
    fn test_built_in_template_variables() -> TestResult {
        let (temp_dir, rjot_dir) = setup();
        let templates_dir = rjot_dir.join("templates");
        fs::create_dir(&templates_dir)?;

        // 1. Initialize a git repo to test the {{branch}} variable.
        let repo = Repository::init(temp_dir.path())?;
        // The signature for the commit
        let signature = Signature::now("rjot-test", "test@rjot.com")?;
        // Create an empty tree for the initial commit
        let tree_id = repo.index()?.write_tree()?;
        let tree = repo.find_tree(tree_id)?;
        // Create the initial commit
        let oid = repo.commit(None, &signature, &signature, "Initial commit", &tree, &[])?;
        let commit = repo.find_commit(oid)?;
        repo.branch("main", &commit, false)?;
        repo.set_head("refs/heads/main")?; // Switch to the new branch

        // 2. Create a template file with all the built-in variables.
        let template_content =
            "Date: {{date}}\nBranch: {{branch}}\nProject: {{project_dir}}\nID: {{uuid}}";
        fs::write(templates_dir.join("built-in.md"), template_content)?;

        // 3. Run the `new` command with the template.
        Command::cargo_bin("rjot")?
            .current_dir(&temp_dir) // Run from inside the temp dir to get project_dir
            .args(["new", "--template", "built-in.md"])
            .env("RJOT_DIR", &rjot_dir)
            .env("EDITOR", "true") // No-op editor
            .assert()
            .success();

        // 4. Verify the output file.
        let entries_dir = rjot_dir.join("notebooks").join("default");
        let entry_path = fs::read_dir(entries_dir)?.next().unwrap()?.path();
        let content = fs::read_to_string(entry_path)?;

        assert!(content.contains("Branch: main"));
        assert!(content.contains(&format!(
            "Project: {}",
            temp_dir.path().file_name().unwrap().to_str().unwrap()
        )));
        assert!(is_match(r"ID: [0-9a-f]{8}-([0-9a-f]{4}-){3}[0-9a-f]{12}")
            .unwrap()
            .eval(&content));
        assert!(!content.contains("{{date}}")); // Just check it was replaced

        Ok(())
    }

    /// Tests the replacement of custom variables passed via the `-v` flag.
    #[test]
    fn test_custom_template_variables() -> TestResult {
        let (_temp_dir, rjot_dir) = setup();
        let templates_dir = rjot_dir.join("templates");
        fs::create_dir(&templates_dir)?;
        fs::write(
            templates_dir.join("custom.md"),
            "Ticket: {{ticket_id}}\nFeature: {{feature}}",
        )?;

        Command::cargo_bin("rjot")?
            .args([
                "new",
                "--template",
                "custom.md",
                "-v",
                "ticket_id=PROJ-456",
                "-v",
                "feature=templating",
            ])
            .env("RJOT_DIR", &rjot_dir)
            .env("EDITOR", "true")
            .assert()
            .success();

        let entries_dir = rjot_dir.join("notebooks").join("default");
        let entry_path = fs::read_dir(entries_dir)?.next().unwrap()?.path();
        let content = fs::read_to_string(entry_path)?;

        assert!(content.contains("Ticket: PROJ-456"));
        assert!(content.contains("Feature: templating"));

        Ok(())
    }
}

// A module for testing the interactive shell.
#[cfg(test)]
mod shell {
    use super::*;
    use std::io::Write;
    // Use the standard library's process::Command for interactive tests.
    use std::process::{Command, Stdio};

    /// Tests the basic lifecycle of the interactive shell.
    #[test]
    fn test_shell_lifecycle_and_commands() -> TestResult {
        let (_temp_dir, rjot_dir) = setup();

        // Use std::process::Command to get a handle to stdin.
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_rjot"));
        cmd.arg("shell")
            .env("RJOT_DIR", &rjot_dir)
            .stdin(Stdio::piped()) // Pipe stdin so we can write to it.
            .stdout(Stdio::piped()); // Pipe stdout so we can read it.

        let mut process = cmd.spawn()?;

        {
            let stdin = process.stdin.as_mut().expect("Failed to open stdin");

            // 1. Create a jot to ensure the list command has output.
            stdin.write_all(b"a note for the shell test\n")?;

            // 2. Run the `list` command and verify its output.
            stdin.write_all(b"list\n")?;

            // 3. Exit the shell.
            stdin.write_all(b"exit\n")?;
        }

        // Wait for the process to exit and capture its output.
        let output = process.wait_with_output()?;

        // Assert that the entire process was successful.
        assert!(output.status.success());

        // Verify the output contains expected strings from the shell lifecycle.
        let stdout = String::from_utf8(output.stdout)?;
        assert!(stdout.contains("██████╗")); // Check for the ASCII logo
                                             // Check that the output from the `list` command is present.
        assert!(stdout.contains("a note for the shell test"));
        assert!(stdout.contains("Exiting rjot shell."));

        Ok(())
    }
}
