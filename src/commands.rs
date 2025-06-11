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

use crate::cli::{InfoArgs, TagAction, TagArgs};
use crate::helpers::{
    self, display_note_list, get_note_path_for_action, get_rjot_dir_root, get_templates_dir,
    parse_note_from_file, Frontmatter,
};

/// Initializes the `rjot` directory, optionally with Git and/or encryption.
pub fn command_init(git: bool, encrypt: bool) -> Result<()> {
    let root_dir = get_rjot_dir_root()?;
    println!("rjot directory is at: {:?}", root_dir);

    if git {
        match Repository::init(&root_dir) {
            Ok(repo) => {
                println!("Initialized a new Git repository in {:?}", root_dir);
                let gitignore_path = root_dir.join(".gitignore");
                if !repo.is_empty()? {
                    println!("Git repository is not empty. Assuming it is already set up.");
                } else if !gitignore_path.exists() {
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
                println!("Git repository already exists in {:?}", root_dir)
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
            println!("Generated new encryption identity at: {:?}", identity_path);
            println!("\nIMPORTANT: Back this file up somewhere safe!");

            let config_path = root_dir.join("config.toml");
            let config_str = format!("recipient = \"{}\"", recipient);
            fs::write(config_path, config_str)?;
            println!("Saved public key to config.toml.");
            println!("\nYour public key (recipient) is: {}", recipient);
        }
    }
    Ok(())
}

/// Permanently decrypts all notes in the journal.
pub fn command_decrypt(entries_dir: &PathBuf, force: bool) -> Result<()> {
    let root_dir = get_rjot_dir_root()?;
    let identity_path = root_dir.join("identity.txt");
    if !identity_path.exists() {
        println!("Journal is not encrypted (no identity.txt found). Nothing to do.");
        return Ok(());
    }

    if !force {
        print!("This will permanently decrypt all notes and remove your identity file. This action cannot be undone. Continue? [y/N] ");
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

    println!("Starting decryption of all notes...");
    for entry in fs::read_dir(entries_dir)?.filter_map(Result::ok) {
        let path = entry.path();
        if path.is_file() {
            let file_bytes = fs::read(&path)?;
            if !file_bytes.starts_with(b"age-encryption.org") {
                println!(
                    "Skipping non-encrypted file: {:?}",
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
                println!("Decrypted {:?}", path.file_name().unwrap());
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

    println!("Committed changes with message: '{}'", commit_message);

    let head = repo.head()?;
    let branch_name = head.shorthand().with_context(|| {
        "Could not get branch name from HEAD. Are you in a detached HEAD state?"
    })?;
    let refspec = format!("refs/heads/{}:refs/heads/{}", branch_name, branch_name);

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

    println!("Pushing to remote 'origin' on branch '{}'...", branch_name);
    remote.push(&[&refspec], Some(&mut push_options))?;

    println!("Sync complete.");

    Ok(())
}

/// Creates a new jot instantly from command-line arguments.
pub fn command_down(entries_dir: &Path, message: &str, tags: Option<Vec<String>>) -> Result<()> {
    let mut content = String::new();
    if let Some(tags) = tags {
        if !tags.is_empty() {
            let frontmatter = Frontmatter { tags };
            let fm_str = serde_yaml::to_string(&frontmatter)?;
            content.push_str("---\n");
            content.push_str(&fm_str);
            content.push_str("---\n\n");
        }
    }
    content.push_str(message);
    println!("Jotting down: \"{}\"", message);
    let now = Local::now();
    let filename = now.format("%Y-%m-%d-%H%M%S.md").to_string();
    let file_path = entries_dir.join(filename);
    helpers::write_note_file(&file_path, &content)?;
    println!("Successfully saved to {:?}", file_path);
    Ok(())
}

/// Creates a new jot by opening the default editor.
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
        println!("Successfully saved to {:?}", file_path);
    }
    Ok(())
}

/// Opens an existing jot in the default editor.
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

/// Lists the most recent jots.
pub fn command_list(entries_dir: &PathBuf, count: Option<usize>) -> Result<()> {
    let num_to_list = count.unwrap_or(10);
    let entries = fs::read_dir(entries_dir)?;
    let mut notes = Vec::new();
    for entry in entries.filter_map(Result::ok) {
        notes.push(parse_note_from_file(&entry.path())?);
    }
    notes.sort_by(|a, b| b.id.cmp(&a.id));
    notes.truncate(num_to_list);
    display_note_list(notes);
    Ok(())
}

/// Performs a full-text search of all jots.
pub fn command_find(entries_dir: &PathBuf, query: &str) -> Result<()> {
    println!("Searching for \"{}\" in your jots...", query);
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
pub fn command_tags_filter(entries_dir: &PathBuf, tags: &[String]) -> Result<()> {
    println!("Filtering by tags: {:?}", tags);
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
pub fn command_by_date_filter(entries_dir: &PathBuf, date: NaiveDate, compile: bool) -> Result<()> {
    let date_prefix = date.format("%Y-%m-%d").to_string();
    println!("Finding jots from {}...", date_prefix);
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
pub fn command_today(entries_dir: &PathBuf, compile: bool) -> Result<()> {
    command_by_date_filter(entries_dir, Local::now().date_naive(), compile)
}

/// Lists jots created yesterday.
pub fn command_yesterday(entries_dir: &PathBuf, compile: bool) -> Result<()> {
    let yesterday = Local::now().date_naive() - chrono::Duration::days(1);
    command_by_date_filter(entries_dir, yesterday, compile)
}

/// Lists jots created in the current week.
pub fn command_by_week(entries_dir: &PathBuf, compile: bool) -> Result<()> {
    let today = Local::now().date_naive();
    let week_start = today - chrono::Duration::days(today.weekday().num_days_from_sunday() as i64);
    println!("Finding jots from this week (starting {})...", week_start);
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
pub fn command_on(entries_dir: &PathBuf, date_spec: &str, compile: bool) -> Result<()> {
    let mut matches = Vec::new();
    if let Some((start_str, end_str)) = date_spec.split_once("..") {
        let start_date = NaiveDate::parse_from_str(start_str, "%Y-%m-%d")?;
        let end_date = NaiveDate::parse_from_str(end_str, "%Y-%m-%d")?;
        println!("Finding jots from {} to {}...", start_date, end_date);
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
pub fn command_show(note_path: PathBuf) -> Result<()> {
    let content = helpers::read_note_file(&note_path)?;
    println!("{}", content);
    Ok(())
}

/// Deletes a specific jot with user confirmation.
pub fn command_delete(note_path: PathBuf, force: bool) -> Result<()> {
    let filename = note_path.file_name().unwrap().to_string_lossy();
    if !force {
        print!("Are you sure you want to delete '{}'? [y/N] ", filename);
        io::stdout().flush()?;
        let mut confirmation = String::new();
        io::stdin().read_line(&mut confirmation)?;
        if confirmation.trim().to_lowercase() != "y" {
            println!("Deletion aborted.");
            return Ok(());
        }
    }
    fs::remove_file(&note_path)?;
    println!("Successfully deleted '{}'.", filename);
    Ok(())
}

/// Displays information and statistics about the journal.
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
        println!("Root Directory:  {:?}", helpers::get_rjot_dir_root()?);
        println!("Entries:         {:?}", entries_dir);
        println!("Templates:       {:?}", helpers::get_templates_dir()?);
    }
    if args.stats {
        println!("\n--- rjot stats ---");
        let entries = fs::read_dir(entries_dir)?;
        let mut note_count = 0;
        let mut tag_counts: HashMap<String, usize> = HashMap::new();
        for entry in entries.filter_map(Result::ok) {
            note_count += 1;
            let note = parse_note_from_file(&entry.path())?;
            for tag in note.frontmatter.tags {
                *tag_counts.entry(tag).or_insert(0) += 1;
            }
        }
        println!("Total jots: {}", note_count);
        if !tag_counts.is_empty() {
            let mut sorted_tags: Vec<_> = tag_counts.into_iter().collect();
            sorted_tags.sort_by(|a, b| b.1.cmp(&a.1));
            sorted_tags.truncate(5);
            println!("\nMost common tags:");
            for (tag, count) in sorted_tags {
                println!("  - {} ({})", tag, count);
            }
        }
    }
    Ok(())
}
