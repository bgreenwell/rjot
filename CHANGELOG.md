### `CHANGELOG.md`

# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased] - 2025-07-08

This release focuses on a major organizational improvement: multi-notebook support. It also includes quality-of-life improvements, robust error handling, and a comprehensive test suite to solidify the core user experience.

### Added

* **Enhanced template system** (addresses [#10](https://github.com/bgreenwell/rjot/issues/10)): The templating engine now supports built-in variables (`{{date}}`, `{{branch}}`, `{{project_dir}}`, `{{uuid}}`) and custom command-line variables (`-v key=value`) for creating dynamic, context-aware notes.

* **Import/export functionality** (addresses [#15](https://github.com/bgreenwell/rjot/issues/15)):
    * New `export` command to save an entire notebook to a `.zip` archive or a single `.json` file.
    * New `import` command to restore a notebook from a `.zip` or `.json` file, enabling backups and sharing.

  * **Global search** (addresses [#8](https://github.com/bgreenwell/rjot/issues/8)): The `find` command now includes an `--all` flag to search for keywords across every note in every notebook, with an updated output format to show which notebook a result belongs to.

  * **Task management** (addresses [#11](https://github.com/bgreenwell/rjot/issues/11)): `rjot` now has first-class support for task lists.
    * A new `task` command (with aliases `todo` and `t`) instantly creates a jot formatted as a Markdown task (e.g., `- [ ] My new task`).
    * The `list` command has a new `--tasks` flag to display only jots that contain incomplete tasks.
    * The `info --stats` command now includes a "Task Summary" section, showing a count of pending and completed tasks across the notebook.

  * **Jot pinning** (addresses [#9](https://github.com/bgreenwell/rjot/issues/9)): You can now pin important jots to keep them easily accessible.
    * The new `pin` and `unpin` subcommands allow you to toggle the pinned status of any jot.
    * A `pinned: true` attribute is added to the frontmatter of pinned notes.
    * The `list` command now includes a `--pinned` flag to show only pinned jots in the active notebook.

  * **Multi-notebook support** (addresses [#7](https://github.com/bgreenwell/rjot/issues/7)): `rjot` now supports organizing jots into separate notebooks.
      * A new `notebooks/` directory is created in the `rjot` root to store all notebooks as subdirectories.
      * Automatic, one-time migration for existing users, moving old `entries/` into a `notebooks/default/` directory.
      * All commands (`list`, `find`, `today`, etc.) are now scoped to an **active notebook**.
      * The active notebook is determined by the `RJOT_ACTIVE_NOTEBOOK` environment variable, or falls back to `default`.
  * A new `notebook` subcommand (alias `n`) for managing notebooks:
      * `rjot notebook new <NAME>`: Creates a new notebook.
      * `rjot notebook list`: Lists all available notebooks, highlighting the active one.
      * `rjot notebook use <NAME>`: Prints the shell command to switch the active notebook session.
      * `rjot notebook status`: Displays the currently active notebook.
  * A global `--notebook <NAME>` flag to execute a single command in a specific notebook without changing the active session.
  * The `info --stats` command now displays stats for the active notebook.
  * A new `--all` flag for `info --stats` to show combined statistics for all notebooks.
  * An extensive suite of integration tests for all notebook functionality, including error handling and edge cases.
  * The `decrypt` command now operates globally, decrypting all notes in all notebooks at once.

### Changed

  * The project has been refactored into multiple source files (`main.rs`, `cli.rs`, `commands.rs`, `helpers.rs`) for better organization and maintainability.
  * The `info --paths` command has been updated to display the full notebook path structure.
  * The logic for finding an editor for the `new` and `edit` commands was improved. It now gracefully falls back to common editors like `vim` or `nano` if the `$EDITOR` environment variable is not set.

### Fixed

  * A race condition in the test suite where notes created in rapid succession could have the same filename was fixed by adding a small delay.
  * The `init --git` command now creates a `.gitignore` that correctly tracks notebook files by default, only ignoring sensitive key files.
  * An "out of bounds" error was fixed where using `--last` with a number larger than the total number of notes would incorrectly target the first note; it now provides a clear error message.

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