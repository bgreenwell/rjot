//! This module contains helper functions and data structures used across the application.
//!
//! It handles tasks like file system interactions, configuration management, note parsing,
//! and encryption/decryption logic, centralizing common functionality.

use std::env;
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use age::{
    x25519::{Identity, Recipient},
    Encryptor,
};
use anyhow::{anyhow, bail, Context, Result};
use serde::{Deserialize, Serialize};
use which::which;

pub const NOTEBOOKS_DIR_NAME: &str = "notebooks";
pub const DEFAULT_NOTEBOOK_NAME: &str = "default";
pub const ACTIVE_NOTEBOOK_ENV_VAR: &str = "RJOT_ACTIVE_NOTEBOOK";

// --- Data Structures ---

/// Represents the YAML frontmatter section of a note.
#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Frontmatter {
    /// A list of tags associated with the note.
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Represents a fully parsed jot note, including its metadata and content.
#[derive(Debug, Default)]
pub struct Note {
    pub id: String,
    pub path: PathBuf,
    pub frontmatter: Frontmatter,
    pub content: String,
}

/// Represents the `config.toml` file used for encryption settings.
#[derive(Serialize, Deserialize, Debug, Default)]
struct Config {
    /// The public key (`age` recipient) used for encrypting notes.
    recipient: Option<String>,
}

// --- Path and Editor Helpers ---

/// Gets the root directory for all `rjot` data, creating it if it doesn't exist.
///
/// Honors the `$RJOT_DIR` environment variable if set, otherwise uses the platform-specific
/// user config directory.
pub fn get_rjot_dir_root() -> Result<PathBuf> {
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

/// Gets the root directory for all notebooks, creating it if it doesn't exist.
pub fn get_notebooks_root_dir() -> Result<PathBuf> {
    let rjot_root = get_rjot_dir_root()?;
    let notebooks_root = rjot_root.join(NOTEBOOKS_DIR_NAME);
    if !notebooks_root.exists() {
        fs::create_dir_all(&notebooks_root)
            .with_context(|| format!("Failed to create notebooks directory at {:?}", notebooks_root))?;
    }
    Ok(notebooks_root)
}

/// Gets the directory for a specific notebook name, creating it if it doesn't exist.
pub fn get_specific_notebook_dir(notebook_name: &str) -> Result<PathBuf> {
    let notebooks_root = get_notebooks_root_dir()?;
    let notebook_path = notebooks_root.join(notebook_name);
    if !notebook_path.exists() {
        // println!("[Debug get_specific_notebook_dir] Path {:?} DOES NOT exist. Creating.", notebook_path);
        fs::create_dir_all(&notebook_path)
            .with_context(|| format!("Failed to create notebook directory at {:?}", notebook_path))?;
    } else {
        // println!("[Debug get_specific_notebook_dir] Path {:?} ALREADY exists.", notebook_path);
    }
    Ok(notebook_path)
}

/// Gets the directory where note entries are stored, considering the active notebook.
pub fn get_entries_dir() -> Result<PathBuf> {
    match env::var(ACTIVE_NOTEBOOK_ENV_VAR) {
        Ok(notebook_name) if !notebook_name.is_empty() => {
            let notebooks_root = get_notebooks_root_dir()?;
            let specific_notebook_path = notebooks_root.join(&notebook_name);

            if specific_notebook_path.exists() && specific_notebook_path.is_dir() {
                Ok(specific_notebook_path)
            } else {
                eprintln!(
                    "Warning: Notebook \"{}\" specified by {} does not exist. Falling back to default notebook.",
                    notebook_name, ACTIVE_NOTEBOOK_ENV_VAR
                );
                get_specific_notebook_dir(DEFAULT_NOTEBOOK_NAME)
            }
        }
        _ => {
            get_specific_notebook_dir(DEFAULT_NOTEBOOK_NAME)
        }
    }
}

/// Gets the directory where note templates are stored, ensuring it exists.
pub fn get_templates_dir() -> Result<PathBuf> {
    let root_dir = get_rjot_dir_root()?;
    let templates_dir = root_dir.join("templates");
    if !templates_dir.exists() {
        fs::create_dir_all(&templates_dir)?;
    }
    Ok(templates_dir)
}


/// Determines which command-line editor to use.
pub fn get_editor() -> Result<String> {
    if let Ok(editor) = env::var("EDITOR") {
        if !editor.is_empty() {
            return Ok(editor);
        }
    }
    #[cfg(unix)]
    let fallbacks = ["vim", "nvim", "nano"];
    #[cfg(windows)]
    let fallbacks = ["notepad.exe"];
    #[cfg(not(any(unix, windows)))]
    let fallbacks: [&str; 0] = [];

    for editor in fallbacks {
        if which(editor).is_ok() {
            return Ok(editor.to_string());
        }
    }
    bail!("Could not find a default editor. Please set the $EDITOR environment variable.")
}

// --- Core File I/O Logic ---

/// Writes content to a note file, encrypting it if encryption is enabled.
pub fn write_note_file(path: &Path, content: &str) -> Result<()> {
    let root_dir = get_rjot_dir_root()?;
    let config_path = root_dir.join("config.toml");
    let config: Config = if config_path.exists() {
        toml::from_str(&fs::read_to_string(config_path)?)?
    } else {
        Config::default()
    };

    // Parent directory should be guaranteed to exist by get_specific_notebook_dir or get_entries_dir
    if let Some(recipient_str) = config.recipient {
        let recipient: Recipient = recipient_str
            .parse()
            .map_err(|e| anyhow!("Failed to parse recipient from config: {}", e))?;
        let encrypted_bytes = {
            let encryptor = Encryptor::with_recipients(vec![Box::new(recipient)]);
            let mut encrypted = vec![];
            let mut writer = encryptor.expect("REASON").wrap_output(&mut encrypted)?;
            writer.write_all(content.as_bytes())?;
            writer.finish()?;
            encrypted
        };
        fs::write(path, encrypted_bytes)?;
    } else {
        fs::write(path, content)?;
    }
    Ok(())
}

/// Reads content from a note file, decrypting it if necessary.
pub fn read_note_file(path: &Path) -> Result<String> {
    let root_dir = get_rjot_dir_root()?;
    let identity_path = root_dir.join("identity.txt");
    let file_bytes = fs::read(path)?;

    if identity_path.exists() && file_bytes.starts_with(b"age-encryption.org") {
        let identity_str = fs::read_to_string(identity_path)?;
        let identity: Identity = identity_str
            .parse()
            .map_err(|_| anyhow!("Failed to parse identity file."))?;
        let decryptor = age::Decryptor::new(&file_bytes as &[u8])?;
        let mut decrypted_bytes = vec![];
        if let age::Decryptor::Recipients(reader) = decryptor {
            let identities: Vec<Box<dyn age::Identity>> = vec![Box::new(identity)];
            reader
                .decrypt(identities.iter().map(|i| i.as_ref()))?
                .read_to_end(&mut decrypted_bytes)?;
        } else {
            bail!("Expected recipients-based encryption");
        }
        Ok(String::from_utf8(decrypted_bytes)?)
    } else {
        Ok(String::from_utf8(file_bytes)?)
    }
}

// --- Other Helpers ---

/// Parses a file into a `Note` struct, separating frontmatter from content.
pub fn parse_note_from_file(path: &Path) -> Result<Note> {
    let filename = path.file_name().unwrap().to_string_lossy().to_string();
    let id = filename.replace(".md", "");
    let file_content =
        read_note_file(path).with_context(|| format!("Could not read file: {path:?}"))?;

    if file_content.starts_with("---") {
        if let Some(end_frontmatter) = file_content.get(3..).and_then(|s| s.find("---")) {
            let frontmatter_str = &file_content[3..(3 + end_frontmatter)];
            let content = file_content[(3 + end_frontmatter + 3)..].trim().to_string();
            let frontmatter: Frontmatter = serde_yaml::from_str(frontmatter_str)
                .with_context(|| format!("Failed to parse YAML frontmatter in {path:?}"))?;
            return Ok(Note {
                id,
                path: path.to_path_buf(),
                frontmatter,
                content,
            });
        }
    }
    Ok(Note {
        id,
        path: path.to_path_buf(),
        frontmatter: Frontmatter::default(),
        content: file_content,
    })
}

/// Determines which note to act on based on user input (ID prefix or `--last` flag).
pub fn get_note_path_for_action(
    entries_dir: &Path,
    id_prefix: Option<String>,
    last: Option<usize>,
) -> Result<PathBuf> {
    if let Some(index) = last {
        if id_prefix.is_some() {
            bail!("Cannot use an ID prefix and the --last flag at the same time.");
        }
        find_note_by_index_from_end(entries_dir, index)
    } else if let Some(prefix) = id_prefix {
        find_unique_note_by_prefix(entries_dir, &prefix)
    } else {
        unreachable!();
    }
}

/// Formats and prints a list of notes to the console.
pub fn display_note_list(notes: Vec<Note>) {
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

/// Formats and prints a compiled summary of notes to the console.
pub fn compile_notes(notes: Vec<Note>) -> Result<()> {
    for note in notes {
        println!("---\n\n# {}\n\n{}", note.id, note.content);
    }
    Ok(())
}

/// Finds a single, unique note file based on a starting prefix of its ID.
pub fn find_unique_note_by_prefix(entries_dir: &Path, prefix: &str) -> Result<PathBuf> {
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

/// Gets the appropriate ordinal suffix for a number (e.g., "st", "nd", "rd", "th").
pub fn get_ordinal_suffix(n: usize) -> &'static str {
    if (11..=13).contains(&(n % 100)) {
        "th"
    } else {
        match n % 10 {
            1 => "st",
            2 => "nd",
            3 => "rd",
            _ => "th",
        }
    }
}

/// Finds the Nth most recent note.
pub fn find_note_by_index_from_end(entries_dir: &Path, index: usize) -> Result<PathBuf> {
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
    entries.sort_by_key(|e| e.file_name());
    let target_index = total_jots - index;
    entries
        .get(target_index)
        .map(|e| e.path())
        .with_context(|| "Failed to get entry at calculated index.")
}
