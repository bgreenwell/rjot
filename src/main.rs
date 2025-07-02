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

/// The main entrypoint for the rjot application.
///
/// This function parses command-line arguments, gets the necessary directory paths,
/// and dispatches to the appropriate command handler based on user input.
fn main() -> Result<()> {
    let cli = cli::Cli::parse();
    let entries_dir = helpers::get_entries_dir()?;

    // Match on the subcommand provided by the user.
    match cli.command {
        Some(command) => match command {
            Commands::New { template } => commands::command_new(&entries_dir, template)?,
            Commands::List { count } => commands::command_list(&entries_dir, count)?,
            Commands::Find { query } => commands::command_find(&entries_dir, &query)?,
            Commands::Tags { tags } => commands::command_tags_filter(&entries_dir, &tags)?,
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
            Commands::Init { git, encrypt } => commands::command_init(git, encrypt)?,
            Commands::Sync => commands::command_sync()?,
            Commands::Decrypt { force } => commands::command_decrypt(&entries_dir, force)?,
        },
        // If no subcommand is given, this is the default action.
        None => {
            if !cli.message.is_empty() {
                let message = cli.message.join(" ");
                commands::command_down(&entries_dir, &message, cli.tags)?;
            } else {
                // If no subcommand and no message, show a brief help message.
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
