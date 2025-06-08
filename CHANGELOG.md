# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.1] - 2025-06-07

### Added

- Added a changelog to document project updates and changes.

### Fixed

- Corrected the installation instructions in the README.

---

## [0.1.0] - 2025-06-07

This is the initial public release of `jot` on crates.io. This version includes the core functionality for creating, finding, and organizing notes from the command line.

### Added

#### Core Note Creation
- Initial implementation of `jot "<message>"` to instantly create a note from a string argument.
- Added `jot new` subcommand to create long-form notes by launching the system's default editor (defined by `$EDITOR`).

#### Note Retrieval & Organization
- Added `jot list` subcommand to display a summary of the 10 most recent notes.
- Introduced support for YAML frontmatter (e.g., for tags) at the top of note files.
- Added `jot tags <tag>...` subcommand to filter and display notes that contain one or more specified tags in their frontmatter.
- Added `jot find <keyword>` subcommand for a case-insensitive, full-text search of the content of all notes.

#### Command-Line Experience
- Added `--tags` (and short-form `-t`) flag to allow adding tags directly when creating a one-liner note (e.g., `jot "My idea" -t idea`).
- Support for comma-separated values with the `--tags` flag (e.g., `-t idea,project`).

#### Configuration & File System
- Notes are stored in a dedicated `entries` directory inside a main `jot` folder.
- The `jot` folder respects platform-specific conventions by default, using the `dirs` crate:
  - **macOS:** `~/Library/Application Support/jot/`
  - **Linux:** `~/.config/jot/`
  - **Windows:** `%APPDATA%\jot\`
- Support for overriding the default storage location with the `$JOT_DIR` environment variable.

#### Project Scaffolding
- Initialized project with a `README.md`, MIT License, project logo, and installation instructions.
- Set up the package name and binary name in `Cargo.toml` for publishing to crates.io.

[unreleased]: https://github.com/YOUR_USERNAME/jot/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/YOUR_USERNAME/jot/releases/tag/v0.1.0
