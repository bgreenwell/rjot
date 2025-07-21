//! This module defines the entire command-line interface for the `rjot` application.
//!
//! It uses the `clap` crate with the `derive` feature to declaratively build the CLI
//! structure from Rust structs and enums. This includes all subcommands, arguments,
//! and their help messages.

use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

/// The main CLI structure, representing the `rjot` command itself.
#[derive(Parser, Debug)]
#[command(name = "rjot", version, about = "A minimalist, command-line journal.")]
pub struct Cli {
    /// The subcommand to execute. If no subcommand is provided, `rjot` will
    /// treat the input as a new note for the default action.
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Tags to add to a new jot, specified alongside the message.
    ///
    /// Accepts multiple values, either comma-separated or space-separated.
    #[arg(long, short, value_delimiter = ',', num_args(1..))]
    pub tags: Option<Vec<String>>,

    /// Run a command in a specific notebook without switching the active one.
    #[arg(long, global = true)]
    pub notebook: Option<String>,

    /// The message for a new jot. This captures all positional arguments
    /// that are not part of a subcommand.
    pub message: Vec<String>,
}

/// Parses a key-value pair from the command line.
fn parse_key_val(s: &str) -> Result<(String, String), String> {
    s.split_once('=')
        .map(|(key, value)| (key.to_string(), value.to_string()))
        .ok_or_else(|| format!("invalid key-value pair: {s}"))
}

/// An enumeration of all possible subcommands `rjot` can execute.
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Create a new jot using an editor, optionally with a template.
    New {
        /// The name of the template to use from the templates directory.
        #[arg(long, short)]
        template: Option<String>,

        /// Set custom variables for the template (e.g., -v key=value).
        #[arg(long, short = 'v', value_parser = parse_key_val)]
        variables: Vec<(String, String)>,
    },
    /// List the most recent jots.
    List {
        /// The number of jots to list. Defaults to 10.
        count: Option<usize>,
        /// A flag to show only pinned jots.
        #[arg(long, short)]
        pinned: bool,
        /// A flag to show only jots containing incomplete tasks.
        #[arg(long)] // Or short('t') if you prefer
        tasks: bool,
    },
    /// Pin a jot.
    Pin {
        /// The prefix of the jot ID to pin. Must be unique.
        #[arg(group = "target", required = true)]
        id_prefix: Option<String>,
        /// Pin the Nth most recent jot (e.g., --last=1 or just --last).
        #[arg(long, short, group = "target", num_args(0..=1), default_missing_value = "1")]
        last: Option<usize>,
    },
    /// Unpin a jot.
    Unpin {
        /// The prefix of the jot ID to unpin. Must be unique.
        #[arg(group = "target", required = true)]
        id_prefix: Option<String>,
        /// Unpin the Nth most recent jot.
        #[arg(long, short, group = "target", num_args(0..=1), default_missing_value = "1")]
        last: Option<usize>,
    },
    /// Create a new jot formatted as a task.
    #[command(aliases = ["t", "todo"])] // Optional aliases
    Task {
        /// The content of the task.
        #[arg(required = true)]
        message: String,
    },
    /// Find jots by searching their content.
    Find {
        /// Text to search for, case-insensitively.
        #[arg(required = true)]
        query: String,

        /// Search across all notebooks.
        #[arg(long, short)] // Or --global if you prefer
        all: bool,
    },
    /// Interactively select a note using a fuzzy finder.
    #[command(alias = "s")]
    #[cfg(not(windows))] // Fuzzy finder is not supported on Windows
    Select,
    /// List jots that have specific tags.
    Tags {
        /// Tags to filter by (can be comma-separated or space-separated).
        #[arg(required = true, value_delimiter = ',')]
        tags: Vec<String>,
    },
    /// List jots from today.
    Today {
        /// Compile all of today's jots into a single summary.
        #[arg(long, short)]
        compile: bool,
    },
    /// List jots from yesterday.
    Yesterday {
        #[arg(long, short)]
        compile: bool,
    },
    /// List jots from this week.
    Week {
        #[arg(long, short)]
        compile: bool,
    },
    /// List jots from a specific date or date range.
    On {
        /// The date (YYYY-MM-DD) or range (YYYY-MM-DD..YYYY-MM-DD) to filter by.
        #[arg(required = true)]
        date_spec: String,
        #[arg(long, short)]
        compile: bool,
    },
    /// Open an existing jot in the default editor.
    Edit {
        /// The prefix of the jot ID to edit. Must be unique.
        #[arg(group = "target", required = true)]
        id_prefix: Option<String>,
        /// Edit the Nth most recent jot (e.g., --last=1 or just --last).
        #[arg(long, short, group = "target", num_args(0..=1), default_missing_value = "1")]
        last: Option<usize>,
    },
    /// Display the full content of a jot in the terminal.
    Show {
        /// The prefix of the jot ID to show. Must be unique.
        #[arg(group = "target", required = true)]
        id_prefix: Option<String>,
        /// Show the Nth most recent jot.
        #[arg(long, short, group = "target", num_args(0..=1), default_missing_value = "1")]
        last: Option<usize>,
    },
    /// Delete a jot with confirmation.
    #[command(alias = "rm")]
    Delete {
        /// The prefix of the jot ID to delete. Must be unique.
        #[arg(group = "target", required = true)]
        id_prefix: Option<String>,
        /// Delete the Nth most recent jot.
        #[arg(long, short, group = "target", num_args(0..=1), default_missing_value = "1")]
        last: Option<usize>,
        /// Force deletion without a confirmation prompt.
        #[arg(long, short)]
        force: bool,
    },
    /// Display information about your rjot setup.
    Info(InfoArgs),
    /// Manage tags on an existing jot.
    Tag(TagArgs),
    /// Manage notebooks for organizing jots.
    #[command(alias = "n")]
    Notebook(NotebookArgs),
    /// Initialize the rjot directory, optionally with Git and/or encryption.
    Init {
        /// Initialize the rjot directory as a Git repository.
        #[arg(long)]
        git: bool,
        /// Encrypt the rjot directory with a new identity.
        #[arg(long)]
        encrypt: bool,
    },
    /// Commit and push changes to a remote Git repository.
    Sync,
    /// Permanently decrypt all notes in the rjot directory.
    Decrypt {
        /// Force decryption without a confirmation prompt.
        #[arg(long, short)]
        force: bool,
    },
    /// Export a notebook to a ZIP archive or a JSON file.
    Export(ExportArgs),

    /// Import a notebook from a ZIP archive or a JSON file.
    Import(ImportArgs),

    /// Enter the interactive rjot shell.
    #[command(alias = "sh")]
    Shell,
}

/// Arguments for the `notebook` subcommand.
#[derive(Args, Debug)]
pub struct NotebookArgs {
    /// The notebook management action to perform.
    #[command(subcommand)]
    pub action: NotebookAction,
}

/// An enumeration of all possible notebook management actions.
#[derive(Subcommand, Debug)]
pub enum NotebookAction {
    /// Create a new, empty notebook.
    New {
        /// The name for the new notebook.
        #[arg(required = true)]
        name: String,
    },
    /// List all available notebooks.
    #[command(alias = "ls")]
    List,
    /// Print the command to switch the active notebook for the current shell session.
    ///
    /// Usage: eval $(rjot notebook use <NAME>)
    Use {
        /// The name of the notebook to switch to.
        #[arg(required = true)]
        name: String,
    },
    /// Show the currently active notebook.
    Status,
}

/// Arguments for the `info` subcommand.
#[derive(Args, Debug)]
pub struct InfoArgs {
    /// Display the paths used by rjot for storage and templates.
    #[arg(long)]
    pub paths: bool,
    /// Display statistics about your jots, like total count and tag frequency.
    #[arg(long)]
    pub stats: bool,
    /// Show stats for all notebooks combined.
    #[arg(long, requires = "stats")]
    pub all: bool,
}

/// Arguments for the `tag` subcommand.
#[derive(Args, Debug)]
pub struct TagArgs {
    /// The tag management action to perform.
    #[command(subcommand)]
    pub action: TagAction,
}

/// An enumeration of all possible tag management actions.
#[derive(Subcommand, Debug)]
pub enum TagAction {
    /// Add one or more tags to a jot.
    Add {
        /// The ID prefix of the note to tag.
        #[arg(long, short = 'p', group = "target")]
        id_prefix: Option<String>,
        /// Target the Nth most recent note.
        #[arg(long, short, group = "target")]
        last: Option<usize>,
        /// The tags to add.
        #[arg(required = true, value_delimiter = ',')]
        tags: Vec<String>,
    },
    /// Remove one or more tags from a jot.
    #[command(alias = "rm")]
    Remove {
        /// The ID prefix of the note to modify.
        #[arg(long, short = 'p', group = "target")]
        id_prefix: Option<String>,
        /// Target the Nth most recent note.
        #[arg(long, short, group = "target")]
        last: Option<usize>,
        /// The tags to remove.
        #[arg(required = true, value_delimiter = ',')]
        tags: Vec<String>,
    },
    /// Overwrite all existing tags on a jot.
    Set {
        /// The ID prefix of the note to modify.
        #[arg(long, short = 'p', group = "target")]
        id_prefix: Option<String>,
        /// Target the Nth most recent note.
        #[arg(long, short, group = "target")]
        last: Option<usize>,
        /// The new set of tags.
        #[arg(required = true, value_delimiter = ',')]
        tags: Vec<String>,
    },
}

/// Arguments for the `export` subcommand.
#[derive(Args, Debug)]
pub struct ExportArgs {
    /// The name of the notebook to export.
    #[arg(required = true)]
    pub notebook_name: String,

    /// The format for the export (zip or json).
    #[arg(long, short, default_value = "zip")]
    pub format: String,

    /// The path for the output file.
    #[arg(long, short, required = true)]
    pub output: PathBuf,
}

/// Arguments for the `import` subcommand.
#[derive(Args, Debug)]
pub struct ImportArgs {
    /// The path to the file to import.
    #[arg(required = true)]
    pub file_path: PathBuf,
}
