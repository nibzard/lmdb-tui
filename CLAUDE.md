# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build Commands

```bash
# Development build
cargo build

# Release build
cargo build --release

# Run tests
cargo test

# Run with verbose test output
cargo test --verbose

# Format code
cargo fmt --all

# Lint (must pass before committing)
cargo clippy --all-targets --all-features -- -D warnings

# Cross-platform builds (creates dist/ artifacts)
./scripts/cross_build.sh

# Generate packaging manifests
python scripts/generate_manifests.py
```

## Architecture Overview

### State Management Pattern
The app uses a Redux-style architecture in `src/app.rs`:
- All UI state changes go through the `Action` enum
- State is immutable between updates
- View transitions are explicit (Main vs Query view)

### Database Layer Structure
The `src/db/` module provides safe LMDB operations:
- `env.rs` - Environment lifecycle management
- `kv.rs` - Generic key-value operations using `heed` codecs
- `query.rs` - Query engine supporting prefix, range, and regex patterns
- `txn.rs` - Transaction wrappers enforcing single-writer pattern
- `undo.rs` - Undo/redo stack management

### Async Jobs Pattern
`src/jobs.rs` implements a generic worker pattern:
- Workers run on tokio runtime without blocking UI
- Communication via message channels
- Used for stats calculation, exports, and background operations

### UI Layer
Built with `ratatui`, the UI code in `src/ui/` follows:
- Stateless rendering functions taking app state
- Event handling returns Actions for state updates
- Cross-platform key handling via crossterm

## Key Development Patterns

### Error Handling
- Use `anyhow::Result` for all fallible operations
- Add context with `.context()` for better error messages
- Exit codes: 1 for errors, 2 for not found

### LMDB Safety Rules
- Never hold transactions across await points
- Always use `heed` for type-safe operations
- Single writer enforced at environment level
- Zero-copy reads by default, convert only when needed

### Testing Approach
- Unit tests use temporary databases via `tempfile`
- Integration tests use `assert_cmd` for CLI testing
- Test both TUI and CLI output modes
- Mock terminal for UI snapshot tests

## Configuration

Config file locations:
- Unix: `~/.config/lmdb-tui/config.toml`
- Windows: `%APPDATA%\lmdb-tui\config.toml`

Configuration is loaded on startup and includes theme, keybindings, and behavior settings.

## Rust Best Practices
- Target **stable Rust 1.78** or newer.
- Prefer **explicit** types and lifetimes.
- Run `cargo fmt --all` and `cargo clippy --all-targets --all-features` before committing.
- Use `Result<T, anyhow::Error>` for application-level errors with context from `thiserror`.
- Write unit tests for each module and integration tests for end-to-end flows.

## Git Workflow
- Use **conventional commit** messages:
  - `feat:`, `fix:`, `docs:`, `chore:`, `refactor:` etc.
  - Subject line should be short; body explains the reasoning when necessary.
- Keep commits focused. Avoid mixing unrelated changes.

## Style Guide
- Follow `rustfmt` defaults for formatting.
- Document public items with Rustdoc comments.
- Name modules and files with `snake_case`; types with `CamelCase`.
- Use `clippy` suggestions unless there's a good reason not to.

## Pull Requests
- Ensure `cargo test` passes before raising a PR.
- Provide a short summary of changes and link to relevant tasks in `Todo.md`.

## Versioning
- Always use semver and every major piece of work needs to at least increment patch number