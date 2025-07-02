//! `rjot` is a minimalist, command-line journal that's fast, private, and git-friendly.
//!
//! This crate provides the main entrypoint and command-line parsing logic. It orchestrates
//! the different modules to execute user commands.

// Declare the modules that make up the application.
mod cli;
mod commands;
mod helpers;

use anyhow::Result;
use clap::Parser;

use cli::Commands;
// use helpers::DEFAULT_NOTEBOOK_NAME; // No longer needed here

/// The main entrypoint for the rjot application.
///
/// This function parses command-line arguments, gets the necessary directory paths,
/// and dispatches to the appropriate command handler based on user input.
fn main() -> Result<()> {
    let cli = cli::Cli::parse();

    // Determine the entries_dir for commands that need it.
    // Some commands (Init, Sync, Notebook) handle paths differently or don't need this entries_dir.
    let entries_dir_res = if let Some(notebook_name) = &cli.notebook_opt {
        helpers::get_specific_notebook_dir(notebook_name)
    } else {
        helpers::get_entries_dir() // This handles RJOT_ACTIVE_NOTEBOOK or default
    };

    // Match on the subcommand provided by the user.
    // For commands that don't use the standard `entries_dir` (like Init, Sync, Notebook),
    // they are matched first. Other commands will use the resolved `entries_dir`.
    match cli.command {
        Some(Commands::Init { git, encrypt }) => {
            commands::command_init(git, encrypt)?
        }
        Some(Commands::Sync) => {
            commands::command_sync()?
        }
        Some(Commands::Notebook(notebook_cmd)) => {
            // The command_notebook function handles its own path logic internally if needed
            // (e.g., for listing all notebooks, creating new ones)
            // It doesn't rely on a pre-calculated `entries_dir` in the same way other commands do.
            commands::command_notebook(notebook_cmd)?
        }
        Some(other_command) => {
            // All other commands require entries_dir to be successfully resolved.
            let entries_dir = entries_dir_res?;
            match other_command {
                Commands::New { template } => commands::command_new(&entries_dir, template)?,
                Commands::List { count } => commands::command_list(&entries_dir, count)?,
                Commands::Find { query } => commands::command_find(&entries_dir, &query)?,
                Commands::Tags { tags } => commands::command_tags_filter(&entries_dir, &tags)?,
                #[cfg(not(windows))]
                Commands::Select => commands::command_select(&entries_dir)?,
                Commands::Today { compile } => commands::command_today(&entries_dir, compile)?,
                Commands::Yesterday { compile } => commands::command_yesterday(&entries_dir, compile)?,
                Commands::Week { compile } => commands::command_by_week(&entries_dir, compile)?,
                Commands::On { date_spec, compile } => {
                    commands::command_on(&entries_dir, &date_spec, compile)?
                }
                Commands::Edit { id_prefix, last } => {
                    let note_path = helpers::get_note_path_for_action(&entries_dir, id_prefix, last)?;
                    commands::command_edit(note_path)?;
                }
                Commands::Show { id_prefix, last } => {
                    let note_path = helpers::get_note_path_for_action(&entries_dir, id_prefix, last)?;
                    commands::command_show(note_path)?;
                }
                Commands::Delete {
                    id_prefix,
                    last,
                    force,
                } => {
                    let note_path = helpers::get_note_path_for_action(&entries_dir, id_prefix, last)?;
                    commands::command_delete(note_path, force)?;
                }
                Commands::Info(args) => commands::command_info(&entries_dir, args)?,
                Commands::Tag(args) => commands::command_tag(&entries_dir, args)?,
                Commands::Decrypt { force } => commands::command_decrypt(&entries_dir, force)?,
                // Init, Sync, Notebook already handled. This makes the match exhaustive.
                Commands::Init { .. } | Commands::Sync | Commands::Notebook(_) => unreachable!("Already handled"),
            }
        }
        // If no subcommand is given, this is the default action (jot down a message).
        None => {
            let entries_dir = entries_dir_res?; // Use the already resolved entries_dir
            if !cli.message.is_empty() {
                let message = cli.message.join(" ");
                commands::command_down(&entries_dir, &message, cli.tags)?;
            } else {
                println!(
                    "No message provided. Use 'rjot <MESSAGE>' or a subcommand like 'rjot list'."
                );
                println!("\nFor more information, try 'rjot --help'");
            }
        }
    }

    Ok(())
}

// Unit tests for helpers that are simple and don't require file system access.
#[cfg(test)]
mod tests {
    use crate::helpers::get_ordinal_suffix;

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
