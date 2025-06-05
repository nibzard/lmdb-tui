# Project Task List

This roadmap derives from [`SPECS.md`](SPECS.md) and lists the concrete steps
needed to build *lmdb-tui*. Use it to track progress and priorities.

## Style Guide
- `[ ]` task not started
- `[~]` task in progress (semaphore)
- `[x]` task complete
- Priorities: **hi**, **mid**, **lo**

## Phased Tasks

### Phase 0 – Bootstrap
001. [x] **hi** Initialize Cargo binary crate `lmdb-tui`
002. [x] **hi** CI workflow with clippy, fmt, test and build
003. [x] **mid** Add dependencies: `crossterm`, `ratatui`, `heed`, `clap`, `tokio`
004. [x] **mid** Basic `--help` and `--version` output

### Phase 1 – Core View
005. [x] **hi** FR-01: open existing LMDB environment and list databases
006. [x] **hi** FR-02: display keys and values in scrollable list
007. [x] **mid** FR-04: support read-only sessions
008. [~] **mid** App state reducer and navigation stack
009. [x] **mid** Unit tests for env open/list operations

### Phase 2 – CRUD & Transactions
010. [x] **hi** FR-03: CRUD operations inside read-write txn
011. [x] **mid** FR-06: transaction management commands
012. [x] **mid** FR-11: undo/redo stack
013. [x] **mid** DB service layer (`db::*` modules)
014. [x] **mid** Integration tests covering CRUD flows

### Phase 3 – Query Engine
015. [ ] **mid** FR-05: rich query modes (prefix, range, regex, JSONPath)
016. [ ] **mid** Implement `db::query` module with decoders
017. [ ] **mid** Snapshot tests for query UI

### Phase 4 – Visuals and Stats
018. [ ] **mid** FR-07: environment and DB statistics panes
019. [ ] **lo** FR-08: bookmarks and jump-to-key history
020. [ ] **mid** Background job queue for statistics (tokio tasks)
021. [ ] **mid** Benchmarks for scanning 1M keys/sec

### Phase 5 – Export & Import
022. [ ] **mid** FR-09: export/import in JSON or CSV
023. [ ] **mid** Command progress indicators
024. [ ] **lo** Validate values as JSON before export

### Phase 6 – Polish & Docs
025. [ ] **mid** FR-10: configurable keybindings and themes
026. [ ] **lo** FR-12: embedded help screen with searchable palette
027. [ ] **mid** Packaging scripts: cross-build, Homebrew/Scoop manifests
028. [ ] **mid** Usage examples and screenshots in README
029. [ ] **mid** Document config format in `docs/`

### Future Enhancements
030. [ ] **lo** remote mode via agent process
031. [ ] **lo** gRPC server for automation
032. [ ] **lo** WebAssembly build
033. [ ] **lo** plugin API for custom decoders

## Module Implementation
034. [~] **hi** `app` main loop and state reducer
035. [~] **hi** `ui` layouts and widgets using ratatui
036. [x] **hi** `db::env` open/close env and query stats
037. [x] **hi** `db::txn` safe wrapper over heed txns
038. [ ] **mid** `db::query` searching and decoding
039. [ ] **mid** `commands` CRUD, export/import, undo stack
040. [ ] **mid** `jobs` async workers and channels
041. [ ] **mid** `config` load/save YAML/TOML settings
042. [ ] **lo** `util` helpers (hex/utf-8, formatting)
043. [ ] **mid** `errors` define `AppError` via `thiserror`

## Non-Functional Goals
044. [ ] **mid** Keep RSS under 200 MB via streaming
045. [ ] **hi** Maintain 60 fps while scanning 1M keys/sec
046. [ ] **mid** Colour-blind-friendly palette, honour `$NO_COLOR`
047. [ ] **mid** Audit crates to ensure no network I/O

### CLI UX Guidelines
048. [ ] **mid** Update SPECS for help output when run without args; Clap shows examples
049. [ ] **mid** Map common errors to exit codes and add tests
050. [ ] **mid** Implement `--plain` and `--json` output modes
051. [ ] **mid** Honour `$DEBUG` and `$PAGER` environment variables
052. [ ] **mid** Add `-q` quiet and `--verbose` logging flags
053. [ ] **mid** Include README link and example commands in `--help`
