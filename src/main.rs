use std::env;
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;
use std::process::Command;
use anyhow::{bail, Context, Result};
use chrono::Local;
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(version, about = "A minimalist, command-line journal.")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// The message to jot down. If no subcommand is used, this is the default action.
    /// Can be combined with flags from 'new' for a one-liner with metadata.
    message: Vec<String>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Create a new jot using the default editor
    New,
    /// List the last 10 jots
    List,
}

/// Gets the root directory for all jot data, ensuring it exists.
/// Honors the JOT_DIR environment variable if set.
fn get_jot_dir() -> Result<PathBuf> {
    let path = match env::var("JOT_DIR") {
        Ok(val) => PathBuf::from(val),
        Err(_) => {
            // Use the standard user config directory.
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


fn main() -> Result<()> {
    let cli = Cli::parse();
    let jot_dir = get_jot_dir()?;

    // Decide which command to run
    if let Some(command) = cli.command {
        match command {
            Commands::New => command_new(&jot_dir)?,
            Commands::List => command_list(&jot_dir)?,
        }
    } else {
        // No subcommand was provided, so treat it as a direct jot.
        if cli.message.is_empty() {
             // If no subcommand AND no message, show help.
            println!("No message provided. Use 'jot \"your message\"' or a subcommand like 'jot new'.");
            println!("\nFor more information, try '--help'");
        } else {
            let message = cli.message.join(" ");
            command_now(&jot_dir, &message)?;
        }
    }

    Ok(())
}

/// Handles the default action of creating a jot directly from arguments.
fn command_now(jot_dir: &PathBuf, message: &str) -> Result<()> {
    println!("Jotting down: \"{}\"", message);
    let now = Local::now();
    let filename = now.format("%Y-%m-%d-%H%M%S.md").to_string();
    let file_path = jot_dir.join(filename);

    fs::write(&file_path, message)
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
    
    // Check if the file is empty after editing. If so, delete it.
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
    println!("Listing last 10 jots from {:?}:", jot_dir);

    let entries = fs::read_dir(jot_dir)
        .with_context(|| format!("Failed to read jot directory at {:?}", jot_dir))?;

    let mut sorted_entries: Vec<_> = entries
        .filter_map(Result::ok) // Ignore files we can't read
        .collect();

    // Sort by filename, which is chronological
    sorted_entries.sort_by_key(|e| e.file_name());
    // Reverse to get most recent first
    sorted_entries.reverse();

    if sorted_entries.is_empty() {
        println!("\nNo jots found. Get jotting!");
        return Ok(());
    }
    
    println!("\n{:<22} {}", "ID", "FIRST LINE");
    println!("{:-<22} {:-<50}", "", "");

    for entry in sorted_entries.iter().take(10) {
        let path = entry.path();
        let filename = entry.file_name().to_string_lossy().to_string();
        let id = filename.replace(".md", "");

        let contents = fs::read_to_string(&path).unwrap_or_else(|_| "Could not read file.".to_string());
        let first_line = contents.lines().next().unwrap_or("").trim();
        
        println!("{:<22} {}", id, first_line);
    }

    Ok(())
}
