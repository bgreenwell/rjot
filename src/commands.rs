//! This module contains the logic for executing each subcommand.
//!
//! Each public function corresponds to a command defined in the `cli` module.
//! It uses functions from the `helpers` module to interact with the file system
//! and perform other utility tasks.

use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

use age::{secrecy::ExposeSecret, x25519, Decryptor, Identity};
use anyhow::{anyhow, bail, Context, Result};
use chrono::{Datelike, Local, NaiveDate};
use git2::{Cred, PushOptions, RemoteCallbacks, Repository, Signature};

// Conditionally compile everything related to skim
#[cfg(not(windows))]
use {
    crossbeam_channel::unbounded,
    skim::prelude::*,
    std::{borrow::Cow, sync::Arc},
};

use crate::cli::{InfoArgs, NotebookAction, NotebookArgs, TagAction, TagArgs};
use crate::helpers::{
    self, display_note_list, get_note_path_for_action, get_notebooks_dir, get_rjot_dir_root,
    get_templates_dir, parse_note_from_file, Frontmatter, TaskStats,
};

// --- Notebook Commands ---

/// Handles all notebook-related subcommands.
pub fn command_notebook(args: NotebookArgs) -> Result<()> {
    match args.action {
        NotebookAction::New { name } => command_notebook_new(&name)?,
        NotebookAction::List => command_notebook_list()?,
        NotebookAction::Use { name } => command_notebook_use(&name)?,
        NotebookAction::Status => command_notebook_status()?,
    }
    Ok(())
}

/// Creates a new notebook directory.
fn command_notebook_new(name: &str) -> Result<()> {
    // Basic sanitization to prevent directory traversal or invalid names.
    if name.contains('/') || name.contains('\\') || name == "." || name == ".." {
        bail!(
            "Invalid notebook name: '{}'. Names cannot contain slashes or be dots.",
            name
        );
    }

    let notebooks_dir = get_notebooks_dir()?;
    let new_notebook_path = notebooks_dir.join(name);

    if new_notebook_path.exists() {
        println!("Notebook '{name}' already exists.");
    } else {
        fs::create_dir_all(&new_notebook_path)?;
        println!("Successfully created new notebook: '{name}'.");
    }
    Ok(())
}

/// Lists all available notebooks.
fn command_notebook_list() -> Result<()> {
    let notebooks_dir = get_notebooks_dir()?;
    let active_notebook =
        env::var("RJOT_ACTIVE_NOTEBOOK").unwrap_or_else(|_| "default".to_string());

    println!("Available notebooks (* indicates active):");

    for entry in fs::read_dir(notebooks_dir)?.filter_map(Result::ok) {
        if entry.path().is_dir() {
            let notebook_name = entry.file_name().to_string_lossy().to_string();
            let prefix = if notebook_name == active_notebook {
                "*"
            } else {
                " "
            };
            println!("  {prefix} {notebook_name}");
        }
    }
    Ok(())
}

/// Prints the shell command to switch the active notebook.
fn command_notebook_use(name: &str) -> Result<()> {
    let notebooks_dir = get_notebooks_dir()?;
    let target_notebook = notebooks_dir.join(name);

    if !target_notebook.exists() || !target_notebook.is_dir() {
        bail!(
            "Notebook '{}' not found. Create it with `rjot notebook new {}`.",
            name,
            name
        );
    }

    // This command prints the shell command for the user to evaluate.
    // It cannot modify the parent shell's environment directly.
    println!("export RJOT_ACTIVE_NOTEBOOK=\"{name}\"");
    Ok(())
}

/// Shows the currently active notebook.
fn command_notebook_status() -> Result<()> {
    let active_notebook =
        env::var("RJOT_ACTIVE_NOTEBOOK").unwrap_or_else(|_| "default".to_string());
    println!("Active notebook: {active_notebook}");
    Ok(())
}

// --- Other Commands (Modified where necessary) ---

/// Initializes the `rjot` directory, optionally with Git and/or encryption.
pub fn command_init(git: bool, encrypt: bool) -> Result<()> {
    let root_dir = get_rjot_dir_root()?;
    println!("rjot directory is at: {root_dir:?}");

    if git {
        match Repository::init(&root_dir) {
            Ok(repo) => {
                println!("Initialized a new Git repository in {root_dir:?}");
                let gitignore_path = root_dir.join(".gitignore");
                if !repo.is_empty()? {
                    println!("Git repository is not empty. Assuming it is already set up.");
                } else if !gitignore_path.exists() {
                    // Correctly ignore only sensitive files. Notebooks should be tracked.
                    fs::write(&gitignore_path, "identity.txt\nconfig.toml\n")?;
                    println!("Created .gitignore to exclude sensitive files.");

                    let mut index = repo.index()?;
                    index.add_path(Path::new(".gitignore"))?;
                    index.write()?;
                    let oid = index.write_tree()?;
                    let tree = repo.find_tree(oid)?;
                    let signature = Signature::now("rjot", "rjot@localhost")?;
                    repo.commit(
                        Some("HEAD"),
                        &signature,
                        &signature,
                        "Initial commit: Add .gitignore",
                        &tree,
                        &[],
                    )?;
                    println!("Created initial commit to track .gitignore");
                }
            }
            Err(e) if e.code() == git2::ErrorCode::Exists => {
                println!("Git repository already exists in {root_dir:?}")
            }
            Err(e) => bail!("Failed to initialize Git repository: {}", e),
        }
    }

    if encrypt {
        let identity = x25519::Identity::generate();
        let recipient = identity.to_public();
        let identity_path = root_dir.join("identity.txt");

        if identity_path.exists() {
            println!("Encryption identity already exists. Doing nothing.");
        } else {
            fs::write(&identity_path, identity.to_string().expose_secret())?;
            println!("Generated new encryption identity at: {identity_path:?}");
            println!("\nIMPORTANT: Back this file up somewhere safe!");

            let config_path = root_dir.join("config.toml");
            let config_str = format!("recipient = \"{recipient}\"");
            fs::write(config_path, config_str)?;
            println!("Saved public key to config.toml.");
            println!("\nYour public key (recipient) is: {recipient}");
        }
    }
    Ok(())
}

/// Permanently decrypts all notes in ALL notebooks. It no longer takes
/// an `entries_dir` argument as it operates globally.
pub fn command_decrypt(force: bool) -> Result<()> {
    let root_dir = get_rjot_dir_root()?;
    let notebooks_dir = get_notebooks_dir()?;
    let identity_path = root_dir.join("identity.txt");

    if !identity_path.exists() {
        println!("Journal is not encrypted (no identity.txt found). Nothing to do.");
        return Ok(());
    }

    if !force {
        print!("This will permanently decrypt all notes in ALL notebooks and remove your identity file. This action cannot be undone. Continue? [y/N] ");
        io::stdout().flush()?;
        let mut confirmation = String::new();
        io::stdin().read_line(&mut confirmation)?;
        if confirmation.trim().to_lowercase() != "y" {
            println!("Decryption aborted.");
            return Ok(());
        }
    }

    println!("Loading decryption key...");
    let identity_str = fs::read_to_string(&identity_path)?;
    let identity = identity_str
        .parse::<x25519::Identity>()
        .map_err(|e| anyhow!(e))?;
    let identities: Vec<Box<dyn Identity>> = vec![Box::new(identity)];

    println!("Starting decryption of all notes in all notebooks...");
    for notebook_entry in fs::read_dir(notebooks_dir)?.filter_map(Result::ok) {
        if notebook_entry.path().is_dir() {
            let entries_dir = notebook_entry.path();
            println!(
                "\nDecrypting notebook: {:?}",
                entries_dir.file_name().unwrap()
            );
            for entry in fs::read_dir(entries_dir)?.filter_map(Result::ok) {
                let path = entry.path();
                if path.is_file() {
                    let file_bytes = fs::read(&path)?;
                    if !file_bytes.starts_with(b"age-encryption.org") {
                        println!(
                            "  Skipping non-encrypted file: {:?}",
                            path.file_name().unwrap()
                        );
                        continue;
                    }

                    let decryptor = Decryptor::new(&file_bytes as &[u8])?;
                    if let Decryptor::Recipients(reader) = decryptor {
                        let mut decrypted_bytes = vec![];
                        reader
                            .decrypt(identities.iter().map(|i| i.as_ref()))?
                            .read_to_end(&mut decrypted_bytes)?;
                        fs::write(&path, decrypted_bytes)?;
                        println!("  - Decrypted {:?}", path.file_name().unwrap());
                    }
                }
            }
        }
    }

    let config_path = root_dir.join("config.toml");
    fs::remove_file(&identity_path)?;
    if config_path.exists() {
        fs::remove_file(config_path)?;
    }
    println!("\nSuccessfully decrypted journal and removed encryption keys.");
    Ok(())
}

/// Commits and pushes all changes in the rjot Git repository to the `origin` remote.
/// This command requires no changes, as it operates on the root git repo which now
/// contains the `notebooks` directory.
pub fn command_sync() -> Result<()> {
    let root_dir = get_rjot_dir_root()?;
    let repo = Repository::open(&root_dir).map_err(|_| {
        anyhow!(
            "rjot directory at {:?} is not a Git repository. Run `rjot init --git` first.",
            root_dir
        )
    })?;

    println!("Staging all changes...");
    let mut index = repo.index()?;
    index.add_all(["."].iter(), git2::IndexAddOption::DEFAULT, None)?;
    index.write()?;

    let oid = index.write_tree()?;
    let tree = repo.find_tree(oid)?;

    let signature = Signature::now("rjot", "rjot@localhost")?;
    let commit_message = format!("rjot sync: {}", Local::now().to_rfc2822());

    let head = repo.head();
    let parent_commits = match head {
        Ok(head_ref) => vec![head_ref.peel_to_commit()?],
        Err(ref e) if e.code() == git2::ErrorCode::UnbornBranch => Vec::new(),
        Err(e) => return Err(e.into()),
    };

    let parents_ref: Vec<&git2::Commit> = parent_commits.iter().collect();

    repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        &commit_message,
        &tree,
        &parents_ref,
    )?;

    println!("Committed changes with message: '{commit_message}'");

    let head = repo.head()?;
    let branch_name = head.shorthand().with_context(|| {
        "Could not get branch name from HEAD. Are you in a detached HEAD state?"
    })?;
    let refspec = format!("refs/heads/{branch_name}:refs/heads/{branch_name}");

    let mut remote = repo.find_remote("origin").map_err(|_| {
        anyhow!("Could not find remote 'origin'. Please add a remote to your git repository.")
    })?;

    let mut callbacks = RemoteCallbacks::new();
    callbacks.credentials(|_url, username_from_git, allowed_types| {
        let username = username_from_git.unwrap_or("git");

        if allowed_types.is_user_pass_plaintext() {
            if let Ok(token) = env::var("GITHUB_TOKEN") {
                return Cred::userpass_plaintext(username, &token);
            }
        }

        if allowed_types.is_ssh_key() {
            if let Ok(cred) = Cred::ssh_key_from_agent(username) {
                return Ok(cred);
            }
        }

        if allowed_types.is_ssh_key() {
            if let Some(home_dir) = dirs::home_dir() {
                if let Ok(cred) =
                    Cred::ssh_key(username, None, &home_dir.join(".ssh").join("id_rsa"), None)
                {
                    return Ok(cred);
                }
            }
        }

        if allowed_types.is_user_pass_plaintext() {
            if let Ok(cred) = Cred::credential_helper(&repo.config()?, _url, Some(username)) {
                return Ok(cred);
            }
        }

        Err(git2::Error::new(
            git2::ErrorCode::Auth,
            git2::ErrorClass::Ssh,
            "failed to acquire credentials",
        ))
    });

    let mut push_options = PushOptions::new();
    push_options.remote_callbacks(callbacks);

    println!("Pushing to remote 'origin' on branch '{branch_name}'...");
    remote.push(&[&refspec], Some(&mut push_options))?;

    println!("Sync complete.");

    Ok(())
}

/// Creates a new jot instantly from command-line arguments.
/// No changes needed, as it receives the correct `entries_dir`.
pub fn command_down(entries_dir: &Path, message: &str, tags: Option<Vec<String>>) -> Result<()> {
    let mut content = String::new();
    if let Some(tags) = tags {
        if !tags.is_empty() {
            // `..Default::default()` handles the `pinned` field, setting it to false.
            let frontmatter = Frontmatter {
                tags,
                ..Default::default()
            };
            let fm_str = serde_yaml::to_string(&frontmatter)?;
            content.push_str("---\n");
            content.push_str(&fm_str);
            content.push_str("---\n\n");
        }
    }
    content.push_str(message);
    println!("Jotting down: \"{message}\"");
    let now = Local::now();
    let filename = now.format("%Y-%m-%d-%H%M%S.md").to_string();
    let file_path = entries_dir.join(filename);
    helpers::write_note_file(&file_path, &content)?;
    println!("Successfully saved to {file_path:?}");
    Ok(())
}

/// Creates a new jot formatted as a Markdown task.
pub fn command_task(entries_dir: &Path, message: &str) -> Result<()> {
    let task_content = format!("- [ ] {}", message);
    println!("Jotting down task: \"{}\"", message);
    let now = Local::now();
    let filename = now.format("%Y-%m-%d-%H%M%S.md").to_string();
    let file_path = entries_dir.join(filename);
    helpers::write_note_file(&file_path, &task_content)?;
    println!("Successfully saved to {:?}", file_path);
    Ok(())
}

/// Creates a new jot by opening the default editor.
/// No changes needed.
pub fn command_new(entries_dir: &Path, template_name: Option<String>) -> Result<()> {
    let editor = helpers::get_editor()?;
    let now = Local::now();
    let filename = now.format("%Y-%m-%d-%H%M%S.md").to_string();
    let file_path = entries_dir.join(filename);
    let tpl_name = template_name.unwrap_or_else(|| "default.md".to_string());
    let templates_dir = get_templates_dir()?;
    let tpl_path = templates_dir.join(tpl_name);
    let mut initial_content = String::new();
    if tpl_path.exists() {
        let now_str = now.to_rfc3339();
        initial_content = fs::read_to_string(tpl_path)?.replace("{{date}}", &now_str);
    }
    helpers::write_note_file(&file_path, &initial_content)?;
    let status = Command::new(&editor).arg(&file_path).status()?;
    if !status.success() {
        bail!("Editor exited with a non-zero status.");
    }
    let final_content = helpers::read_note_file(&file_path)?;
    if final_content.trim().is_empty() {
        fs::remove_file(&file_path)?;
        println!("Empty jot discarded.");
    } else {
        println!("Successfully saved to {file_path:?}");
    }
    Ok(())
}

/// Opens an existing jot in the default editor.
/// No changes needed.
pub fn command_edit(note_path: PathBuf) -> Result<()> {
    let editor = helpers::get_editor()?;
    println!(
        "Opening {:?} in {}...",
        &note_path.file_name().unwrap(),
        &editor
    );
    let status = Command::new(&editor).arg(&note_path).status()?;
    if !status.success() {
        bail!("Editor exited with a non-zero status.");
    }
    println!("Finished editing {:?}.", &note_path.file_name().unwrap());
    Ok(())
}

/// Manages tags on an existing jot.
/// No changes needed.
pub fn command_tag(entries_dir: &Path, args: TagArgs) -> Result<()> {
    let (id_prefix, last) = match &args.action {
        TagAction::Add {
            id_prefix, last, ..
        }
        | TagAction::Remove {
            id_prefix, last, ..
        }
        | TagAction::Set {
            id_prefix, last, ..
        } => (id_prefix.as_ref(), *last),
    };
    let note_path = get_note_path_for_action(entries_dir, id_prefix.cloned(), last)?;
    let mut note = parse_note_from_file(&note_path)?;
    match args.action {
        TagAction::Add { tags, .. } => {
            for tag in tags {
                if !note.frontmatter.tags.contains(&tag) {
                    note.frontmatter.tags.push(tag);
                }
            }
            println!("Added tags to '{}'.", note.id);
        }
        TagAction::Remove { tags, .. } => {
            note.frontmatter.tags.retain(|t| !tags.contains(t));
            println!("Removed tags from '{}'.", note.id);
        }
        TagAction::Set { tags, .. } => {
            note.frontmatter.tags = tags;
            println!("Set tags for '{}'.", note.id);
        }
    }
    note.frontmatter.tags.sort();
    note.frontmatter.tags.dedup();
    let new_frontmatter_str = serde_yaml::to_string(&note.frontmatter)?;
    let new_content = format!("---\n{}---\n\n{}", new_frontmatter_str, note.content);
    helpers::write_note_file(&note.path, &new_content)?;
    Ok(())
}

// A private helper function to toggle the pinned status of a note.
// This avoids duplicating the logic for finding, parsing, modifying, and saving a note.
fn command_toggle_pin_status(
    entries_dir: &Path,
    id_prefix: Option<String>,
    last: Option<usize>,
    pin: bool,
) -> Result<()> {
    let note_path = get_note_path_for_action(entries_dir, id_prefix, last)?;
    let mut note = parse_note_from_file(&note_path)?;

    // Avoid unnecessary writes if the state is already correct.
    if note.frontmatter.pinned == pin {
        println!(
            "Jot '{}' is already {}.",
            note.id,
            if pin { "pinned" } else { "unpinned" }
        );
        return Ok(());
    }

    note.frontmatter.pinned = pin;

    // Reconstruct the file content with the updated frontmatter.
    let new_frontmatter_str = serde_yaml::to_string(&note.frontmatter)?;
    let new_content = format!("---\n{}---\n\n{}", new_frontmatter_str, note.content);
    helpers::write_note_file(&note.path, &new_content)?;

    println!(
        "Successfully {} jot '{}'.",
        if pin { "pinned" } else { "unpinned" },
        note.id
    );

    Ok(())
}

// Public command function to pin a note.
pub fn command_pin(
    entries_dir: &Path,
    id_prefix: Option<String>,
    last: Option<usize>,
) -> Result<()> {
    command_toggle_pin_status(entries_dir, id_prefix, last, true)
}

// Public command function to unpin a note.
pub fn command_unpin(
    entries_dir: &Path,
    id_prefix: Option<String>,
    last: Option<usize>,
) -> Result<()> {
    command_toggle_pin_status(entries_dir, id_prefix, last, false)
}

/// Lists the most recent jots.
pub fn command_list(
    entries_dir: &PathBuf,
    count: Option<usize>,
    pinned: bool,
    tasks: bool,
) -> Result<()> {
    let num_to_list = count.unwrap_or(10);
    let entries = fs::read_dir(entries_dir)?;
    let mut notes = Vec::new();

    for entry in entries.filter_map(Result::ok) {
        // We parse every note first
        notes.push(parse_note_from_file(&entry.path())?);
    }

    // Filter for pinned notes if the flag is provided.
    if pinned {
        notes.retain(|note| note.frontmatter.pinned);
        println!("Showing pinned jots:");
    }

    if tasks {
        notes.retain(|note| note.tasks.iter().any(|t| !t.completed));
        println!("Showing jots with incomplete tasks:");
    }

    // Sort by date (most recent first) and then truncate to the desired count.
    notes.sort_by(|a, b| b.id.cmp(&a.id));
    notes.truncate(num_to_list);

    display_note_list(notes);
    Ok(())
}

/// Interactively selects a jot using a fuzzy finder.
/// No changes needed.
#[cfg(not(windows))]
pub fn command_select(entries_dir: &PathBuf) -> Result<()> {
    // This struct and its implementation are now inside the conditional block
    struct NoteItem {
        id: String,
        display_text: String,
    }

    impl SkimItem for NoteItem {
        fn text(&self) -> Cow<str> {
            Cow::Borrowed(&self.display_text)
        }

        fn output(&self) -> Cow<str> {
            Cow::Borrowed(&self.id)
        }
    }

    let entries = fs::read_dir(entries_dir)?;
    let mut notes = vec![];
    for entry in entries.filter_map(Result::ok) {
        notes.push(parse_note_from_file(&entry.path())?);
    }
    notes.sort_by(|a, b| b.id.cmp(&a.id));

    let options = SkimOptionsBuilder::default()
        // The .height() option is removed to enable the alternate screen
        .multi(false)
        .reverse(true)
        .build()?;

    // Create a type alias to simplify the complex channel type
    type SkimChannel = (Sender<Arc<dyn SkimItem>>, Receiver<Arc<dyn SkimItem>>);

    let (tx, rx): SkimChannel = unbounded();

    for note in notes {
        let display_text = format!(
            "{} | {}",
            note.id,
            note.content.lines().next().unwrap_or("").trim()
        );
        let item = NoteItem {
            id: note.id,
            display_text,
        };
        let _ = tx.send(Arc::new(item));
    }
    drop(tx);

    let skim_output = Skim::run_with(&options, Some(rx));

    if let Some(output) = skim_output {
        if output.is_abort {
            return Ok(());
        }
    }

    Ok(())
}

/// Performs a full-text search of all jots.
/// No changes needed.
pub fn command_find(entries_dir: &PathBuf, query: &str) -> Result<()> {
    println!("Searching for \"{query}\" in your jots...");
    let entries = fs::read_dir(entries_dir)?;
    let mut matches = Vec::new();
    for entry in entries.filter_map(Result::ok) {
        let note = parse_note_from_file(&entry.path())?;
        if note.content.to_lowercase().contains(&query.to_lowercase()) {
            matches.push(note);
        }
    }
    display_note_list(matches);
    Ok(())
}

/// Filters jots by one or more tags.
/// No changes needed.
pub fn command_tags_filter(entries_dir: &PathBuf, tags: &[String]) -> Result<()> {
    println!("Filtering by tags: {tags:?}");
    let entries = fs::read_dir(entries_dir)?;
    let mut matches = Vec::new();
    for entry in entries.filter_map(Result::ok) {
        let note = parse_note_from_file(&entry.path())?;
        if note.frontmatter.tags.iter().any(|t| tags.contains(t)) {
            matches.push(note);
        }
    }
    display_note_list(matches);
    Ok(())
}

/// A helper function for all date-based filtering.
/// No changes needed.
pub fn command_by_date_filter(entries_dir: &PathBuf, date: NaiveDate, compile: bool) -> Result<()> {
    let date_prefix = date.format("%Y-%m-%d").to_string();
    println!("Finding jots from {date_prefix}...");
    let mut matches = Vec::new();
    for entry in fs::read_dir(entries_dir)?.filter_map(Result::ok) {
        if entry
            .file_name()
            .to_string_lossy()
            .starts_with(&date_prefix)
        {
            matches.push(parse_note_from_file(&entry.path())?);
        }
    }
    matches.sort_by(|a, b| a.id.cmp(&b.id));
    if compile {
        helpers::compile_notes(matches)?
    } else {
        display_note_list(matches)
    }
    Ok(())
}

/// Lists jots created today.
/// No changes needed.
pub fn command_today(entries_dir: &PathBuf, compile: bool) -> Result<()> {
    command_by_date_filter(entries_dir, Local::now().date_naive(), compile)
}

/// Lists jots created yesterday.
/// No changes needed.
pub fn command_yesterday(entries_dir: &PathBuf, compile: bool) -> Result<()> {
    let yesterday = Local::now().date_naive() - chrono::Duration::days(1);
    command_by_date_filter(entries_dir, yesterday, compile)
}

/// Lists jots created in the current week.
/// No changes needed.
pub fn command_by_week(entries_dir: &PathBuf, compile: bool) -> Result<()> {
    let today = Local::now().date_naive();
    let week_start = today - chrono::Duration::days(today.weekday().num_days_from_sunday() as i64);
    println!("Finding jots from this week (starting {week_start})...");
    let mut matches = Vec::new();
    for entry in fs::read_dir(entries_dir)?.filter_map(Result::ok) {
        let filename = entry.file_name().to_string_lossy().to_string();
        if let Ok(date) = NaiveDate::parse_from_str(&filename[0..10], "%Y-%m-%d") {
            if date >= week_start && date <= today {
                matches.push(parse_note_from_file(&entry.path())?);
            }
        }
    }
    matches.sort_by(|a, b| a.id.cmp(&b.id));
    if compile {
        helpers::compile_notes(matches)?
    } else {
        display_note_list(matches)
    }
    Ok(())
}

/// Lists jots from a specific date or date range.
/// No changes needed.
pub fn command_on(entries_dir: &PathBuf, date_spec: &str, compile: bool) -> Result<()> {
    let mut matches = Vec::new();
    if let Some((start_str, end_str)) = date_spec.split_once("..") {
        let start_date = NaiveDate::parse_from_str(start_str, "%Y-%m-%d")?;
        let end_date = NaiveDate::parse_from_str(end_str, "%Y-%m-%d")?;
        println!("Finding jots from {start_date} to {end_date}...");
        for entry in fs::read_dir(entries_dir)?.filter_map(Result::ok) {
            let filename = entry.file_name().to_string_lossy().to_string();
            if let Ok(date) = NaiveDate::parse_from_str(&filename[0..10], "%Y-%m-%d") {
                if date >= start_date && date <= end_date {
                    matches.push(parse_note_from_file(&entry.path())?);
                }
            }
        }
    } else {
        let date = NaiveDate::parse_from_str(date_spec, "%Y-%m-%d")?;
        return command_by_date_filter(entries_dir, date, compile);
    }
    matches.sort_by(|a, b| a.id.cmp(&b.id));
    if compile {
        helpers::compile_notes(matches)?
    } else {
        display_note_list(matches)
    }
    Ok(())
}

/// Displays the full content of a specific jot.
/// No changes needed.
pub fn command_show(note_path: PathBuf) -> Result<()> {
    let content = helpers::read_note_file(&note_path)?;
    println!("{content}");
    Ok(())
}

/// Deletes a specific jot with user confirmation.
/// No changes needed.
pub fn command_delete(note_path: PathBuf, force: bool) -> Result<()> {
    let filename = note_path.file_name().unwrap().to_string_lossy();
    if !force {
        print!("Are you sure you want to delete '{filename}'? [y/N] ");
        io::stdout().flush()?;
        let mut confirmation = String::new();
        io::stdin().read_line(&mut confirmation)?;
        if confirmation.trim().to_lowercase() != "y" {
            println!("Deletion aborted.");
            return Ok(());
        }
    }
    fs::remove_file(&note_path)?;
    println!("Successfully deleted '{filename}'.");
    Ok(())
}

/// Displays information and statistics about the journal.
/// This command is notebook-aware.
pub fn command_info(entries_dir: &PathBuf, args: InfoArgs) -> Result<()> {
    if !args.paths && !args.stats {
        println!(
            "Please provide a flag to the info command, e.g., `rjot info --paths` or `rjot info --stats`"
        );
        println!("\nFor more information, try '--help'");
        return Ok(());
    }
    if args.paths {
        println!("--- rjot paths ---");
        let active_notebook =
            env::var("RJOT_ACTIVE_NOTEBOOK").unwrap_or_else(|_| "default".to_string());
        println!("Root Directory:   {:?}", helpers::get_rjot_dir_root()?);
        println!("Notebooks Root:   {:?}", helpers::get_notebooks_dir()?);
        println!("Active Notebook:  {active_notebook}");
        println!("Entries:          {entries_dir:?}");
        println!("Templates:        {:?}", helpers::get_templates_dir()?);
    }
    if args.stats {
        println!("\n--- rjot stats ---");

        if args.all {
            // Stats for all notebooks
            let notebooks_dir = get_notebooks_dir()?;
            let mut total_notes = 0;
            let mut all_tags: HashMap<String, usize> = HashMap::new();
            let mut total_task_stats = TaskStats::default();

            for entry in fs::read_dir(notebooks_dir)?.filter_map(Result::ok) {
                if entry.path().is_dir() {
                    let notebook_path = entry.path();
                    let (note_count, tag_counts, task_stats) =
                        calculate_stats_for_dir(&notebook_path)?;
                    total_notes += note_count;
                    for (tag, count) in tag_counts {
                        *all_tags.entry(tag).or_insert(0) += count;
                    }
                    total_task_stats.completed += task_stats.completed;
                    total_task_stats.pending += task_stats.pending;
                }
            }
            println!("Stats for all notebooks combined:");
            print_stats(total_notes, all_tags, total_task_stats);
        } else {
            // Stats for the active notebook only
            let active_notebook_name = entries_dir
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("default");
            println!("Stats for active notebook: '{active_notebook_name}'");
            let (note_count, tag_counts, task_stats) = calculate_stats_for_dir(entries_dir)?;
            print_stats(note_count, tag_counts, task_stats);
        }
    }
    Ok(())
}

/// Helper function to calculate stats for a given directory.
fn calculate_stats_for_dir(dir: &Path) -> Result<(usize, HashMap<String, usize>, TaskStats)> {
    let mut note_count = 0;
    let mut tag_counts: HashMap<String, usize> = HashMap::new();
    let mut task_stats = TaskStats::default();

    for entry in fs::read_dir(dir)?.filter_map(Result::ok) {
        if entry.path().is_file() {
            note_count += 1;
            let note = parse_note_from_file(&entry.path())?;
            for tag in note.frontmatter.tags {
                *tag_counts.entry(tag).or_insert(0) += 1;
            }
            for task in note.tasks {
                if task.completed {
                    task_stats.completed += 1;
                } else {
                    task_stats.pending += 1;
                }
            }
        }
    }
    Ok((note_count, tag_counts, task_stats))
}

/// Helper function to print formatted stats.
fn print_stats(note_count: usize, tag_counts: HashMap<String, usize>, task_stats: TaskStats) {
    println!("Total jots: {note_count}");
    if !tag_counts.is_empty() {
        let mut sorted_tags: Vec<_> = tag_counts.into_iter().collect();
        sorted_tags.sort_by(|a, b| b.1.cmp(&a.1));
        sorted_tags.truncate(5);
        println!("\nMost common tags:");
        for (tag, count) in sorted_tags {
            println!("  - {tag} ({count})");
        }
    }
    if task_stats.completed > 0 || task_stats.pending > 0 {
        println!("\nTask Summary:");
        println!("  - Completed: {}", task_stats.completed);
        println!("  - Pending:   {}", task_stats.pending);
    }
}
