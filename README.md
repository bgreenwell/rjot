# jot <img src="assets/logo.png" align="right" height="120" alt="jot logo" />

A minimalist, command-line journal that's fast, private, and git-friendly.

[![Crates.io](https://img.shields.io/crates/v/jot-cli.svg?label=crates.io)](https://crates.io/crates/jot-cli)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## The Vision

`jot` is a tool for capturing thoughts at the speed of typing. It's built on a few core principles:

* **CLI-First, Not CLI-Only**: The terminal is the most powerful and frictionless interface for capturing text. `jot` is designed to be a first-class citizen of your command line.
* **Plain Text is Sacred**: Your data is just a folder of Markdown files. It will always be readable, editable, and portable with or without `jot`. No proprietary formats, no databases, no lock-in.
* **You Own Your Data, You Own Your Sync**: `jot` will never push you to a proprietary sync service. It's designed from the ground up to leverage robust, existing tools like `git` for versioning and synchronization.

This project aims to be the perfect, minimalist companion for developers, writers, and anyone who lives in the terminal.

## Current Features

* **Instant Capture**: Jot down a thought instantly from the command line.
* **Editor Integration**: Use `jot new` to open your favorite editor (`$EDITOR`) for longer-form entries.
* **List Recent Notes**: Quickly view your most recent jots with `jot list`.
* **Search and Tagging**: Full-text search (`jot find`) and metadata filtering (`jot tags`).
* **Standard Directories**: Follows platform-specific conventions for data storage.
* **Configurable**: Respects standard environment variables like `$EDITOR` and `$JOT_DIR`.

## Installation

Currently, you must build from source.

```sh
# Clone the repository
git clone https://github.com/bgreenwell/jot.git
cd jot

# Build and install the binary
cargo install --path .
```
Once published, it will be available via `cargo install jot-cli`.

### Where Notes Are Stored

`jot` respects platform-specific conventions to avoid cluttering your home directory. By default, notes are stored in the `entries` sub-folder of:

* **macOS:** `~/Library/Application Support/jot/`
* **Linux:** `~/.config/jot/`
* **Windows:** `C:\Users\<YourUsername>\AppData\Roaming\jot\`

You can always override this default location by setting the `$JOT_DIR` environment variable.

## Usage

### Create a quick note
```sh
❯ jot 'This is a quick thought I want to save!'
```

### Create a tagged, one-liner note
```sh
❯ jot 'A great idea for the project' --tags idea,project
```

### Create a longer note using Vim (or your `$EDITOR`)
```sh
❯ jot new
```

### List your recent notes
```sh
❯ jot list
```

### Find notes by content or tags
```sh
❯ jot find "secret project"
❯ jot tags idea cli
```

### A Note on shells and quotes

Your command-line shell (like Bash, Zsh, etc.) can interpret special characters like `!` even when they are inside double quotes. This can cause your commands to fail or hang unexpectedly.

For example, this command will likely fail in most shells:
```sh
# This will probably hang or error out!
❯ jot "This is a great idea!"
```

The shell tries to interpret `!"` as a command from your history.

**The best practice is to use single quotes (`'`) for your messages.** Single quotes tell the shell to treat every character literally.

```sh
# This works perfectly!
❯ jot 'This is a great idea!'
```

If you must use double quotes (e.g., to expand a variable), you can escape the special character with a backslash (`\`).
```sh
❯ jot "This is a great idea\! It uses the $USER variable."
```

## Roadmap

`jot` is under active development. Here is the plan for upcoming features:

-   [ ] **Templates & Time-Based Views**
    -   [ ] Custom templates for `jot new` (e.g., `jot new --template meeting`).
    -   [ ] `jot today`: View all notes from the current day.
    -   [ ] `jot this-week`: View all notes from the current week.
    -   [ ] `jot <date> --compile`: Compile notes from a time range into a single summary file.
-   [ ] **Power Features**
    -   [ ] `jot init --git`: Native `git` integration for versioning and sync.
    -   [ ] `jot init --encrypt`: Transparent, on-disk file encryption using `age`.
    -   [ ] `jot remind`: Set system-level reminders (e.g., `jot remind me in 1 hour to...`).

## Contributing

This project is open source and contributions are welcome! If you'd like to help, please feel free to open an issue to discuss a bug or a new feature, or check the [Roadmap](#roadmap) for ideas to work on.

## License

This project is licensed under the **MIT License**.