# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased] - 2025-XY-XY

This release focuses on quality-of-life improvements, robust error handling, and a comprehensive test suite to solidify the core user experience before adding advanced features.

### Added

* A new `select` subcommand (alias `s`) that launches an interactive fuzzy finder, allowing users to quickly search and select a note.

* The `--encrypt` flag for the `init` command, which generates a new cryptographic key and enables transparent, on-disk encryption for all notes.

* A new `decrypt` subcommand to permanently decrypt all notes in the journal and remove the encryption keys.

* A new optional `<COUNT>` argument to the `list` subcommand to allow users to specify how many recent notes to display (e.g., `rjot list 20`).

* A new `init` subcommand to initialize the `rjot` directory.

*  The `--git` flag for the `init` command, which turns the `rjot` data directory into a local Git repository.

* A new `sync` subcommand that automatically stages all changes, creates a timestamped commit, and pushes to a remote repository named `origin`.

* The `sync` command now includes robust authentication logic that automatically handles both SSH and HTTPS (via a `GITHUB_TOKEN` environment variable) remotes.

* A new `tag` subcommand was introduced to manage metadata on existing notes. It includes `add`, `rm` (remove), and `set` actions to modify tags without opening an editor.

* The `--last=<N>` flag was added to `show`, `edit`, and `delete` commands, allowing users to easily target the Nth most recent note (e.g., `--last` for the last note, `--last=2` for the second to last).

* A comprehensive integration test suite was created in the `tests/` directory to cover all major commands and user workflows.

* A unit test was added to `src/main.rs` to validate the logic for generating ordinal suffixes (st, nd, rd, th).

* A GitHub Actions workflow was added to automatically run `cargo fmt`, `cargo clippy`, and `cargo test` on Linux, macOS, and Windows to ensure code quality and cross-platform compatibility.

* A "pro-tip" section was added to the `README.md` with instructions on how to create a personal shell alias for `rjot` on all major operating systems.

### Changed

* The project has been refactored into multiple source files (`main.rs`, `cli.rs`, `commands.rs`, `helpers.rs`) for better organization and maintainability.

* The logic for finding an editor for the `new` and `edit` commands was improved. It now gracefully falls back to common editors like `vim` or `nano` if the `$EDITOR` environment variable is not set.

* The `tag add`, `tag rm`, and `tag set` subcommands were updated to use a consistent `--id-prefix` (or `-p`) flag for targeting notes, improving clarity and preventing argument parsing errors.

### Fixed

* An "out of bounds" error was fixed where using `--last` with a number larger than the total number of notes would incorrectly target the first note; it now provides a clear error message.

* A bug in the integration tests was fixed where an incorrect number of arguments was passed to the `tag` command, causing a test failure.

* A cross-platform compatibility issue in the test suite was resolved. The mock editor script is now created as a `.bat` file on Windows, allowing all tests to pass.

* The `README.md` was updated to reflect the correct default storage path on macOS (`~/Library/Application Support/rjot`).

* A warning about shell character expansion (`!`) was added to the `README.md` to guide users on the best practice of using single quotes for notes.

## [0.1.0](https://github.com/bgreenwell/rjot/releases/tag/v0.1.0) - 2025-06-08

This is the initial public release of `rjot`. This version provides a comprehensive, file-based journaling system with a focus on a robust command-line experience.

### Added

#### Core note creation
- A default command to instantly create a note from a string argument (e.g., `rjot 'My new idea'`).
- The `new` subcommand to create long-form notes by launching the system's default editor (`$EDITOR`).
- The `--tags` (or `-t`) flag to add metadata when creating a note. It accepts both comma-separated and space-separated values.
- A template system for the `new` command, which uses a `default.md` template or a user-specified template via `rjot new --template <name>`.

#### Note retrieval and filtering
- The `list` subcommand to display a summary of the 10 most recent notes.
- Support for YAML frontmatter at the top of note files for metadata.
- The `find` subcommand for case-insensitive, full-text search of all note content.
- The `tags` subcommand to filter and display notes that contain one or more specified tags.
- Time-based filtering commands: `today`, `yesterday`, and `week`.
- The `on <date>` subcommand to filter notes for a specific date (`YYYY-MM-DD`) or a date range (`YYYY-MM-DD..YYYY-MM-DD`).
- The `--compile` flag for all time-based commands to generate a single summary document.

#### Note management
- The `show` subcommand to display the full content of a specific note in the terminal.
- The `edit` subcommand to open an existing note in the default editor.
- The `delete` subcommand (with `rm` alias) to safely remove a note, with a confirmation prompt.
- The `--force` flag for `delete` to bypass the confirmation prompt.
- The `--last=<N>` flag for `show`, `edit`, and `delete` to easily target the Nth most recent note (e.g., `--last` is the same as `--last=1`).

#### Utility and configuration
- The `info` subcommand with `--paths` to display storage locations and `--stats` to show total note count and tag frequency.
- Notes are stored in platform-specific standard locations (`~/Library/Application Support/rjot` on macOS, `~/.config/rjot` on Linux).
- Support for overriding the default storage location with the `$RJOT_DIR` environment variable.

#### Project health and documentation
- A full integration test suite using `assert_cmd` to verify all command functionality.
- A GitHub Actions workflow for Continuous Integration (CI) that checks formatting, runs Clippy, and executes the test suite on Linux, macOS, and Windows.
- A comprehensive `README.md` with usage examples and project details.
- A `LICENSE` file (MIT).