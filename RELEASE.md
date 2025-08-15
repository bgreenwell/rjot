# Release Process Checklist

This document provides a comprehensive checklist for preparing and releasing new versions of rjot.

## Pre-Release Preparation

### 📋 **Version Planning**

- [ ] **Determine release type** (major, minor, patch)
  - **Major** (x.0.0): Breaking changes, major new features
  - **Minor** (0.x.0): New features, backwards compatible
  - **Patch** (0.0.x): Bug fixes, minor improvements
- [ ] **Update version number** in `Cargo.toml`
- [ ] **Choose release date** and coordinate with any announcements

### 📝 **Documentation Updates**

- [ ] **Update CHANGELOG.md**
  - [ ] Move unreleased items to new version section
  - [ ] Add release date
  - [ ] Ensure all significant changes are documented
  - [ ] Group changes by type: Added, Changed, Fixed, Deprecated, Removed, Security
  - [ ] Include issue/PR references where applicable
- [ ] **Review and update README.md**
  - [ ] Verify installation instructions are current
  - [ ] Update any version-specific examples
  - [ ] Ensure feature descriptions match current functionality
  - [ ] Check that screenshots/examples reflect current UI
- [ ] **Update inline documentation**
  - [ ] Review `cargo doc` output for completeness
  - [ ] Ensure all public APIs have proper documentation
  - [ ] Update any version-specific doc comments

### 🧪 **Quality Assurance**

- [ ] **Code Quality Checks**
  - [ ] Run `cargo fmt --check` - formatting
  - [ ] Run `cargo clippy -- -D warnings` - linting  
  - [ ] Run `cargo doc --no-deps` - documentation builds
  - [ ] Run `cargo audit` - security vulnerabilities
- [ ] **Comprehensive Testing**
  - [ ] Run `cargo test` - all unit tests pass
  - [ ] Run integration tests in clean environment
  - [ ] Test on all supported platforms (Linux, macOS, Windows)
  - [ ] Test with minimum supported Rust version (MSRV)
  - [ ] Manual smoke testing of core features
- [ ] **Build Verification**
  - [ ] Run `cargo build --release` - release builds cleanly
  - [ ] Test release binary functionality
  - [ ] Verify binary size is reasonable

### 🔍 **Feature Verification**

- [ ] **Core Functionality**
  - [ ] Basic jot creation and listing
  - [ ] Notebook operations (create, switch, list)
  - [ ] Search and filtering (find, tags, date ranges)
  - [ ] File operations (edit, show, delete)
- [ ] **Advanced Features**
  - [ ] Interactive shell with autocompletion
  - [ ] Template system with variables
  - [ ] Task management (create, list tasks)
  - [ ] Import/export functionality
  - [ ] Encryption/decryption workflows
  - [ ] Git integration (if enabled)
- [ ] **Cross-Platform Testing**
  - [ ] Test on Linux (Ubuntu/similar)
  - [ ] Test on macOS (Intel and Apple Silicon if possible)
  - [ ] Test on Windows
  - [ ] Verify fuzzy search works on non-Windows platforms

### 🔧 **Dependencies & Security**

- [ ] **Dependency Review**
  - [ ] Run `cargo update` to get latest compatible versions
  - [ ] Review dependency changes with `cargo tree --duplicates`
  - [ ] Check for deprecated dependencies
  - [ ] Verify no dev-dependencies leak into release
- [ ] **Security Audit**
  - [ ] Run `cargo audit` and resolve any issues
  - [ ] Review any new dependencies for security concerns
  - [ ] Ensure no secrets or sensitive data in repo
  - [ ] Verify encryption functionality works correctly

## Release Execution

### 🏗️ **Pre-Release Build**

- [ ] **Clean Build Environment**
  - [ ] `cargo clean` to remove build artifacts
  - [ ] Fresh clone in clean directory (optional but recommended)
- [ ] **Final Integration Test**
  - [ ] Run full test suite one final time
  - [ ] Manual testing of release candidate
  - [ ] Verify all CI checks pass on main branch

### 📦 **Release Creation**

- [ ] **Git Preparation**
  - [ ] Ensure all changes are committed and pushed
  - [ ] Main branch is in clean, releasable state
  - [ ] All CI checks are passing
- [ ] **Tag Creation**
  - [ ] Create annotated git tag: `git tag -a v<VERSION> -m "Release v<VERSION>"`
  - [ ] Push tag: `git push origin v<VERSION>`
  - [ ] Verify release workflow triggers and completes successfully
- [ ] **Automated Release Verification**
  - [ ] Check GitHub Actions release workflow completes
  - [ ] Verify all platform binaries are built and uploaded
  - [ ] Confirm crates.io publication succeeds
  - [ ] Review generated GitHub release draft

### 📢 **Publication**

- [ ] **GitHub Release**
  - [ ] Review auto-generated release notes
  - [ ] Add any additional context or highlights
  - [ ] Attach any additional assets if needed
  - [ ] Publish the release (undraft)
- [ ] **Package Distribution**
  - [ ] Verify package appears on crates.io
  - [ ] Test installation: `cargo install rjot`
  - [ ] Check package metadata is correct
- [ ] **Documentation**
  - [ ] Ensure docs.rs builds successfully
  - [ ] Review generated documentation for accuracy

## Post-Release

### 📊 **Monitoring**

- [ ] **Release Verification** (within 24 hours)
  - [ ] Monitor download/install statistics
  - [ ] Watch for user-reported issues
  - [ ] Check CI/CD pipeline health
- [ ] **Community Response**
  - [ ] Monitor GitHub issues for release-related bugs
  - [ ] Check discussions/social media for feedback
  - [ ] Be responsive to early adopter concerns

### 🔄 **Prepare for Next Cycle**

- [ ] **Repository Maintenance**
  - [ ] Create new "Unreleased" section in CHANGELOG.md
  - [ ] Update version in Cargo.toml to next development version (if following semantic versioning strictly)
  - [ ] Close any completed milestones
  - [ ] Create milestone for next version
- [ ] **Retrospective**
  - [ ] Document any lessons learned from this release
  - [ ] Note any process improvements for next time
  - [ ] Update this RELEASE.md if needed

## Emergency Procedures

### 🚨 **Hotfix Releases**

If a critical bug is discovered post-release:

- [ ] **Assessment**
  - [ ] Evaluate severity and impact
  - [ ] Determine if hotfix release is warranted
  - [ ] Document the issue thoroughly
- [ ] **Rapid Response**
  - [ ] Create hotfix branch from release tag
  - [ ] Apply minimal fix with focused testing
  - [ ] Skip non-critical checklist items
  - [ ] Release with patch version bump
  - [ ] Clearly communicate the hotfix to users

### 📋 **Rollback Plan**

If major issues are discovered:

- [ ] **Assessment**
  - [ ] Document the issue and impact
  - [ ] Decide if rollback is necessary
- [ ] **Rollback Actions**
  - [ ] Mark GitHub release as pre-release or draft
  - [ ] Consider yanking crates.io version if severe
  - [ ] Communicate clearly to users
  - [ ] Plan corrective release

## Required Secrets & Access

Ensure you have access to:

- [ ] **GitHub**: Repository admin access for releases
- [ ] **Crates.io**: Publishing permissions (`CARGO_REGISTRY_TOKEN`)
- [ ] **CI/CD**: Access to GitHub Actions and secrets

## Version-Specific Notes

### Current Release (v0.2.0 example)
- New CI/CD pipeline - verify all platforms build
- ASCII logo changes - test visual consistency
- Enhanced shell UX - verify exit instructions work

### Common Gotchas
- **Windows support**: Fuzzy search (`skim`) is disabled on Windows
- **Cross-compilation**: ARM64 builds require proper toolchain setup  
- **Encryption**: Test with both new and existing encrypted notebooks
- **Migration**: Verify legacy `entries/` to `notebooks/default/` migration

---

## Quick Reference Commands

```bash
# Quality checks
cargo fmt --check
cargo clippy -- -D warnings  
cargo test
cargo build --release
cargo audit

# Release process
git tag -a v0.2.0 -m "Release v0.2.0"
git push origin v0.2.0

# Post-release verification
cargo install rjot
rjot --version
```

## Additional Resources

- [Semantic Versioning](https://semver.org/)
- [Keep a Changelog](https://keepachangelog.com/)
- [Cargo Book - Publishing](https://doc.rust-lang.org/cargo/reference/publishing.html)
- [GitHub Releases Guide](https://docs.github.com/en/repositories/releasing-projects-on-github)