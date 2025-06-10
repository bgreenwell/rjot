# rjot <img src="assets/logo-color.png" align="right" height="120" alt="rjot logo" />

A minimalist, command-line jotting utility that's fast, private, git-friendly, and written in Rust.

[![Build Status](https://github.com/bgreenwell/rjot/actions/workflows/rust.yml/badge.svg)](https://github.com/bgreenwell/rjot/actions)
[![Crates.io](https://img.shields.io/crates/v/rjot.svg?label=crates.io)](https://crates.io/crates/rjot)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

---

## The vision

`rjot` is a tool for capturing thoughts at the speed of typing. It's built on a few core principles:

* **CLI-first, not CLI-only**: The terminal is the most powerful and frictionless interface for capturing text. `rjot` is designed to be a first-class citizen of your command line.
* **Plain text is sacred**: Your data is just a folder of Markdown files. It will always be readable, editable, and portable with or without `rjot`. No proprietary formats, no databases, no lock-in.
* **You own your data**: `rjot` will never push you to a proprietary sync service. It's designed from the ground up to empower you with control over your own data.

This project aims to be the perfect, minimalist companion for developers, writers, and anyone who lives in the terminal.

## Features

* **Instant capture**: Jot down a thought instantly from the command line.
* **Editor integration**: Use `rjot new` to open your favorite editor (`$EDITOR`) for longer-form entries with template support.
* **Powerful search & filtering**: Full-text search, tag-based filtering, and time-based views (`today`, `week`, `on <date>`, or `on <date-from>...<date-to>`).
* **Note management**: Easily `show`, `edit`, `tag`, or `delete` any note using a unique ID prefix or its recency (`--last` or `--last=3`).
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

Your command-line shell (like Bash or Zsh) can interpret special characters like `!` or expand variables like `$USER` inside double quotes (`"`). This can cause unexpected behavior.

**The best practice is to always use single quotes (`'`) for your messages.** This tells the shell to treat every character literally.

```sh
# GOOD: This works perfectly.
❯ rjot 'This is a great idea!'

# BAD: This will probably fail!
❯ rjot "This is a great idea!"

# OK (with escaping): If you must use double quotes, escape special characters.
❯ rjot "Note for user: $USER. This is a great idea\!"
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

### Managing tags

Use the `tag` subcommand to modify tags on an existing note without opening an editor.

**Important:** The `tag` command requires you to target a note with either `--last=<N>` or `-p <ID_PREFIX>`.

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

**1. Get info about your setup:**
```sh
# Show storage paths
❯ rjot info --paths

# Show note and tag statistics
❯ rjot info --stats
```

### Git integration (optional)

`rjot` offers a convenient, built-in way to version control and synchronize your notes using Git. This is an optional feature for users who are comfortable with Git concepts. You can always manage the repository manually by running `git` commands inside your `rjot` data directory.

#### One-time setup

1.  **Initialize `rjot` with Git:**
    This command prepares your `rjot` data directory and initializes a Git repository inside it.
    ```sh
    ❯ rjot init --git
    ```

2.  **Create a private remote repository:**
    Go to GitHub (or another Git provider) and create a new, empty **private** repository (e.g., named `my-journal`). Do not add a README or license file.

3.  **Link the remote:**
    Navigate into your `rjot` directory (`rjot info --paths` will show you where) and add the remote you just created.
    ```sh
    # Example for GitHub over SSH
    ❯ git remote add origin git@github.com:YOUR_USERNAME/my-journal.git

    # Example for GitHub over HTTPS
    ❯ git remote add origin [https://github.com/YOUR_USERNAME/my-journal.git](https://github.com/YOUR_USERNAME/my-journal.git)
    ```

#### The `sync` command

Once set up, you can use the `sync` command from anywhere to save your work.
```sh
❯ rjot sync
```
This single command will automatically:
1.  Stage all your new and modified notes (`git add .`).
2.  Create a commit with a timestamped message.
3.  Push the changes to your `origin` remote on your current branch.

#### How authentication works

The `rjot sync` command is designed to be secure and work automatically with common Git authentication methods. It will try the following, in order:

1.  **HTTPS (Personal Access Token):** If your remote URL uses `https://` and you have a `GITHUB_TOKEN` environment variable set, it will use that token for authentication.
2.  **SSH Agent:** If your remote URL uses `ssh://` (or `git@`), it will first try to authenticate using your system's SSH agent.
3.  **Default SSH Keys:** If the SSH agent fails, it will look for your default SSH key files (e.g., `~/.ssh/id_rsa`).
4.  **Git Credential Helper:** As a final fallback, it will try to use Git's configured credential helper.


## Configuration

### File storage location

`rjot` respects platform conventions. By default, notes are stored in the `entries` sub-folder of:

* **macOS:** `~/Library/Application Support/rjot/`
* **Linux:** `~/.config/rjot/`
* **Windows:** `C:\Users\<YourUsername>\AppData\Roaming\rjot\`

You can always override this by setting the `$RJOT_DIR` environment variable.

### Templates

Create custom templates in the `templates` subdirectory of your `rjot` root folder (e.g., `~/.config/rjot/templates/`). `rjot` supports one variable: `{{date}}`, which will be replaced with the current timestamp.

### Pro-tip: create a personal alias

For faster, more ergonomic use, you can create a personal alias for `rjot`. This allows you to type a short command like `jd` instead of `rjot`.

Here’s how to set it up for your specific operating system:

#### For macOS and Linux (zsh or bash)

1.  **Identify your shell's config file:**
    * If you're on a modern Mac, you are likely using `zsh`, so the file is `~/.zshrc`.
    * On most Linux distributions, you are likely using `bash`, so the file is `~/.bashrc`.

2.  **Add the alias to the file:**
    Run the following command in your terminal.

    ```sh
    # For zsh (macOS) or bash (Linux)
    echo "alias jd='rjot'" >> ~/.zshrc # Or ~/.bashrc
    ```

3.  **Apply the changes:**
    For the changes to take effect, either close and reopen your terminal, or run `source ~/.zshrc` (or `source ~/.bashrc`).

#### For Windows (PowerShell)

1.  **Check for (or create) a profile file:**

    ```powershell
    if (!(Test-Path $PROFILE)) { New-Item -Path $PROFILE -Type File -Force }
    ```

2.  **Add the alias to your profile:**

    ```powershell
    Add-Content -Path $PROFILE -Value "function jd { rjot.exe @args }"
    ```

3.  **Apply the changes:**
    Close and reopen your PowerShell window, or run `. $PROFILE`.

## Roadmap

`rjot` is under active development. The next major step is to implement the "power features":

* [ ] **Git integration**: `rjot init --git` to turn your journal into a version-controlled repository, and `rjot sync` to automate commits and sync with a remote.
* [ ] **Encryption**: `rjot init --encrypt` to enable transparent, on-disk file encryption using `age` for ultimate privacy.
* [ ] **Reminders**: `rjot remind` to set system-level notifications (e.g., `rjot remind me in 1 hour to...`).

## Contributing

This project is open source and contributions are welcome! Please feel free to open an issue or submit a pull request.

## License

This project is licensed under the **MIT License**.