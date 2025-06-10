use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use which::which;

// This file contains all the data structures and helper/utility functions.

// --- Data Structures ---
#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Frontmatter {
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Default)]
pub struct Note {
    pub id: String,
    pub path: PathBuf,
    pub frontmatter: Frontmatter,
    pub content: String,
}

// --- Helper Functions ---
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

pub fn get_entries_dir() -> Result<PathBuf> {
    let root_dir = get_rjot_dir_root()?;
    let entries_dir = root_dir.join("entries");
    if !entries_dir.exists() {
        fs::create_dir_all(&entries_dir)?;
    }
    Ok(entries_dir)
}

pub fn get_templates_dir() -> Result<PathBuf> {
    let root_dir = get_rjot_dir_root()?;
    let templates_dir = root_dir.join("templates");
    if !templates_dir.exists() {
        fs::create_dir_all(&templates_dir)?;
    }
    Ok(templates_dir)
}

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

pub fn parse_note_from_file(path: &Path) -> Result<Note> {
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

pub fn compile_notes(notes: Vec<Note>) -> Result<()> {
    for note in notes {
        println!("---\n\n# {}\n\n{}", note.id, note.content);
    }
    Ok(())
}

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
        .with_context(|| "Failed to get entry at calculated index. This is an unexpected error.")
}

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
