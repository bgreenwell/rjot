//! This module defines the entire command-line interface for the `rjot` application.
//!
//! It uses the `clap` crate with the `derive` feature to declaratively build the CLI
//! structure from Rust structs and enums. This includes all subcommands, arguments,
//! and their help messages.

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

    /// The message for a new jot. This captures all positional arguments
    /// that are not part of a subcommand.
    pub message: Vec<String>,
}

/// An enumeration of all possible subcommands `rjot` can execute.
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Create a new jot using an editor, optionally with a template.
    New {
        /// The name of the template to use from the templates directory.
        #[arg(long, short)]
        template: Option<String>,
    },
    /// List the most recent jots.
    List {
        /// The number of jots to list. Defaults to 10.
        count: Option<usize>,
    },
    /// Find jots by searching their content.
    Find {
        /// Text to search for, case-insensitively.
        #[arg(required = true)]
        query: String,
    },
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
