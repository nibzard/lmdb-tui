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
- [ ] **hi** Initialize Cargo binary crate `lmdb-tui`
- [ ] **hi** CI workflow with clippy, fmt, test and build
- [ ] **mid** Add dependencies: `crossterm`, `ratatui`, `heed`, `clap`, `tokio`
- [ ] **mid** Basic `--help` and `--version` output

### Phase 1 – Core View
- [ ] **hi** FR-01: open existing LMDB environment and list databases
- [ ] **hi** FR-02: display keys and values in scrollable list
- [ ] **mid** FR-04: support read-only sessions
- [ ] **mid** App state reducer and navigation stack
- [ ] **mid** Unit tests for env open/list operations

### Phase 2 – CRUD & Transactions
- [ ] **hi** FR-03: CRUD operations inside read-write txn
- [ ] **mid** FR-06: transaction management commands
- [ ] **mid** FR-11: undo/redo stack
- [ ] **mid** DB service layer (`db::*` modules)
- [ ] **mid** Integration tests covering CRUD flows

### Phase 3 – Query Engine
- [ ] **mid** FR-05: rich query modes (prefix, range, regex, JSONPath)
- [ ] **mid** Implement `db::query` module with decoders
- [ ] **mid** Snapshot tests for query UI

### Phase 4 – Visuals and Stats
- [ ] **mid** FR-07: environment and DB statistics panes
- [ ] **lo** FR-08: bookmarks and jump-to-key history
- [ ] **mid** Background job queue for statistics (tokio tasks)
- [ ] **mid** Benchmarks for scanning 1M keys/sec

### Phase 5 – Export & Import
- [ ] **mid** FR-09: export/import in JSON or CSV
- [ ] **mid** Command progress indicators
- [ ] **lo** Validate values as JSON before export

### Phase 6 – Polish & Docs
- [ ] **mid** FR-10: configurable keybindings and themes
- [ ] **lo** FR-12: embedded help screen with searchable palette
- [ ] **mid** Packaging scripts: cross-build, Homebrew/Scoop manifests
- [ ] **mid** Usage examples and screenshots in README
- [ ] **mid** Document config format in `docs/`

### Future Enhancements
- [ ] **lo** remote mode via agent process
- [ ] **lo** gRPC server for automation
- [ ] **lo** WebAssembly build
- [ ] **lo** plugin API for custom decoders

## Module Implementation
- [ ] **hi** `app` main loop and state reducer
- [ ] **hi** `ui` layouts and widgets using ratatui
- [ ] **hi** `db::env` open/close env and query stats
- [ ] **hi** `db::txn` safe wrapper over heed txns
- [ ] **mid** `db::query` searching and decoding
- [ ] **mid** `commands` CRUD, export/import, undo stack
- [ ] **mid** `jobs` async workers and channels
- [ ] **mid** `config` load/save YAML/TOML settings
- [ ] **lo** `util` helpers (hex/utf-8, formatting)
- [ ] **mid** `errors` define `AppError` via `thiserror`

## Non-Functional Goals
- [ ] **mid** Keep RSS under 200 MB via streaming
- [ ] **hi** Maintain 60 fps while scanning 1M keys/sec
- [ ] **mid** Colour-blind-friendly palette, honour `$NO_COLOR`
- [ ] **mid** Audit crates to ensure no network I/O

