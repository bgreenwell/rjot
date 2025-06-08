# jot <img src="assets/logo.png" align="right" height="120" alt="jot logo" />

A minimalist, command-line journal that's fast, private, and git-friendly.

[![Crates.io](https://img.shields.io/crates/v/jot-cli.svg?label=crates.io)](https://crates.io/crates/jot-cli)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

---

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
* **Standard Directories**: Follows the XDG directory spec, storing notes in `~/.config/jot` by default.
* **Configurable**: Respects standard environment variables like `$EDITOR` and `$JOT_DIR`.

## Basic Usage

### Create a quick note
```sh
❯ jot 'This is a quick thought I want to save.'
Jotting down: "This is a quick thought I want to to save."
Successfully saved to /Users/bgreenwell/.config/jot/entries/2025-06-07-194510.md
```

### Create a longer note using Vim (or your `$EDITOR`)
```sh
# This will open vim
❯ jot new

# After you save and exit the editor:
Successfully saved to /Users/bgreenwell/.config/jot/entries/2025-06-07-194625.md
```

### List your recent notes
```sh
❯ jot list
Listing last 10 jots from "/Users/bgreenwell/.config/jot/entries":

ID                      FIRST LINE
----------------------  --------------------------------------------------
2025-06-07-194625       This is a longer entry I wrote in Vim.
2025-06-07-194510       This is a quick thought I want to save.
```

## Installation

Currently, you must build from source.

```sh
# Clone the repository
git clone https://github.com/YOUR_USERNAME/jot.git
cd jot

# Build and install the binary
cargo install --path .
```

Once published, it will be available via `cargo install jot-cli`.

## Roadmap

`jot` is under active development. Here is the plan for upcoming features:

-   [ ] **Organization & Search**
    -   [ ] `jot find <keyword>`: High-performance, full-text search of all notes.
    -   [ ] `jot tags <tag>`: Filter notes by tags defined in YAML frontmatter.
-   [ ] **Templates & Time-Based Views**
    -   [ ] Custom templates for `jot new` (e.g., `jot new --template meeting`).
    -   [ ] `jot today`: View all notes from the current day.
    * [ ] `jot this-week`: View all notes from the current week.
    -   [ ] `jot <date> --compile`: Compile notes from a time range into a single summary file.
-   [ ] **Power Features**
    -   [ ] `jot init --git`: Native `git` integration for versioning and sync.
    -   [ ] `jot init --encrypt`: Transparent, on-disk file encryption using `age`.
    -   [ ] `jot remind`: Set system-level reminders (e.g., `jot remind me in 1 hour to...`).
    -   [ ] `jot compile`: Collect notes by tag/keyword into a single Markdown file.

## Contributing

This project is open source and contributions are welcome! If you'd like to help, please feel free to open an issue to discuss a bug or a new feature, or check the [Roadmap](#roadmap) for ideas to work on.

## License

This project is licensed under the **MIT License**. See the `LICENSE` file for details.
