use assert_cmd::Command;
use chrono::{Datelike, Local};
use predicates::prelude::*;
use std::env;
use std::fs;
// std::io is not directly used, fs operations return Result from std::io implicitly.
use std::path::{Path, PathBuf};
// std::thread and std::time::Duration are used in some tests.
use std::thread;
use std::time::Duration;
use tempfile::{tempdir, TempDir};

type TestResult = Result<(), Box<dyn std::error::Error>>;

// Helper function to set up a test environment
fn setup_test_environment() -> (TempDir, PathBuf) {
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let rjot_dir = temp_dir.path().to_path_buf();
    // For notebook-aware tests, RJOT_DIR is the root.
    // The 'init' command or direct helper calls will create 'notebooks/default' etc.
    env::set_var("RJOT_DIR", &rjot_dir);
    // Critical: Unset RJOT_ACTIVE_NOTEBOOK to ensure tests start with a clean slate
    env::remove_var("RJOT_ACTIVE_NOTEBOOK");
    (temp_dir, rjot_dir)
}

// Helper to get the path to a specific notebook's entries directory
fn get_notebook_entries_path(rjot_dir: &Path, notebook_name: &str) -> PathBuf {
    rjot_dir.join("notebooks").join(notebook_name)
}

// Helper to get rjot command
fn rjot_cmd() -> Command {
    let mut cmd = Command::cargo_bin("rjot").unwrap();
    // Ensure RJOT_DIR is set from the test environment for every command
    // This is redundant if setup_test_environment is always called AND its env var is inherited,
    // but explicit can be safer for tests.
    if let Ok(rjot_dir_val) = env::var("RJOT_DIR") {
        cmd.env("RJOT_DIR", rjot_dir_val);
    }
    // If RJOT_ACTIVE_NOTEBOOK is set in the test env, pass it through.
    // Otherwise, ensure it's not set for the command (clean state).
    if let Ok(active_notebook_val) = env::var("RJOT_ACTIVE_NOTEBOOK") {
        cmd.env("RJOT_ACTIVE_NOTEBOOK", active_notebook_val);
    } else {
        cmd.env_remove("RJOT_ACTIVE_NOTEBOOK");
    }
    cmd
}

#[test]
fn test_init_command_creates_default_notebook() -> TestResult {
    let (_temp_dir, rjot_dir) = setup_test_environment();

    rjot_cmd().arg("init").assert().success();

    assert!(get_notebook_entries_path(&rjot_dir, "default").exists(), "Default notebook directory should be created by init");
    // assert!(rjot_dir.join("templates").exists()); // init does not create templates dir by default
    Ok(())
}

#[test]
fn test_default_jot_creation_in_default_notebook() -> TestResult {
    let (_temp_dir, rjot_dir) = setup_test_environment();
    rjot_cmd().arg("init").assert().success();
    std::thread::sleep(std::time::Duration::from_millis(100)); // Small delay

    rjot_cmd()
        .arg("a default note")
        .assert()
        .success()
        .stdout(predicate::str::contains("Jotting down:"));

    let entries_dir = get_notebook_entries_path(&rjot_dir, "default");
    assert_eq!(fs::read_dir(entries_dir)?.count(), 1);
    Ok(())
}

#[test]
fn test_misspelled_command_is_a_note_in_default_notebook() -> TestResult {
    let (_temp_dir, rjot_dir) = setup_test_environment();
    rjot_cmd().arg("init").assert().success();

    rjot_cmd()
        .arg("lisy") // Misspelled "list"
        .assert()
        .success()
        .stdout(predicate::str::contains("Jotting down: \"lisy\""));

    let entries_dir = get_notebook_entries_path(&rjot_dir, "default");
    assert_eq!(fs::read_dir(entries_dir)?.count(), 1);
    Ok(())
}

#[test]
fn test_tagged_jot_creation_in_default_notebook() -> TestResult {
    let (_temp_dir, rjot_dir) = setup_test_environment();
    rjot_cmd().arg("init").assert().success();

    rjot_cmd()
        .arg("a tagged note")
        .args(["--tags", "rust,project"])
        .assert()
        .success();

    let entries_dir = get_notebook_entries_path(&rjot_dir, "default");
    let entry_path = fs::read_dir(entries_dir)?.next().unwrap()?.path();
    let content = fs::read_to_string(entry_path)?;

    assert!(content.contains("tags:"));
    assert!(content.contains("- rust"));
    assert!(content.contains("- project"));
    Ok(())
}

#[test]
fn test_list_and_find_in_default_notebook() -> TestResult {
    let (_temp_dir, rjot_dir) = setup_test_environment();
    rjot_cmd().arg("init").assert().success();

    rjot_cmd()
        .arg("note about a unique_keyword_default")
        .assert()
        .success();

    rjot_cmd()
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("unique_keyword_default"));

    rjot_cmd()
        .arg("find")
        .arg("unique_keyword_default")
        .assert()
        .success()
        .stdout(predicate::str::contains("unique_keyword_default"));
    Ok(())
}

#[test]
fn test_notebook_new_list_path_cmds() -> TestResult {
    let (_temp_dir, rjot_dir) = setup_test_environment();
    rjot_cmd().arg("init").assert().success(); // Creates 'default'

    // Create 'work' notebook
    rjot_cmd()
        .args(["notebook", "new", "work"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Successfully created notebook 'work'"));
    assert!(get_notebook_entries_path(&rjot_dir, "work").exists());

    // Create 'personal' notebook using alias
    rjot_cmd().args(["nb", "new", "personal"]).assert().success();
    assert!(get_notebook_entries_path(&rjot_dir, "personal").exists());

    // List notebooks
    rjot_cmd()
        .args(["notebook", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("- default"))
        .stdout(predicate::str::contains("- work"))
        .stdout(predicate::str::contains("- personal"));

    // Get path of 'work' notebook
    let work_path_output = rjot_cmd()
        .args(["notebook", "path", "work"])
        .assert()
        .success();
    let work_path_str = String::from_utf8(work_path_output.get_output().stdout.clone())?;
    assert!(work_path_str.trim().ends_with("notebooks/work"));
    assert!(Path::new(work_path_str.trim()).exists());

    // Get path of current active notebook (should be 'default')
    let default_path_output = rjot_cmd()
        .args(["notebook", "path"]) // No name
        .assert()
        .success();
    let default_path_str = String::from_utf8(default_path_output.get_output().stdout.clone())?;
    assert!(default_path_str.trim().ends_with("notebooks/default"));
    assert!(Path::new(default_path_str.trim()).exists());

    Ok(())
}

#[test]
fn test_notebook_use_cmd_output() -> TestResult {
    let (_temp_dir, _rjot_dir) = setup_test_environment();
    rjot_cmd().arg("init").assert().success();
    rjot_cmd().args(["notebook", "new", "projectx"]).assert().success();

    rjot_cmd()
        .args(["notebook", "use", "projectx"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "export RJOT_ACTIVE_NOTEBOOK=\"projectx\"",
        ))
        .stdout(predicate::str::contains(
            "$env:RJOT_ACTIVE_NOTEBOOK = \"projectx\"",
        ));

    rjot_cmd()
        .args(["notebook", "use", "nonexistent"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Notebook 'nonexistent' not found."));
    Ok(())
}

#[test]
fn test_jot_in_custom_notebook_via_env_var() -> TestResult {
    let (_temp_dir, rjot_dir) = setup_test_environment();
    rjot_cmd().arg("init").assert().success();
    rjot_cmd().args(["notebook", "new", "envbook"]).assert().success();

    rjot_cmd()
        .arg("note in default")
        .assert()
        .success();

    env::set_var("RJOT_ACTIVE_NOTEBOOK", "envbook");

    rjot_cmd().arg("Note for envbook").assert().success();

    let entries_dir = get_notebook_entries_path(&rjot_dir, "envbook");
    assert_eq!(fs::read_dir(entries_dir)?.count(), 1);
    let entry_path = fs::read_dir(get_notebook_entries_path(&rjot_dir, "envbook"))?.next().unwrap()?.path();
    let content = fs::read_to_string(entry_path)?;
    assert!(content.contains("Note for envbook"));

    rjot_cmd()
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("Note for envbook"))
        .stdout(predicate::str::contains("note in default").not());

    env::remove_var("RJOT_ACTIVE_NOTEBOOK");
    Ok(())
}

#[test]
fn test_jot_in_custom_notebook_via_notebook_flag() -> TestResult {
    let (_temp_dir, rjot_dir) = setup_test_environment();
    rjot_cmd().arg("init").assert().success();
    rjot_cmd().args(["notebook", "new", "flagbook"]).assert().success();
    rjot_cmd().arg("A default note first").assert().success();

    rjot_cmd()
        .args(["--notebook-opt", "flagbook", "Note for flagbook"])
        .assert()
        .success();

    let entries_dir_flag = get_notebook_entries_path(&rjot_dir, "flagbook");
    assert_eq!(fs::read_dir(entries_dir_flag)?.count(), 1);

    rjot_cmd()
        .args(["list", "--notebook-opt", "flagbook"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Note for flagbook"))
        .stdout(predicate::str::contains("A default note first").not());

    rjot_cmd()
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("A default note first"))
        .stdout(predicate::str::contains("Note for flagbook").not());
    Ok(())
}

#[test]
fn test_info_paths_reflects_notebook_context() -> TestResult {
    let (_temp_dir, _rjot_dir) = setup_test_environment();
    rjot_cmd().arg("init").assert().success();
    rjot_cmd().args(["notebook", "new", "infobook"]).assert().success();

    let rjot_dir_path = PathBuf::from(env::var("RJOT_DIR").unwrap());

    let default_notebook_path_str = get_notebook_entries_path(&rjot_dir_path, "default").to_string_lossy().into_owned();
    let infobook_path_str = get_notebook_entries_path(&rjot_dir_path, "infobook").to_string_lossy().into_owned();

    rjot_cmd()
        .args(["info", "--paths"])
        .assert()
        .success()
        .stdout(predicate::str::contains(format!("Active Notebook Dir: \"{}\"", default_notebook_path_str)))
        .stdout(predicate::str::contains("Env Var (RJOT_ACTIVE_NOTEBOOK): Not set"));

    rjot_cmd()
        .args(["info", "--paths", "--notebook-opt", "infobook"])
        .assert()
        .success()
        .stdout(predicate::str::contains(format!("Active Notebook Dir: \"{}\"", infobook_path_str)))
        .stdout(predicate::str::contains("Env Var (RJOT_ACTIVE_NOTEBOOK): Not set"));

    env::set_var("RJOT_ACTIVE_NOTEBOOK", "infobook");
    rjot_cmd()
        .args(["info", "--paths"])
        .assert()
        .success()
        .stdout(predicate::str::contains(format!("Active Notebook Dir: \"{}\"", infobook_path_str)))
        .stdout(predicate::str::contains("Env Var (RJOT_ACTIVE_NOTEBOOK): infobook"));

    env::remove_var("RJOT_ACTIVE_NOTEBOOK");
    Ok(())
}

#[test]
fn test_find_command_scoped_to_notebook() -> TestResult {
    let (_temp_dir, rjot_dir) = setup_test_environment();
    rjot_cmd().arg("init").assert().success();
    rjot_cmd().args(["notebook", "new", "findnb"]).assert().success();

    rjot_cmd().arg("CommonText in default").assert().success();
    rjot_cmd().args(["--notebook-opt", "findnb", "UniqueToFindNB CommonTextInFindNB"]).assert().success();

    rjot_cmd()
        .args(["find", "CommonText"])
        .assert()
        .success()
        .stdout(predicate::str::contains("CommonText in default"))
        .stdout(predicate::str::contains("UniqueToFindNB").not());

    rjot_cmd()
        .args(["find", "--notebook-opt", "findnb", "CommonTextInFindNB"])
        .assert()
        .success()
        .stdout(predicate::str::contains("UniqueToFindNB CommonTextInFindNB"));

    env::set_var("RJOT_ACTIVE_NOTEBOOK", "findnb");
     rjot_cmd()
        .args(["find", "UniqueToFindNB"])
        .assert()
        .success()
        .stdout(predicate::str::contains("UniqueToFindNB CommonTextInFindNB"));
    env::remove_var("RJOT_ACTIVE_NOTEBOOK");
    Ok(())
}

#[test]
fn test_fallback_to_default_notebook_if_env_var_points_to_nonexistent() -> TestResult {
    let (_temp_dir, rjot_dir) = setup_test_environment();
    rjot_cmd().arg("init").assert().success();

    rjot_cmd().arg("Note in actual default for fallback test").assert().success();

    env::set_var("RJOT_ACTIVE_NOTEBOOK", "nonexistent_notebook");

    rjot_cmd()
        .arg("list")
        .assert()
        .success()
        .stderr(predicate::str::contains("Warning: Notebook \"nonexistent_notebook\" specified by RJOT_ACTIVE_NOTEBOOK does not exist."))
        .stdout(predicate::str::contains("Note in actual default for fallback test"));

    rjot_cmd()
        .arg("Jot during fallback warning")
        .assert()
        .success();  // Expect success, will show output if it fails (non-zero exit)

    let default_entries = get_notebook_entries_path(&rjot_dir, "default");
    let default_notes = fs::read_dir(default_entries)?
        .filter_map(Result::ok)
        .map(|e| fs::read_to_string(e.path()))
        .filter_map(Result::ok)
        .collect::<Vec<String>>();

    assert_eq!(default_notes.len(), 2);
    assert!(default_notes.iter().any(|s| s.contains("Jot during fallback warning")));

    env::remove_var("RJOT_ACTIVE_NOTEBOOK");
    Ok(())
}


// --- Older tests adapted for notebook structure ---

#[test]
fn test_show_edit_delete_in_default_notebook() -> TestResult {
    let (temp_dir, rjot_dir) = setup_test_environment();
    rjot_cmd().arg("init").assert().success();


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

    let default_notebook_entries = get_notebook_entries_path(&rjot_dir, "default");
    let mut entries: Vec<_> = fs::read_dir(default_notebook_entries)?
        .map(|r| r.unwrap().path())
        .collect();
    entries.sort();
    let first_note_id = entries[0].file_stem().unwrap().to_str().unwrap();

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

    rjot_cmd()
        .arg("edit")
        .arg(first_note_id)
        .env("EDITOR", &script_path)
        .assert()
        .success();

    let first_note_content = fs::read_to_string(&entries[0])?;
    assert!(first_note_content.contains("edited content"));

    rjot_cmd()
        .arg("delete")
        .arg("--last")
        .arg("--force")
        .assert()
        .success();

    assert_eq!(
        fs::read_dir(get_notebook_entries_path(&rjot_dir, "default"))?.count(),
        1,
        "Expected one jot to remain."
    );

    Ok(())
}

#[test]
fn test_tag_management_in_default_notebook() -> TestResult {
    let (_temp_dir, _rjot_dir) = setup_test_environment();
     rjot_cmd().arg("init").assert().success();

    rjot_cmd()
        .arg("note for tags")
        .assert()
        .success();

    rjot_cmd()
        .args(["tag", "add", "--last=1", "rust", "testing"])
        .assert()
        .success();

    rjot_cmd()
        .args(["show", "--last"])
        .assert()
        .success()
        .stdout(predicate::str::contains("- rust"))
        .stdout(predicate::str::contains("- testing"));
    Ok(())
}

#[test]
fn test_time_based_commands_and_compile_in_default_notebook() -> TestResult {
    let (_temp_dir, rjot_dir) = setup_test_environment();
    rjot_cmd().arg("init").assert().success();
    let entries_dir = get_notebook_entries_path(&rjot_dir, "default");

    let today = Local::now().date_naive();
    fs::write(
        entries_dir.join(format!("{}-120000.md", today.format("%Y-%m-%d"))),
        "note for today",
    )?;
    let week_start = today - chrono::Duration::days(today.weekday().num_days_from_sunday() as i64);
    if week_start != today {
        fs::write(
            entries_dir.join(format!("{}-120000.md", week_start.format("%Y-%m-%d"))),
            "note from start of week",
        )?;
    }

    rjot_cmd()
        .arg("today")
        .assert()
        .success()
        .stdout(predicate::str::contains("note for today"));

    rjot_cmd()
        .arg("week")
        .arg("--compile")
        .assert()
        .success()
        .stdout(predicate::str::contains("note for today"))
        .stdout(predicate::str::contains("note from start of week"));
    Ok(())
}

#[test]
fn test_new_with_template_in_default_notebook() -> TestResult {
    let (_temp_dir, rjot_dir) = setup_test_environment();
    rjot_cmd().arg("init").assert().success();
    let templates_dir = rjot_dir.join("templates");
    fs::create_dir_all(&templates_dir)?; // Ensure templates dir exists
    fs::write(templates_dir.join("daily.md"), "tags:\n  - daily")?;

    rjot_cmd()
        .args(["new", "--template", "daily.md"])
        .env("EDITOR", "true")
        .assert()
        .success();

    let entries_dir = get_notebook_entries_path(&rjot_dir, "default");
    let entry_path = fs::read_dir(entries_dir)?.next().unwrap()?.path();
    let content = fs::read_to_string(entry_path)?;
    assert!(content.contains("- daily"));
    Ok(())
}

#[test]
fn test_git_init_and_sync_with_notebook_structure() -> TestResult {
    let (_temp_dir, rjot_dir) = setup_test_environment();

    rjot_cmd()
        .args(["init", "--git"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Initialized a new Git repository"));

    assert!(rjot_dir.join(".git").exists());
    assert!(get_notebook_entries_path(&rjot_dir, "default").exists());

    let remote_dir_temp = tempdir()?;
    git2::Repository::init_bare(remote_dir_temp.path())?;
    let local_repo = git2::Repository::open(&rjot_dir)?;
    local_repo.remote("origin", remote_dir_temp.path().to_str().unwrap())?;

    rjot_cmd()
        .arg("a note to be synced in default notebook")
        .assert()
        .success();

    rjot_cmd().args(["notebook", "new", "gitnb"]).assert().success();
    rjot_cmd()
        .args(["--notebook-opt", "gitnb", "a note in gitnb"])
        .assert()
        .success();

    rjot_cmd()
        .arg("sync")
        .assert()
        .success()
        .stdout(predicate::str::contains("Sync complete."));

    let remote_repo = git2::Repository::open_bare(remote_dir_temp.path())?;
    let head = remote_repo.head()?.peel_to_commit()?;
    assert!(head.message().unwrap().contains("rjot sync"));
    Ok(())
}

#[test]
fn test_encryption_lifecycle_with_default_notebook() -> TestResult {
    let (_temp_dir, rjot_dir) = setup_test_environment();
    rjot_cmd()
        .args(["init", "--encrypt"])
        .assert()
        .success();

    let default_entries_dir = get_notebook_entries_path(&rjot_dir, "default");

    rjot_cmd()
        .arg("this is a secret in default")
        .assert()
        .success();

    let entry_path = fs::read_dir(&default_entries_dir)?.next().unwrap()?.path();
    let raw_content = fs::read(&entry_path)?;
    assert!(raw_content.starts_with(b"age-encryption.org"));

    rjot_cmd()
        .arg("show")
        .arg("--last")
        .assert()
        .success()
        .stdout(predicate::str::contains("this is a secret in default"));

    rjot_cmd()
        .args(["decrypt", "--force"])
        .assert()
        .success();

    let raw_content_after = fs::read_to_string(&entry_path)?;
    assert!(raw_content_after.contains("this is a secret in default"));
    assert!(!rjot_dir.join("identity.txt").exists());
    Ok(())
}

#[test]
fn test_direct_helper_then_cmd() -> TestResult {
    let (_temp_dir, rjot_dir) = setup_test_environment(); // Sets RJOT_DIR env var

    // Directly call the helper that should create the notebook dir
    // This call to get_specific_notebook_dir uses the RJOT_DIR env var set by setup_test_environment
    // because get_rjot_dir_root() inside it will read that env var.
    // We need to ensure the rjot_dir crate's helpers are accessible.
    // For an integration test, we can't call crate::helpers directly.
    // So, this test needs to be adapted or we infer from other tests.
    // Let's skip this direct helper call from test and focus on rjot command behavior.

    // Create notebook using rjot command
    rjot_cmd().args(["notebook", "new", "directtest"]).assert().success();
    let notebook_path_check = get_notebook_entries_path(&rjot_dir, "directtest");
    assert!(notebook_path_check.exists(), "Notebook directory 'directtest' should be created by 'rjot notebook new'");


    // Now try to jot into it using the command
    rjot_cmd()
        .args(["--notebook-opt", "directtest", "Jot via direct test"])
        .assert()
        .success();

    let entries_dir = get_notebook_entries_path(&rjot_dir, "directtest");
    assert_eq!(fs::read_dir(entries_dir)?.count(), 1, "Should be one jot in directtest notebook");
    Ok(())
}
