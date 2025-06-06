# Project Task List

This roadmap derives from [`SPECS.md`](SPECS.md) and lists the concrete steps
needed to build *lmdb-tui*. Use it to track progress and priorities.

## Style Guide
- `[ ]` task not started
- `[~]` task in progress (semaphore)
- `[x]` task complete
- Priorities: **hi**, **mid**, **lo**

## ✅ Critical Bug Fixes (Completed)

### Transaction Safety & Core Bugs
001. [x] **hi** Fix transaction commit bug in `list_databases()` - read txns should abort, not commit
002. [x] **hi** Add proper error handling in background job system (jobs/mod.rs:42-56)
003. [x] **hi** Fix potential panic in App::current_view() - handle empty view stack
004. [x] **hi** Add missing error context in database opening (env.rs:26)

### ✅ Configuration Implementation (Completed)
005. [x] **hi** Wire up loaded configuration to actual keybinding handling in app.rs
006. [x] **hi** Implement theme application in UI rendering
007. [x] **mid** Add validation for color parsing with better error messages
008. [x] **mid** Add validation for key parsing with better error messages

### Resource Management
009. [ ] **mid** Fix potential resource leak in pager child process handling
010. [ ] **mid** Review thread safety of environment cloning in job queue
011. [ ] **mid** Add proper cleanup for background threads on app exit

## 🔧 Core Feature Completion

### Query System Implementation
012. [ ] **hi** Complete query view implementation - currently minimal functionality
013. [ ] **mid** Wire up query engine to query view UI
014. [ ] **mid** Add query input handling and result display
015. [ ] **mid** Implement query result navigation and selection

### Export & Import (Phase 5)
016. [ ] **mid** FR-09: export/import in JSON or CSV
017. [ ] **mid** Command progress indicators
018. [ ] **lo** Validate values as JSON before export

### Polish & Configuration (Phase 6)
019. [~] **mid** FR-10: configurable keybindings and themes (partially done - needs wiring)
020. [ ] **mid** Usage examples and screenshots in README
021. [ ] **mid** Document config format in `docs/`

## ✅ Completed Phases

### Phase 0 – Bootstrap
001. [x] **hi** Initialize Cargo binary crate `lmdb-tui`
002. [x] **hi** CI workflow with clippy, fmt, test and build
003. [x] **mid** Add dependencies: `crossterm`, `ratatui`, `heed`, `clap`, `tokio`
004. [x] **mid** Basic `--help` and `--version` output

### Phase 1 – Core View
005. [x] **hi** FR-01: open existing LMDB environment and list databases
006. [x] **hi** FR-02: display keys and values in scrollable list
007. [x] **mid** FR-04: support read-only sessions
008. [x] **mid** App state reducer and navigation stack
009. [x] **mid** Unit tests for env open/list operations

### Phase 2 – CRUD & Transactions
010. [x] **hi** FR-03: CRUD operations inside read-write txn
011. [x] **mid** FR-06: transaction management commands
012. [x] **mid** FR-11: undo/redo stack
013. [x] **mid** DB service layer (`db::*` modules)
014. [x] **mid** Integration tests covering CRUD flows

### Phase 3 – Query Engine
015. [x] **mid** FR-05: rich query modes (prefix, range, regex, JSONPath)
016. [x] **mid** Implement `db::query` module with decoders
017. [x] **mid** Snapshot tests for query UI

### Phase 4 – Visuals and Stats
018. [x] **mid** FR-07: environment and DB statistics panes
019. [x] **lo** FR-08: bookmarks and jump-to-key history
020. [x] **mid** Background job queue for statistics (tokio tasks)
021. [x] **mid** Benchmarks for scanning 1M keys/sec
022. [x] **lo** FR-12: embedded help screen with searchable palette
023. [x] **mid** Packaging scripts: cross-build, Homebrew/Scoop manifests

## 🔮 Future Enhancements

### Advanced Features
030. [ ] **lo** Remote mode via agent process
031. [ ] **lo** gRPC server for automation
032. [ ] **lo** WebAssembly build
033. [ ] **lo** Plugin API for custom decoders

### Code Quality Improvements
034. [ ] **mid** Replace magic numbers with named constants (100 entries, 128 max dbs)
035. [ ] **mid** Standardize error types across codebase (anyhow vs custom)
036. [ ] **lo** Inline single-use helper functions (centered_rect)
037. [ ] **mid** Add more comprehensive integration tests

## 📋 Module Implementation Status

### Core Modules
034. [x] **hi** `app` main loop and state reducer
035. [x] **hi** `ui` layouts and widgets using ratatui
036. [x] **hi** `db::env` open/close env and query stats
037. [x] **hi** `db::txn` safe wrapper over heed txns
038. [x] **mid** `db::query` searching and decoding
039. [x] **mid** `commands` CRUD, export/import, undo stack
040. [x] **mid** `jobs` async workers and channels
041. [~] **mid** `config` load/save TOML settings (needs wiring to UI)
042. [x] **lo** `util` helpers (hex/utf-8, formatting)
043. [x] **mid** `errors` define `AppError` via `thiserror`

## 🎯 Non-Functional Goals

### Performance
044. [ ] **mid** Keep RSS under 200 MB via streaming
045. [ ] **hi** Maintain 60 fps while scanning 1M keys/sec

### Accessibility & UX
046. [ ] **mid** Colour-blind-friendly palette, honour `$NO_COLOR`
047. [ ] **mid** Audit crates to ensure no network I/O

### CLI UX (Completed)
048. [x] **mid** Update SPECS for help output when run without args; Clap shows examples
049. [x] **mid** Map common errors to exit codes and add tests
050. [x] **mid** Implement `--plain` and `--json` output modes
051. [x] **mid** Honour `$DEBUG` and `$PAGER` environment variables
052. [x] **mid** Add `-q` quiet and `--verbose` logging flags
053. [x] **mid** Include README link and example commands in `--help`

## 📅 Sprint Planning

### ✅ Completed Sprint (Critical Bug Fixes)
- ✅ Fixed transaction safety violation - LMDB read transactions now handled correctly
- ✅ Improved error handling in background job system with proper logging
- ✅ Enhanced view stack safety to prevent potential panics
- ✅ Added better error context for database operations

### ✅ Completed Sprint (Configuration Implementation)
- ✅ Wired up configuration system to actual UI keybinding handling
- ✅ Implemented theme application in UI rendering
- ✅ Added comprehensive validation for color and key parsing
- ✅ Extended keybindings to include help, query, and navigation

### Current Sprint Focus (Feature Completion)
- Complete query view implementation
- Fix resource management issues (pager cleanup, thread safety)
- Implement export/import functionality

### Next Sprint (Polish & Documentation)
- Performance optimizations
- Documentation improvements
- Additional integration tests
- Cross-platform compatibility testing

The configuration system is now fully implemented and integrated. Users can customize keybindings, themes, and colors through a TOML configuration file. The application has robust transaction handling, proper error management, and comprehensive configurability.