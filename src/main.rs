use std::env;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{bail, Context, Result};
use chrono::{Datelike, Local, NaiveDate};
use clap::{Parser, Subcommand};
use serde::Deserialize;

// --- Data Structures (Unchanged) ---
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

// --- Clap CLI Definition (Updated for new commands) ---
#[derive(Parser, Debug)]
#[command(name = "rjot", version, about = "A minimalist, command-line journal.")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
    /// The message to jot down.
    message: Vec<String>,
    /// Add tags to a new jot.
    #[arg(long, short, value_delimiter = ',')]
    tags: Option<Vec<String>>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Create a new jot using an editor, optionally with a template
    New {
        /// The name of the template to use from the templates directory
        #[arg(long, short)]
        template: Option<String>,
    },
    /// List the last 10 jots
    List,
    /// Find jots by searching their content
    Find {
        #[arg(required = true)]
        query: String,
    },
    /// List jots that have specific tags
    Tags {
        #[arg(required = true)]
        tags: Vec<String>,
    },
    /// List jots from today
    Today {
        #[arg(long, short)]
        compile: bool,
    },
    /// List jots from yesterday
    Yesterday {
        #[arg(long, short)]
        compile: bool,
    },
    /// List jots from this week
    Week {
        #[arg(long, short)]
        compile: bool,
    },
    // NEW: Utility commands
    /// Open an existing jot in the default editor
    Edit {
        /// The prefix of the jot ID to edit (e.g., '2025-06-07-1945')
        #[arg(required = true)]
        id_prefix: String,
    },
    /// Display the full content of a jot in the terminal
    Show {
        /// The prefix of the jot ID to show (e.g., '2025-06-07-1945')
        #[arg(required = true)]
        id_prefix: String,
    },
}

// --- Helper Functions ---

fn get_rjot_dir_root() -> Result<PathBuf> {
    let path = match env::var("RJOT_DIR") {
        Ok(val) => PathBuf::from(val),
        Err(_) => dirs::config_dir()
            .with_context(|| "Could not find a valid config directory.")?
            .join("rjot"),
    };
    if !path.exists() {
        fs::create_dir_all(&path)?;
    }
    Ok(path)
}

fn get_entries_dir() -> Result<PathBuf> {
    let root_dir = get_rjot_dir_root()?;
    let entries_dir = root_dir.join("entries");
    if !entries_dir.exists() {
        fs::create_dir_all(&entries_dir)?;
    }
    Ok(entries_dir)
}

fn get_templates_dir() -> Result<PathBuf> {
    let root_dir = get_rjot_dir_root()?;
    let templates_dir = root_dir.join("templates");
    if !templates_dir.exists() {
        fs::create_dir_all(&templates_dir)?;
    }
    Ok(templates_dir)
}

fn parse_note_from_file(path: &Path) -> Result<Note> {
    let filename = path.file_name().unwrap().to_string_lossy().to_string();
    let id = filename.replace(".md", "");
    let file_content =
        fs::read_to_string(path).with_context(|| format!("Could not read file: {:?}", path))?;
    if file_content.starts_with("---") {
        if let Some(end_frontmatter) = file_content.get(3..).and_then(|s| s.find("---")) {
            let frontmatter_str = &file_content[3..(3 + end_frontmatter)];
            let content = file_content[(3 + end_frontmatter + 3)..].trim().to_string();
            let frontmatter: Frontmatter = serde_yaml::from_str(frontmatter_str)
                .with_context(|| format!("Failed to parse YAML frontmatter in {:?}", path))?;
            return Ok(Note {
                id,
                frontmatter,
                content,
            });
        }
    }
    Ok(Note {
        id,
        frontmatter: Frontmatter::default(),
        content: file_content,
    })
}

fn display_note_list(notes: Vec<Note>) {
    if notes.is_empty() {
        println!("\nNo jots found.");
        return;
    }
    //println!("\n{:<22} {}", "ID", "FIRST LINE OF CONTENT");
    println!("\n{:<22} FIRST LINE OF CONTENT", "ID");
    println!("{:-<22} {:-<50}", "", "");
    for note in notes {
        let first_line = note.content.lines().next().unwrap_or("").trim();
        println!("{:<22} {}", note.id, first_line);
    }
}

fn compile_notes(notes: Vec<Note>) -> Result<()> {
    for note in notes {
        println!("---\n\n# {}\n\n{}", note.id, note.content);
    }
    Ok(())
}

// NEW: Helper to find a unique note by its ID prefix.
fn find_unique_note_by_prefix(entries_dir: &Path, prefix: &str) -> Result<PathBuf> {
    let entries = fs::read_dir(entries_dir)?;
    let mut matches = Vec::new();
    for entry in entries.filter_map(Result::ok) {
        if entry.file_name().to_string_lossy().starts_with(prefix) {
            matches.push(entry.path());
        }
    }

    if matches.is_empty() {
        bail!("No jot found with the prefix '{}'", prefix);
    } else if matches.len() > 1 {
        bail!(
            "Prefix '{}' is not unique. Multiple jots found:\n{}",
            prefix,
            matches
                .iter()
                .map(|p| p.file_name().unwrap().to_string_lossy())
                .collect::<Vec<_>>()
                .join("\n")
        );
    } else {
        Ok(matches.into_iter().next().unwrap())
    }
}

// --- Main Entrypoint ---
fn main() -> Result<()> {
    let cli = Cli::parse();
    let entries_dir = get_entries_dir()?;

    if let Some(command) = cli.command {
        match command {
            Commands::New { template } => command_new(&entries_dir, template)?,
            Commands::List => command_list(&entries_dir)?,
            Commands::Find { query } => command_find(&entries_dir, &query)?,
            Commands::Tags { tags } => command_tags(&entries_dir, &tags)?,
            Commands::Today { compile } => {
                command_by_date_filter(&entries_dir, Local::now().date_naive(), compile)?
            }
            Commands::Yesterday { compile } => {
                let yesterday = Local::now().date_naive() - chrono::Duration::days(1);
                command_by_date_filter(&entries_dir, yesterday, compile)?;
            }
            Commands::Week { compile } => command_by_week(&entries_dir, compile)?,
            // NEW: Handle the new commands
            Commands::Edit { id_prefix } => command_edit(&entries_dir, &id_prefix)?,
            Commands::Show { id_prefix } => command_show(&entries_dir, &id_prefix)?,
        }
    } else if cli.message.is_empty() {
        if cli.tags.is_some() {
            bail!("The --tags argument can only be used when creating a new rjot with a message.");
        }
        println!(
            "No message provided. Use 'rjot \"your message\"' or a subcommand like 'rjot new'."
        );
        println!("\nFor more information, try '--help'");
    } else {
        let message = cli.message.join(" ");
        command_now(&entries_dir, &message, cli.tags)?;
    }

    Ok(())
}

// --- Command Logic ---

fn command_now(entries_dir: &Path, message: &str, tags: Option<Vec<String>>) -> Result<()> {
    // ... (This function is unchanged)
    let mut content = String::new();
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
    let file_path = entries_dir.join(filename);
    fs::write(&file_path, content)?;
    println!("Successfully saved to {:?}", file_path);
    Ok(())
}

fn command_new(entries_dir: &Path, template_name: Option<String>) -> Result<()> {
    // ... (This function is unchanged)
    let editor =
        env::var("EDITOR").with_context(|| "The '$EDITOR' environment variable is not set.")?;
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
    fs::write(&file_path, initial_content)?;
    let status = Command::new(&editor).arg(&file_path).status()?;
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

fn command_list(entries_dir: &PathBuf) -> Result<()> {
    // ... (This function is unchanged)
    let entries = fs::read_dir(entries_dir)?;
    let mut notes = Vec::new();
    for entry in entries.filter_map(Result::ok) {
        notes.push(parse_note_from_file(&entry.path())?);
    }
    notes.sort_by(|a, b| b.id.cmp(&a.id));
    notes.truncate(10);
    display_note_list(notes);
    Ok(())
}

fn command_find(entries_dir: &PathBuf, query: &str) -> Result<()> {
    // ... (This function is unchanged)
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

fn command_tags(entries_dir: &PathBuf, tags: &[String]) -> Result<()> {
    // ... (This function is unchanged)
    println!("Filtering by tags: {:?}", tags);
    let entries = fs::read_dir(entries_dir)?;
    let mut matches = Vec::new();
    for entry in entries.filter_map(Result::ok) {
        let note = parse_note_from_file(&entry.path())?;
        if let Some(note_tags) = &note.frontmatter.tags {
            if tags.iter().all(|t| note_tags.contains(t)) {
                matches.push(note);
            }
        }
    }
    display_note_list(matches);
    Ok(())
}

fn command_by_date_filter(entries_dir: &PathBuf, date: NaiveDate, compile: bool) -> Result<()> {
    // ... (This function is unchanged)
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
        compile_notes(matches)?;
    } else {
        display_note_list(matches);
    }
    Ok(())
}

fn command_by_week(entries_dir: &PathBuf, compile: bool) -> Result<()> {
    // ... (This function is unchanged)
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
        compile_notes(matches)?;
    } else {
        display_note_list(matches);
    }
    Ok(())
}

// NEW: Command logic for editing a specific jot
fn command_edit(entries_dir: &Path, id_prefix: &str) -> Result<()> {
    let note_path = find_unique_note_by_prefix(entries_dir, id_prefix)?;

    let editor =
        env::var("EDITOR").with_context(|| "The '$EDITOR' environment variable is not set.")?;
    println!("Opening {:?} in {}...", &note_path, &editor);

    let status = Command::new(&editor)
        .arg(&note_path)
        .status()
        .with_context(|| format!("Failed to open editor '{}'", editor))?;

    if !status.success() {
        bail!("Editor exited with a non-zero status.");
    }

    println!("Finished editing {:?}.", &note_path.file_name().unwrap());
    Ok(())
}

// NEW: Command logic for showing a specific jot
fn command_show(entries_dir: &Path, id_prefix: &str) -> Result<()> {
    let note_path = find_unique_note_by_prefix(entries_dir, id_prefix)?;
    let content = fs::read_to_string(&note_path)
        .with_context(|| format!("Could not read file: {:?}", note_path))?;

    println!("{}", content);
    Ok(())
}
