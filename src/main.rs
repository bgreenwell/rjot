// Declare the modules
mod cli;
mod commands;
mod helpers;

use anyhow::Result;
use clap::Parser;

use cli::Commands;

fn main() -> Result<()> {
    // The main function's responsibility is now just to parse arguments
    // and dispatch to the correct command handler.
    let cli = cli::Cli::parse();
    let entries_dir = helpers::get_entries_dir()?;

    // The main match statement now includes the new `init` and `sync` commands.
    match cli.command {
        Some(command) => match command {
            Commands::New { template } => commands::command_new(&entries_dir, template)?,
            Commands::List => commands::command_list(&entries_dir)?,
            Commands::Find { query } => commands::command_find(&entries_dir, &query)?,
            Commands::Tags { tags } => commands::command_tags_filter(&entries_dir, &tags)?,
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
            // New commands are handled here
            Commands::Init { git } => commands::command_init(git)?,
            Commands::Sync => commands::command_sync()?,
        },
        None => {
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
