use clap::{Args, Parser, Subcommand};

// This file contains all the command-line interface definitions using `clap`.

#[derive(Parser, Debug)]
#[command(name = "rjot", version, about = "A minimalist, command-line journal.")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    #[arg(long, short, value_delimiter = ',', num_args(1..))]
    pub tags: Option<Vec<String>>,

    pub message: Vec<String>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Create a new jot using an editor, optionally with a template
    New {
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
        #[arg(required = true)]
        date_spec: String,
        #[arg(long, short)]
        compile: bool,
    },
    /// Open an existing jot in the default editor
    Edit {
        #[arg(group = "target", required = true)]
        id_prefix: Option<String>,
        #[arg(long, short, group = "target", num_args(0..=1), default_missing_value = "1")]
        last: Option<usize>,
    },
    /// Display the full content of a jot in the terminal
    Show {
        #[arg(group = "target", required = true)]
        id_prefix: Option<String>,
        #[arg(long, short, group = "target", num_args(0..=1), default_missing_value = "1")]
        last: Option<usize>,
    },
    /// Delete a jot with confirmation
    #[command(alias = "rm")]
    Delete {
        #[arg(group = "target", required = true)]
        id_prefix: Option<String>,
        #[arg(long, short, group = "target", num_args(0..=1), default_missing_value = "1")]
        last: Option<usize>,
        #[arg(long, short)]
        force: bool,
    },
    /// Display information about your rjot setup
    Info(InfoArgs),
    /// Manage tags on an existing jot
    Tag(TagArgs),
}

#[derive(Args, Debug)]
pub struct InfoArgs {
    #[arg(long)]
    pub paths: bool,
    #[arg(long)]
    pub stats: bool,
}

#[derive(Args, Debug)]
pub struct TagArgs {
    #[command(subcommand)]
    pub action: TagAction,
}

#[derive(Subcommand, Debug)]
pub enum TagAction {
    /// Add one or more tags to a jot
    Add {
        #[arg(long, short = 'p', group = "target")]
        id_prefix: Option<String>,
        #[arg(long, short, group = "target")]
        last: Option<usize>,
        #[arg(required = true, value_delimiter = ',')]
        tags: Vec<String>,
    },
    /// Remove one or more tags from a jot
    #[command(alias = "rm")]
    Remove {
        #[arg(long, short = 'p', group = "target")]
        id_prefix: Option<String>,
        #[arg(long, short, group = "target")]
        last: Option<usize>,
        #[arg(required = true, value_delimiter = ',')]
        tags: Vec<String>,
    },
    /// Overwrite all existing tags on a jot
    Set {
        #[arg(long, short = 'p', group = "target")]
        id_prefix: Option<String>,
        #[arg(long, short, group = "target")]
        last: Option<usize>,
        #[arg(required = true, value_delimiter = ',')]
        tags: Vec<String>,
    },
}
