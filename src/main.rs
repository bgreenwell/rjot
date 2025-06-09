use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{bail, Context, Result};
use chrono::{Datelike, Local, NaiveDate};
use clap::{Args, Parser, Subcommand};
use serde::Deserialize;

// Data structures
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

// Clap CLI definition
#[derive(Parser, Debug)]
#[command(name = "rjot", version, about = "A minimalist, command-line journal.")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// The message to jot down. The default action if no subcommand is given.
    message: Vec<String>,

    /// Add tags to a new jot. Accepts multiple values, space or comma separated.
    #[arg(long, short, value_delimiter = ',', num_args(1..))]
    tags: Option<Vec<String>>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Create a new jot using an editor, optionally with a template
    New {
        /// The name of the template to use
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
        /// Tags to filter by (can be comma-separated or space-separated)
        #[arg(required = true, value_delimiter = ',')]
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
    /// List jots from a specific date or date range
    On {
        /// The date (YYYY-MM-DD) or range (YYYY-MM-DD..YYYY-MM-DD)
        #[arg(required = true)]
        date_spec: String,
        #[arg(long, short)]
        compile: bool,
    },
    /// Open an existing jot in the default editor
    Edit {
        /// The prefix of the jot ID to edit.
        #[arg(group = "target", required = true)]
        id_prefix: Option<String>,
        /// Edit the Nth most recent jot (e.g., --last=1 or just --last).
        #[arg(long, short, group = "target", num_args(0..=1), default_missing_value = "1")]
        last: Option<usize>,
    },
    /// Display the full content of a jot in the terminal
    Show {
        /// The prefix of the jot ID to show.
        #[arg(group = "target", required = true)]
        id_prefix: Option<String>,
        /// Show the Nth most recent jot.
        #[arg(long, short, group = "target", num_args(0..=1), default_missing_value = "1")]
        last: Option<usize>,
    },
    /// Delete a jot with confirmation
    #[command(alias = "rm")]
    Delete {
        /// The prefix of the jot ID to delete.
        #[arg(group = "target", required = true)]
        id_prefix: Option<String>,
        /// Delete the Nth most recent jot.
        #[arg(long, short, group = "target", num_args(0..=1), default_missing_value = "1")]
        last: Option<usize>,
        /// Force deletion without confirmation
        #[arg(long, short)]
        force: bool,
    },
    /// Display information about your rjot setup
    Info(InfoArgs),
}

#[derive(Args, Debug)]
struct InfoArgs {
    #[arg(long)]
    paths: bool,
    #[arg(long)]
    stats: bool,
}

// Helper functions
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

fn get_ordinal_suffix(n: usize) -> &'static str {
    if (11..=13).contains(&(n % 100)) {
        "th"
    } else {
        match n % 10 {
            // What about 11, 12, and 13?
            1 => "st",
            2 => "nd",
            3 => "rd",
            _ => "th",
        }
    }
}

fn find_note_by_index_from_end(entries_dir: &Path, index: usize) -> Result<PathBuf> {
    if index == 0 {
        bail!("--last index must be 1 or greater.");
    }
    let mut entries: Vec<_> = fs::read_dir(entries_dir)?.filter_map(Result::ok).collect();
    let total_jots = entries.len();

    if total_jots == 0 {
        bail!("No jots exist to act upon.");
    }

    if index > total_jots {
        bail!(
            "Index out of bounds. You asked for the {}{} last jot, but only {} exist.",
            index,
            get_ordinal_suffix(index),
            total_jots
        );
    }

    // Sort by filename, which is chronological
    entries.sort_by_key(|e| e.file_name());

    let target_index = total_jots - index;
    entries
        .get(target_index)
        .map(|e| e.path())
        .with_context(|| "Failed to get entry at calculated index. This is an unexpected error.")
}

// Main entrypoint
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
            Commands::On { date_spec, compile } => command_on(&entries_dir, &date_spec, compile)?,
            Commands::Edit { id_prefix, last } => {
                let note_path = get_note_path_for_action(&entries_dir, id_prefix, last)?;
                command_edit(note_path)?;
            }
            Commands::Show { id_prefix, last } => {
                let note_path = get_note_path_for_action(&entries_dir, id_prefix, last)?;
                command_show(note_path)?;
            }
            Commands::Delete {
                id_prefix,
                last,
                force,
            } => {
                let note_path = get_note_path_for_action(&entries_dir, id_prefix, last)?;
                command_delete(note_path, force)?;
            }
            Commands::Info(args) => command_info(&entries_dir, args)?,
        }
    } else if !cli.message.is_empty() {
        let message = cli.message.join(" ");
        command_down(&entries_dir, &message, cli.tags)?;
    } else {
        println!(
            "No message provided. Use 'rjot \"your message\"' or a subcommand like 'rjot list'."
        );
        println!("\nFor more information, try 'rjot --help'");
    }

    Ok(())
}

fn get_note_path_for_action(
    entries_dir: &Path,
    id_prefix: Option<String>,
    last: Option<usize>,
) -> Result<PathBuf> {
    if let Some(index) = last {
        find_note_by_index_from_end(entries_dir, index)
    } else if let Some(prefix) = id_prefix {
        find_unique_note_by_prefix(entries_dir, &prefix)
    } else {
        // This case should be prevented by clap's `required = true` on the group
        unreachable!();
    }
}

// Command logic

fn command_on(entries_dir: &PathBuf, date_spec: &str, compile: bool) -> Result<()> {
    // ...
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
        compile_notes(matches)?
    } else {
        display_note_list(matches)
    }
    Ok(())
}

fn command_down(entries_dir: &Path, message: &str, tags: Option<Vec<String>>) -> Result<()> {
    // ...
    let mut content = String::new();
    if let Some(tags) = tags {
        if !tags.is_empty() {
            content.push_str("---\ntags:\n");
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
    // ...
    let editor = env::var("EDITOR")?;
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
        bail!("Editor exited with a non-zero status.");
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
    // ...
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
    // ...
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
    // ...
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
    // ...
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
        compile_notes(matches)?
    } else {
        display_note_list(matches)
    }
    Ok(())
}

fn command_by_week(entries_dir: &PathBuf, compile: bool) -> Result<()> {
    // ...
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
        compile_notes(matches)?
    } else {
        display_note_list(matches)
    }
    Ok(())
}

fn command_edit(note_path: PathBuf) -> Result<()> {
    let editor =
        env::var("EDITOR").with_context(|| "The '$EDITOR' environment variable is not set.")?;
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

fn command_show(note_path: PathBuf) -> Result<()> {
    let content = fs::read_to_string(&note_path)?;
    println!("{}", content);
    Ok(())
}

fn command_delete(note_path: PathBuf, force: bool) -> Result<()> {
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

fn command_info(entries_dir: &PathBuf, args: InfoArgs) -> Result<()> {
    // ...
    if !args.paths && !args.stats {
        println!("Please provide a flag to the info command, e.g., `rjot info --paths` or `rjot info --stats`");
        println!("\nFor more information, try '--help'");
        return Ok(());
    }
    if args.paths {
        println!("--- rjot paths ---");
        println!("Root Directory:  {:?}", get_rjot_dir_root()?);
        println!("Entries:         {:?}", entries_dir);
        println!("Templates:       {:?}", get_templates_dir()?);
    }
    if args.stats {
        println!("\n--- rjot stats ---");
        let entries = fs::read_dir(entries_dir)?;
        let mut note_count = 0;
        let mut tag_counts: HashMap<String, usize> = HashMap::new();
        for entry in entries.filter_map(Result::ok) {
            note_count += 1;
            let note = parse_note_from_file(&entry.path())?;
            if let Some(tags) = note.frontmatter.tags {
                for tag in tags {
                    *tag_counts.entry(tag).or_insert(0) += 1;
                }
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

#[cfg(test)]
mod tests {
    use super::*; // Import everything from the parent module (our main.rs)

    #[test]
    fn test_ordinal_suffix() {
        assert_eq!(get_ordinal_suffix(1), "st");
        assert_eq!(get_ordinal_suffix(2), "nd");
        assert_eq!(get_ordinal_suffix(3), "rd");
        assert_eq!(get_ordinal_suffix(4), "th");
        assert_eq!(get_ordinal_suffix(10), "th");
        assert_eq!(get_ordinal_suffix(11), "th");
        assert_eq!(get_ordinal_suffix(12), "th");
        assert_eq!(get_ordinal_suffix(13), "th");
        assert_eq!(get_ordinal_suffix(21), "st");
        assert_eq!(get_ordinal_suffix(22), "nd");
        assert_eq!(get_ordinal_suffix(23), "rd");
        assert_eq!(get_ordinal_suffix(101), "st");
    }
}
