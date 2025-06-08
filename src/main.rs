use std::env;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{bail, Context, Result};
use chrono::Local;
use clap::{Parser, Subcommand};
use serde::Deserialize;

// Data structures for a parsed note
#[derive(Debug, Deserialize, Default)]
struct Frontmatter {
    tags: Option<Vec<String>>,
}

#[derive(Debug, Default)]
struct Note {
    id: String,
    frontmatter: Frontmatter,
    content: String,
}

#[derive(Parser, Debug)]
#[command(version, about = "A minimalist, command-line journal.")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// The message to jot down.
    message: Vec<String>,

    /// Add tags to a new jot. Can be used multiple times or with commas.
    #[arg(long, short, value_delimiter = ',')]
    tags: Option<Vec<String>>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Create a new jot using the default editor
    New,
    /// List the last 10 jots
    List,
    /// Find jots by searching their content
    Find {
        /// Text to search for in the content of your jots
        #[arg(required = true)]
        query: String,
    },
    /// List jots that have specific tags
    Tags {
        /// Tags to filter by
        #[arg(required = true)]
        tags: Vec<String>,
    },
}

/// Gets the root directory for all jot data, ensuring it exists.
fn get_jot_dir() -> Result<PathBuf> {
    let path = match env::var("JOT_DIR") {
        Ok(val) => PathBuf::from(val),
        Err(_) => {
            let mut config_dir = dirs::config_dir()
                .with_context(|| "Could not find a valid config directory.")?;
            config_dir.push("jot");
            config_dir
        }
    };

    let entries_dir = path.join("entries");
    if !entries_dir.exists() {
        fs::create_dir_all(&entries_dir)
            .with_context(|| format!("Failed to create jot directory at {:?}", &entries_dir))?;
    }
    Ok(entries_dir)
}

/// Parses a file into a Note struct, separating frontmatter from content.
fn parse_note_from_file(path: &Path) -> Result<Note> {
    let filename = path.file_name().unwrap().to_string_lossy().to_string();
    let id = filename.replace(".md", "");

    let file_content = fs::read_to_string(path)
        .with_context(|| format!("Could not read file: {:?}", path))?;

    if file_content.starts_with("---") {
        if let Some(end_frontmatter) = file_content.get(3..).and_then(|s| s.find("---")) {
            let frontmatter_str = &file_content[3..(3 + end_frontmatter)];
            let content = file_content[(3 + end_frontmatter + 3)..].trim().to_string();

            let frontmatter: Frontmatter = serde_yaml::from_str(frontmatter_str)
                .with_context(|| format!("Failed to parse YAML frontmatter in {:?}", path))?;

            return Ok(Note { id, frontmatter, content });
        }
    }

    // No frontmatter found, treat the whole file as content
    Ok(Note {
        id,
        frontmatter: Frontmatter::default(),
        content: file_content,
    })
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let jot_dir = get_jot_dir()?;

    // Decide which command to run
    if let Some(command) = cli.command {
        match command {
            Commands::New => command_new(&jot_dir)?,
            Commands::List => command_list(&jot_dir)?,
            Commands::Find { query } => command_find(&jot_dir, &query)?,
            Commands::Tags { tags } => command_tags(&jot_dir, &tags)?,
        }
    } else {
        if cli.message.is_empty() {
            // If tags are provided without a message, show an error.
            if cli.tags.is_some() {
                bail!("The --tags argument can only be used when creating a new jot with a message.");
            }
            println!("No message provided. Use 'jot \"your message\"' or a subcommand like 'jot new'.");
            println!("\nFor more information, try '--help'");
        } else {
            let message = cli.message.join(" ");
            command_now(&jot_dir, &message, cli.tags)?;
        }
    }

    Ok(())
}

/// Handles the default action of creating a jot directly from arguments.
fn command_now(jot_dir: &PathBuf, message: &str, tags: Option<Vec<String>>) -> Result<()> {
    let mut content = String::new();

    // If tags are provided, construct a frontmatter string
    if let Some(tags) = tags {
        if !tags.is_empty() {
            content.push_str("---\n");
            content.push_str("tags:\n");
            for tag in tags {
                content.push_str(&format!("  - {}\n", tag));
            }
            content.push_str("---\n\n");
        }
    }

    content.push_str(message);

    println!("Jotting down: \"{}\"", message);
    let now = Local::now();
    let filename = now.format("%Y-%m-%d-%H%M%S.md").to_string();
    let file_path = jot_dir.join(filename);

    fs::write(&file_path, content)
        .with_context(|| format!("Failed to write to file {:?}", file_path))?;

    println!("Successfully saved to {:?}", file_path);
    Ok(())
}

/// Handles the `jot new` subcommand.
fn command_new(jot_dir: &PathBuf) -> Result<()> {
    let editor = env::var("EDITOR")
        .with_context(|| "The '$EDITOR' environment variable is not set.")?;

    let now = Local::now();
    let filename = now.format("%Y-%m-%d-%H%M%S.md").to_string();
    let file_path = jot_dir.join(filename);

    let status = Command::new(&editor)
        .arg(&file_path)
        .status()
        .with_context(|| format!("Failed to open editor '{}'", editor))?;

    if !status.success() {
        bail!("Editor exited with a non-zero status. Aborting.");
    }

    let mut file = fs::File::open(&file_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    if contents.trim().is_empty() {
        fs::remove_file(&file_path)?;
        println!("Empty jot discarded.");
    } else {
        println!("Successfully saved to {:?}", file_path);
    }

    Ok(())
}

/// Handles the `jot list` subcommand.
fn command_list(jot_dir: &PathBuf) -> Result<()> {
    let entries = fs::read_dir(jot_dir)?;
    let mut sorted_entries: Vec<_> = entries.filter_map(Result::ok).collect();
    sorted_entries.sort_by_key(|e| e.file_name());
    sorted_entries.reverse();

    if sorted_entries.is_empty() {
        println!("\nNo jots found. Get jotting!");
        return Ok(());
    }

    println!("\n{:<22} {}", "ID", "FIRST LINE OF CONTENT");
    println!("{:-<22} {:-<50}", "", "");

    for entry in sorted_entries.iter().take(10) {
        let note = parse_note_from_file(&entry.path())?;
        let first_line = note.content.lines().next().unwrap_or("").trim();
        println!("{:<22} {}", note.id, first_line);
    }
    Ok(())
}

/// Handles the `jot find` subcommand.
fn command_find(jot_dir: &PathBuf, query: &str) -> Result<()> {
    println!("Searching for \"{}\" in your jots...", query);
    let entries = fs::read_dir(jot_dir)?;
    let mut matches = Vec::new();

    for entry in entries.filter_map(Result::ok) {
        let note = parse_note_from_file(&entry.path())?;
        if note.content.to_lowercase().contains(&query.to_lowercase()) {
            matches.push(note);
        }
    }

    if matches.is_empty() {
        println!("\nNo matches found.");
        return Ok(());
    }

    println!("\n{:<22} {}", "MATCHING NOTE ID", "FIRST LINE OF CONTENT");
    println!("{:-<22} {:-<50}", "", "");
    for note in matches {
        let first_line = note.content.lines().next().unwrap_or("").trim();
        println!("{:<22} {}", note.id, first_line);
    }
    Ok(())
}

/// Handles the `jot tags` subcommand.
fn command_tags(jot_dir: &PathBuf, tags: &[String]) -> Result<()> {
    println!("Filtering by tags: {:?}", tags);
    let entries = fs::read_dir(jot_dir)?;
    let mut matches = Vec::new();

    for entry in entries.filter_map(Result::ok) {
        let note = parse_note_from_file(&entry.path())?;
        if let Some(note_tags) = &note.frontmatter.tags {
            if tags.iter().all(|t| note_tags.contains(t)) {
                matches.push(note);
            }
        }
    }

    if matches.is_empty() {
        println!("\nNo notes found with all of those tags.");
        return Ok(());
    }

    println!("\n{:<22} {}", "MATCHING NOTE ID", "FIRST LINE OF CONTENT");
    println!("{:-<22} {:-<50}", "", "");
    for note in matches {
        let first_line = note.content.lines().next().unwrap_or("").trim();
        println!("{:<22} {}", note.id, first_line);
    }
    Ok(())
}