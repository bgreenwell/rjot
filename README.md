# rjot <img src="assets/logo-color.png" align="right" height="120" alt="rjot logo" />

A minimalist, command-line jotting utility that's fast, private, git-friendly, and written in Rust.

[![Build Status](https://github.com/bgreenwell/rjot/actions/workflows/rust.yml/badge.svg)](https://github.com/bgreenwell/rjot/actions)
[![Crates.io](https://img.shields.io/crates/v/rjot.svg?label=crates.io)](https://crates.io/crates/rjot)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## The vision

`rjot` is a tool for capturing thoughts at the speed of typing. It's built on a few core principles:

  * **CLI-first, not CLI-only**: The terminal is the most powerful and frictionless interface for capturing text. `rjot` is designed to be a first-class citizen of your command line.
  * **Plain text is sacred**: Your data is just a folder of Markdown files. It will always be readable, editable, and portable with or without `rjot`. No proprietary formats, no databases, no lock-in.
  * **You own your data**: `rjot` will never push you to a proprietary sync service. It's designed from the ground up to empower you with control over your own data.

This project aims to be the perfect, minimalist companion for developers, writers, and anyone who lives in the terminal.

## Features

  * **Instant capture**: Jot down a thought instantly from the command line.
  * **Task management**: Quickly create tasks and view all pending items across a notebook.
  * **Multiple notebooks**: Organize your jots into separate collections (e.g., `work`, `personal`, `project-x`).
  * **Editor integration**: Use `rjot new` to open your favorite editor (`$EDITOR`) for longer-form entries with template support.
  * **Pinning jots**: Mark essential notes with `rjot pin` to keep them readily accessible with `rjot list --pinned`.
  * **Powerful search & filtering**: Full-text search, tag-based filtering, and time-based views (`today`, `week`, `on <date>`, or `on <date-from>...<date-to>`).
  * **Note management**: Easily `show`, `edit`, `tag`, or `delete` any note using a unique ID prefix or its recency (`--last` or `--last=3`).
  * **Standard & configurable**: Follows platform-specific conventions for data storage and respects standard environment variables.

## Installation

**Note:** Once this project gains stable releases, you will be able to install it via your system's package manager (e.g., `apt`, `brew`, etc.). Until then, you can use the methods below.

### From crates.io (recommended)

This method automatically downloads, compiles, and installs `rjot` on your system.

1.  **Install the Rust toolchain**

    If you don't already have it, install Rust from the official site: [rustup.rs](https://rustup.rs/).

2.  **Install `rjot`**

    ```sh
    cargo install rjot
    ```

    This will place the `rjot` executable in your cargo binary path (usually `~/.cargo/bin/`), making it available from anywhere in your terminal.

### From source

To build the very latest version directly from the source code:

```sh
git clone https://github.com/bgreenwell/rjot.git
cd rjot
cargo install --path .
```

-----

## Usage guide

### A note on shells and quotes

Your command-line shell (like Bash or Zsh) can interpret special characters like `!` or expand variables like `$USER` inside double quotes (`"`). This can cause unexpected behavior.

**The best practice is to always use single quotes (`'`) for your messages.** This tells the shell to treat every character literally.

```sh
# GOOD: This works perfectly.
❯ rjot 'This is a great idea!'

# BAD: This will probably fail!
❯ rjot "This is a great idea!"
```

### Creating notes

By default, all jots are created in your active notebook (which is `default` until you change it).

**1. Jot down a quick note (the default action):**

```sh
❯ rjot 'This is a quick thought I want to save.'
```

**2. Create a tagged, one-liner note:**
The `--tags` (or `-t`) flag accepts space-separated or comma-separated values.

```sh
❯ rjot 'A great idea for the project' --tags project rust
```

**3. Create a longer note in your editor:**

```sh
# This opens your default $EDITOR
❯ rjot new

# Use a custom template for structured notes
❯ rjot new --template meeting.md
```

### Working with notebooks

`rjot` allows you to organize your notes into separate notebooks. All commands operate on the currently active notebook.

**1. Create a new notebook:**

```sh
❯ rjot notebook new project-icarus
Successfully created new notebook: 'project-icarus'
```

**2. List all available notebooks:**
An asterisk (`*`) indicates the currently active notebook.

```sh
❯ rjot notebook list
Available notebooks (* indicates active):
  * default
  project-icarus
```

**3. Switch your active notebook:**
Because a program can't change its parent shell's environment, you must use `eval` to make the change take effect for your current terminal session.

```sh
❯ eval $(rjot notebook use project-icarus)

# To check which notebook is active
❯ rjot notebook status
Active notebook: project-icarus
```

**4. Jot in a different notebook without switching:**
You can use the global `--notebook` flag to perform a single action in another notebook.

```sh
# Even if 'project-icarus' is active, this goes to 'personal'
❯ rjot 'Remember to buy milk' --notebook personal
```

### Viewing and filtering notes

All viewing and filtering commands are scoped to the active notebook.

**1. List a specific number of recent notes:**
The `list` command defaults to showing 10 notes, but you can provide a number to see more or less.

```sh
❯ rjot list
❯ rjot list 5
```

**2. Full-text search of all notes:**

```sh
❯ rjot find 'productivity'
```

**3. Filter by one or more tags:**

```sh
# Find notes with BOTH 'rust' and 'cli' tags in the active notebook
❯ rjot tags rust,cli
```

**4. View notes from a specific time:**

```sh
❯ rjot today
❯ rjot week
❯ rjot on 2025-05-01..2025-05-31
```

**5. Compile notes into a summary:**
Add the `--compile` flag to any time-based view to get a single Markdown summary.

```sh
❯ rjot week --compile > weekly-summary.md
```

### Managing specific notes

These commands target a specific note within the active notebook.

**1. Show the full content of a note:**

```sh
# By ID prefix
❯ rjot show 2025-06-08-1345

# By recency (the most recent note)
❯ rjot show --last
```

**2. Edit a note:**

```sh
# Edit the 3rd most recent note
❯ rjot edit --last=3
```

**3. Delete a note:**
This command will ask for confirmation unless you use the `--force` flag.

```sh
# Delete a note by ID prefix, with a confirmation prompt
❯ rjot delete 2025-06-08-1345
```

### Managing tasks

Many jots are simple to-do lists. `rjot` provides a quick way to create tasks and get a high-level overview of all pending items.

**1. Create a task:**
Use the `task` subcommand (or its aliases `todo` and `t`) to quickly create a new jot formatted as a Markdown task.

```sh
❯ rjot task 'Set up the new database schema'
❯ rjot todo 'Write unit tests for the auth service'
```

This creates a new note with the content `- [ ] Set up the new database schema`.

**2. View all incomplete tasks:**
Use the `--tasks` flag with the `list` command to see a list of all jots that contain one or more pending tasks.

```sh
❯ rjot list --tasks
```

### Pinning and unpinning notes/jots

Pinning is a great way to keep important notes from getting buried in your timeline.

**1. Pin a note:**
You can target a note by its ID or by its recency.

```sh
# Pin a specific jot
❯ rjot pin 2025-07-09-105000

# Pin the last jot you created
❯ rjot pin --last
```

**2. View all pinned notes:**
Use the `--pinned` flag with the `list` command.

```sh
❯ rjot list --pinned
```

**3. Unpin a note:**
When a note is no longer critical, you can unpin it.

```sh
❯ rjot unpin 2025-07-09-105000
```

### Managing tags

Use the `tag` subcommand to modify tags on an existing note in the active notebook.

**1. Add tags to a note:**

```sh
# Add 'rust' and 'idea' to the last jot
❯ rjot tag add --last=1 rust,idea
```

**2. Remove tags from a note:**

```sh
# Remove the 'idea' tag from a specific jot
❯ rjot tag rm -p 2025-06-09 idea
```

**3. Overwrite all tags on a note:**

```sh
# Replace all tags on the 2nd to last jot with 'archived'
❯ rjot tag set --last=2 archived
```

### Utility commands

Get info about your setup:

```sh
# Show storage paths and the active notebook
❯ rjot info --paths

# Show note, tag, and task statistics for the active notebook
❯ rjot info --stats

# Show combined stats for ALL notebooks
❯ rjot info --stats --all
```

### Git integration (optional)

`rjot` offers a convenient, built-in way to version control your notes. The git repository is initialized at the `rjot` root, meaning a single repo tracks all of your notebooks.

#### One-time setup

1.  **Initialize `rjot` with Git:**

    ```sh
    ❯ rjot init --git
    ```

2.  **Create a private remote repository:**
    Go to GitHub (or another Git provider) and create a new, empty **private** repository.

3.  **Link the remote:**
    Navigate into your `rjot` directory (`rjot info --paths` will show you where) and add the remote.

    ```sh
    # Example for GitHub over SSH
    ❯ git remote add origin git@github.com:YOUR_USERNAME/my-journal.git
    ```

#### The `sync` command

Once set up, `rjot sync` will automatically stage, commit, and push changes from all notebooks.

```sh
❯ rjot sync
```

### Encryption (optional)

For maximum privacy, you can enable transparent, on-disk encryption for all notebooks. The encryption keys are stored globally in your `rjot` root directory.

**One-time setup:**

```sh
❯ rjot init --encrypt
```

**IMPORTANT:** You must back up the `identity.txt` file somewhere safe. If you lose it, your notes cannot be recovered.

**Turning off encryption:**
The `decrypt` command will permanently decrypt all notes in all notebooks.

```sh
❯ rjot decrypt
```

## Configuration

### File storage location

`rjot` respects platform conventions and the `$RJOT_DIR` environment variable for all its data. By default, your journal is stored in the following locations:

* **macOS:** `~/Users/<YourUsername>/Library/Application Support/rjot/`
* **Linux:** `~/.config/rjot/`
* **Windows:** `C:\Users\<YourUsername>\AppData\Roaming\rjot\`

Within that root directory, your notes are organized in the `notebooks/` subdirectory.

### Templates

You can create custom templates for new notes by placing Markdown files in the `templates/` subdirectory inside your `rjot` root folder (e.g., `~/.config/rjot/templates/`). `rjot` supports one variable, `{{date}}`, which will be replaced with the current timestamp when the note is created.

## Contributing

This project is open source and contributions are welcome\! Please feel free to open an issue or submit a pull request.

## License

This project is licensed under the **MIT License**.