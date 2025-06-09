# rjot <img src="assets/logo-color.png" align="right" height="120" alt="rjot logo" />

A minimalist, command-line jotting utility that's fast, private, and git-friendly.

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
* **Editor integration**: Use `rjot new` to open your favorite editor (`$EDITOR`) for longer-form entries with template support.
* **Powerful search & filtering**: Full-text search, tag-based filtering, and time-based views (`today`, `week`, `on <date>`).
* **Note management**: Easily `show`, `edit`, or `delete` any note using a unique ID prefix or its recency (`--last`).
* **Standard & configurable**: Follows platform-specific conventions for data storage and respects standard environment variables.

## Installation

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

---

## Usage guide

### A note on shells and quotes

Your command-line shell (like Bash or Zsh) can interpret special characters like `!` even inside double quotes (`"`). This can cause commands to fail or hang.

**The best practice is to always use single quotes (`'`) for your messages.** This tells the shell to treat every character literally.
```sh
# GOOD: This works perfectly
❯ rjot 'This is a great idea!'

# BAD: This will probably fail!
❯ rjot "This is a great idea!"
```

### Creating notes

**1. Jot down a quick note (the default action):**
```sh
❯ rjot 'This is a quick thought I want to save.'
```

**2. Create a tagged, one-liner note:**
The `--tags` (or `-t`) flag accepts space-separated or comma-separated values.
```sh
❯ rjot 'A great idea for the project' --tags project rust
# or
❯ rjot 'Another idea' -t project,rust
```

**3. Create a longer note in your editor:**
```sh
# This opens your default $EDITOR
❯ rjot new

# Use a custom template for structured notes
❯ rjot new --template meeting.md
```

### Viewing and filtering notes

**1. List the 10 most recent notes:**
```sh
❯ rjot list
```

**2. Full-text search of all notes:**
```sh
❯ rjot find 'productivity'
```

**3. Filter by one or more tags:**
```sh
# Find notes with the 'project' tag
❯ rjot tags project

# Find notes with BOTH 'rust' and 'cli' tags
❯ rjot tags rust,cli
```

**4. View notes from a specific time:**
```sh
❯ rjot today
❯ rjot yesterday
❯ rjot week

# View notes from a specific date or range
❯ rjot on 2025-05-20
❯ rjot on 2025-05-01..2025-05-31
```

**5. Compile notes into a summary:**
Add the `--compile` flag to any time-based view to get a single Markdown summary.
```sh
❯ rjot week --compile > weekly-summary.md
```

### Managing specific notes

The `show`, `edit`, and `delete` commands allow you to target a specific note in two ways: by its ID prefix or by its recency.

**1. Show the full content of a note:**
```sh
# By ID prefix
❯ rjot show 2025-06-08-1345

# By recency (the most recent note)
❯ rjot show --last
# or
❯ rjot show --last=1
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

# Delete the last note without a prompt
❯ rjot delete --last --force
```

### Utility commands

**1. Get info about your setup:**
```sh
# Show storage paths
❯ rjot info --paths

# Show note and tag statistics
❯ rjot info --stats
```

## Configuration

### File storage location

`rjot` respects platform conventions. By default, notes are stored in the `entries` sub-folder of:

* **macOS:** `~/Library/Application Support/rjot/`
* **Linux:** `~/.config/rjot/`
* **Windows:** `C:\Users\<YourUsername>\AppData\Roaming\rjot\`

You can always override this by setting the `$RJOT_DIR` environment variable.

### Templates

Create custom templates in the `templates` subdirectory of your `rjot` root folder (e.g., `~/.config/rjot/templates/`). `rjot` supports one variable: `{{date}}`, which will be replaced with the current timestamp.

## Roadmap

`rjot` is under active development. The next major step is to implement the "power features":

* [ ] **Git integration**: `rjot init --git` to turn your journal into a version-controlled repository, and `rjot sync` to automate commits and sync with a remote.
* [ ] **Encryption**: `rjot init --encrypt` to enable transparent, on-disk file encryption using `age` for ultimate privacy.
* [ ] **Reminders**: `rjot remind` to set system-level notifications (e.g., `rjot remind me in 1 hour to...`).

## Contributing

This project is open source and contributions are welcome! Please feel free to open an issue or submit a pull request.

## License

This project is licensed under the **MIT License**.