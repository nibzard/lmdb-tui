# Project Task List - Team Assignment Structure

This roadmap derives from [`SPECS.md`](SPECS.md) and lists the concrete steps needed to build *lmdb-tui*. Tasks are organized by module/feature area to enable parallel development without merge conflicts.

## Style Guide
- `[ ]` task not started
- `[~]` task in progress (semaphore)
- `[x]` task complete
- **Owner**: Team member assigned to this area
- **Dependencies**: Other modules this depends on
- Priorities: **hi**, **mid**, **lo**

---

## Team 1: Resource Management & Background Jobs
**Owner**: [Unassigned]  
**Module Focus**: `src/jobs/`, thread safety, process management  
**Dependencies**: None (foundational)

### Tasks
009. [ ] **mid** Fix potential resource leak in pager child process handling
010. [ ] **mid** Review thread safety of environment cloning in job queue
011. [ ] **mid** Add proper cleanup for background threads on app exit
034. [ ] **mid** Replace magic numbers with named constants (100 entries, 128 max dbs)
044. [ ] **mid** Keep RSS under 200 MB via streaming
045. [ ] **hi** Maintain 60 fps while scanning 1M keys/sec

---

## Team 2: Query System Implementation
**Owner**: [Unassigned]  
**Module Focus**: `src/ui/query.rs`, `src/db/query.rs`  
**Dependencies**: Core DB layer (read-only)

### Tasks
012. [ ] **hi** Complete query view implementation - currently minimal functionality
013. [ ] **mid** Wire up query engine to query view UI
014. [ ] **mid** Add query input handling and result display
015. [ ] **mid** Implement query result navigation and selection

---

## Team 3: Export/Import Feature
**Owner**: [Unassigned]  
**Module Focus**: New module `src/export.rs`, CLI commands  
**Dependencies**: DB read operations, serialization

### Tasks
016. [ ] **mid** FR-09: export/import in JSON or CSV
017. [ ] **mid** Command progress indicators
018. [ ] **lo** Validate values as JSON before export

---

## Team 4: Documentation & Examples
**Owner**: [Unassigned]  
**Module Focus**: `docs/`, `README.md`, examples  
**Dependencies**: None (can work independently)

### Tasks
020. [ ] **mid** Usage examples and screenshots in README
021. [ ] **mid** Document config format in `docs/`
046. [ ] **mid** Colour-blind-friendly palette, honour `$NO_COLOR`
047. [ ] **mid** Audit crates to ensure no network I/O

---

## Team 5: Testing & Quality
**Owner**: [Unassigned]  
**Module Focus**: `tests/`, benchmarks, CI  
**Dependencies**: All features (for integration tests)

### Tasks
035. [ ] **mid** Standardize error types across codebase (anyhow vs custom)
036. [ ] **lo** Inline single-use helper functions (centered_rect)
037. [ ] **mid** Add more comprehensive integration tests

### Test Suite Reorganization & Refactoring
054. [ ] **hi** Reorganize test structure into unit/, integration/, performance/, ui/ directories
055. [ ] **hi** Migrate shell scripts from experiments/ to structured Rust integration tests
056. [ ] **mid** Expand UI testing coverage (currently only 2 UI tests for TUI-focused app)
057. [ ] **mid** Add comprehensive TUI interaction testing (keyboard navigation, screen transitions)
058. [ ] **mid** Implement property-based testing for query engine edge cases
059. [ ] **mid** Add configuration system testing (config file loading, validation, overrides)
060. [ ] **mid** Test background jobs and async operations
061. [ ] **mid** Add Unicode/binary data testing (non-UTF8 keys and values)
062. [ ] **mid** Test concurrent access scenarios (read-only during writes, multiple readers)
063. [ ] **mid** Add error recovery testing (corrupted databases, permission issues)
064. [ ] **mid** Create centralized test fixtures and builders (replace dynamic generation)
065. [ ] **lo** Establish performance baselines and regression testing
066. [ ] **lo** Add memory constraint and large dataset testing

---

## Future Enhancements (Backlog)
**Owner**: Future sprints  
**Module Focus**: New features

### Advanced Features
030. [ ] **lo** Remote mode via agent process
031. [ ] **lo** gRPC server for automation
032. [x] **lo** WebAssembly build
033. [ ] **lo** Plugin API for custom decoders

---

## ✅ Completed Work

### Critical Bug Fixes (Completed)
001. [x] **hi** Fix transaction commit bug in `list_databases()` - read txns should abort, not commit
002. [x] **hi** Add proper error handling in background job system (jobs/mod.rs:42-56)
003. [x] **hi** Fix potential panic in App::current_view() - handle empty view stack
004. [x] **hi** Add missing error context in database opening (env.rs:26)

### Configuration Implementation (Completed)
005. [x] **hi** Wire up loaded configuration to actual keybinding handling in app.rs
006. [x] **hi** Implement theme application in UI rendering
007. [x] **mid** Add validation for color parsing with better error messages
008. [x] **mid** Add validation for key parsing with better error messages
019. [x] **mid** FR-10: configurable keybindings and themes

### Core Features (Completed)
- [x] Phase 0: Bootstrap - project setup, CI, dependencies
- [x] Phase 1: Core View - open LMDB, list databases, display key-value pairs
- [x] Phase 2: CRUD & Transactions - all CRUD operations, transaction management, undo/redo
- [x] Phase 3: Query Engine - prefix, range, regex, JSONPath queries
- [x] Phase 4: Visuals and Stats - statistics, bookmarks, help screen, packaging

### Module Implementation Status (Completed)
034. [x] **hi** `app` main loop and state reducer
035. [x] **hi** `ui` layouts and widgets using ratatui
036. [x] **hi** `db::env` open/close env and query stats
037. [x] **hi** `db::txn` safe wrapper over heed txns
038. [x] **mid** `db::query` searching and decoding
039. [x] **mid** `commands` CRUD, export/import, undo stack
040. [x] **mid** `jobs` async workers and channels
041. [x] **mid** `config` load/save TOML settings (fully wired)
042. [x] **lo** `util` helpers (hex/utf-8, formatting)
043. [x] **mid** `errors` define `AppError` via `thiserror`

### CLI UX (Completed)
048. [x] **mid** Update SPECS for help output when run without args
049. [x] **mid** Map common errors to exit codes and add tests
050. [x] **mid** Implement `--plain` and `--json` output modes
051. [x] **mid** Honour `$DEBUG` and `$PAGER` environment variables
052. [x] **mid** Add `-q` quiet and `--verbose` logging flags
053. [x] **mid** Include README link and example commands in `--help`

---

## Development Guidelines

### To Prevent Merge Conflicts:
1. Each team works in their assigned module/directory
2. Avoid modifying `src/app.rs` unless coordinating with other teams
3. New features should be in new files when possible
4. Use feature flags for incomplete work
5. Communicate before modifying shared types in `src/errors.rs` or `src/util.rs`

### Coordination Points:
- **DB Layer** (`src/db/`): Read-only for Teams 2 & 3, coordinate changes
- **UI Framework** (`src/ui/mod.rs`): Shared by Teams 1 & 2
- **Main App State** (`src/app.rs`): Requires team sync for modifications
- **Config** (`src/config.rs`): Stable, minimal changes expected

### Pull Request Strategy:
1. Each team creates PRs for their module area
2. Tag PRs with team name: `[Team1]`, `[Team2]`, etc.
3. Merge order: Team 1 (foundational) → Teams 2-4 (features) → Team 5 (testing)
4. Use feature branches: `team1-resource-mgmt`, `team2-query-system`, etc.