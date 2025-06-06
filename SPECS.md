Developer Specification – lmdb-tui

A Rust-based Terminal UI explorer & visualizer for LMDB environments

⸻

1. Overview

Goal Provide a fast, keyboard-centric terminal UI (TUI) tool—similar in spirit to lazydocker, lazygit, and jqp—that lets developers and operators open any LMDB environment, inspect its databases, query key/value pairs, and perform full CRUD operations with transactional safety.

Scope
	•	Single-binary CLI application (lmdb-tui)
	•	Works on Linux, macOS, Windows (POSIX-style terminals or Windows Terminal)
	•	Targets Rust stable ≥ 1.78, LMDB ≥ 0.9.31
	•	Licensed Apache-2.0

Audience Backend engineers, SREs, embedded developers, and hobbyists using LMDB.

⸻

2. Functional Requirements

ID	Description	Priority
FR-01	Open existing LMDB environment (via path) and list databases (aka named DBIs).	P0
FR-02	Display keys and values in a scrollable list with live preview.	P0
FR-03	Create, read, update, delete key/value pairs inside a read-write txn.	P0
FR-04	Support read-only sessions that guarantee no accidental writes.	P0
FR-05	Rich query modes: exact key, prefix, range, regex (on printable data), and value JSONPath filter.	P1
FR-06	Transaction management: begin, commit, abort; show pending ops indicator.	P1
FR-07	Visualization panes: env stats (page size, mapsize, readers), DB stats (entries, depth, pages), histograms of key/value byte lengths.	P1
FR-08	Bookmarks/favourites and jump-to-key history.	P2
FR-09	Export selected records or entire DB to JSON, CSV, or raw.mdb file; import JSON/CSV.	P2
FR-10	Configurable keybindings & colour themes; mouse support optional.	P2
FR-11	Undo/redo within a session (maintain operation stack until commit).	P2
FR-12	Embedded help screen with searchable command palette.	P2


⸻

3. Non-Functional Requirements
	•	Performance: must render 60 fps in a typical 120×30 terminal while scanning 1 M keys/sec on NVMe.
	•	Memory: keep ≤ 200 MB RSS when viewing multi-GB envs by streaming & pagination.
	•	Safety: never corrupt env—writes only via single top-level RW txn; on panic, abort txn.
	•	Concurrency: background read threads for statistics; UI remains responsive (inspired by lazydocker’s async model).
	•	Accessibility: colour-blind-friendly palette; obey $NO_COLOR.
	•	Security: no network I/O; no executing arbitrary code in values.

⸻

4. Technology Stack & Dependencies

Layer	Crates / Libs	Notes
Terminal backend	crossterm	Cross-platform, WinPTY/ConPTY friendly
TUI framework	ratatui (fork of tui-rs)	Maintained; flexible layout API
Async tasks	tokio (multi-thread)	Only for background jobs & channels
LMDB bindings	heed (safe zero-copy) or rkv or lmdb-rs	Evaluate benches; prefer heed for safety
CLI parsing	clap v4 derive	Flags, subcommands (e.g., stats, open); help displayed when run without args
Serialization	serde, serde_json, serde_yaml, csv	Export/import
Error handling	thiserror, anyhow	Rich context
Logging	tracing, tracing-subscriber	JSON and colour log output
Testing	rstest, insta, cargo-nextest	Snapshot UI tests
Profiling	criterion, flamegraph	―
CI	GitHub Actions	lint → clippy → fmt → test → build


⸻

5. High-Level Architecture

+---------------------------+
|        Terminal          |
+------------+--------------+
             |
             v
+------------+--------------+
|    ratatui  Renderer      |
+------------+--------------+
             |
             v
+------------+--------------+
|      App State (Redux-style)  <----+  Async tasks (stats, scans)
+------------+--------------+       |
             |                        |
             v                        |
+------------+--------------+        |
|  Database Service Layer   |<-------+
|  (Env mgr, Txn facade)    |
+------------+--------------+
             |
             v
+------------+--------------+
|        LMDB FFI           |
+---------------------------+

5.1 State Model

pub struct App {
    pub env: Option<EnvHandle>,
    pub db_tree: Vec<DbMeta>,
    pub view: ViewStack,      // navigation stack (à la lazygit)
    pub input: InputMode,
    pub command_log: Vec<Cmd>,
    pub pending_txn: Option<RwTxn>,
    pub background: JobQueue,
}

State updates travel through Actions dispatched from key handlers; reduced into App—keeping all UI deterministic and testable.

5.2 Concurrency Model
	•	UI thread: owns App and draws every ~16 ms.
	•	Worker runtime (tokio): spawns blocking tasks (spawn_blocking) for large scans, stats, exports. Communicates via mpsc channels: UiMsg, BgMsg.
	•	LMDB safety: each async worker opens its own read-only transaction (since readers are cheap & concurrent). Exactly one RW txn at a time (held in App.pending_txn).

⸻

6. UI/UX Design

6.1 Layout (default)

┌─ DB List ─────────────────┬──── Keys (selected DB) ───────────────┐
│ [•] main                 │ key_0001                              │
│ [ ] cache                │ key_0002                              │
│ [ ] meta                 │ ...                                   │
├───────────────────────────┴─ Value Preview ───────────────────────┤
│ {                                                                │
│   "user_id": 42,                                               │
│   "name": "Ada"                                              │
│ }                                                                │
└───────────────────────────────────────────────────────────────────┘
» F1 Help   F2 New   F3 Edit   F4 Delete   / Search   : Command

	•	Navigation ► arrow keys / hjkl, Tab cycle focus, q back / quit.
	•	Command palette (:) opens modal (like Vim) with fuzzy search over all commands.
	•	Popup modals for edit/insert use multi-line text boxes with JSON validation.

6.2 Keymap (default)

Key	Action
o	Open env path (file picker)
c	Create key/value
e	Edit selected value
d	Delete (prompts)
/	Incremental search in focused pane
Ctrl+s	Commit pending RW txn
Ctrl+z	Abort txn
g/G	Jump top/bottom
?	Toggle inline help

All bindings configurable via ~/.config/lmdb-tui/config.toml.

⸻

7. Detailed Module Breakdown

Crate/Module	Purpose
lmdb_tui (bin)	arg parsing → app::run()
app	Main loop, state reducer, dispatch
ui::*	Layouts, widgets, theming
db::env	Open/close env, stats query
db::txn	Wrapper over heed::RwTxn/RoTxn with lifetimes erased (owning)
db::query	Range, prefix, regex, decoder (JSON, MsgPack)
commands	CRUD, export/import, undo stack
jobs	Background workers, channels
config	Load/save YAML/TOML settings, keymap
util	Hex/UTF-8 converters, time, size formatting
errors	AppError enum, mapped to UI alerts
tests	Integration & snapshot tests


⸻

8. Error Handling & Recovery
	•	Wrap every LMDB call in Result<T, LmdbError>; convert with ? into AppError.
	•	On panic inside RW txn: catch-unwind at worker boundary, abort txn automatically.
	•	Display non-fatal errors in a bottom toast area (fades after 5 s).

⸻

9. LMDB Best Practices Applied
	1.	Single writer, many readers – enforced by design; UI prevents multiple active RW txns.
	2.	Environment flags – expose CLI --map-size, --no-sync, --writemap, but default to safe settings.
	3.	Zero-copy reads – value slices borrowed directly; convert to owned only when edited or exported.
	4.	Large pagesizes – detect hugepages > 4 KB and warn if env uses smaller page.

⸻

10. Testing Strategy
	•	Unit tests: CRUD ops on temp env (use tempfile); value decoders.
	•	Snapshot UI: render frames with ratatui-test and assert against „golden“ dumps.
	•	Integration End-to-End: spawn app in PTY, script keystrokes via rexpect to validate flows.
	•	GitHub Actions matrix: ubuntu-latest, macos-latest, windows-latest.

⸻

11. Performance & Benchmarking
	•	Bench query::range_scan() on synthetic envs (1 M, 10 M keys) using criterion.
	•	Use pprof-rs for flamegraphs; automated weekly bench on main branch.

⸻

12. Build, Packaging, Release
	1.	Static release builds (musl) via cross.
	2.	Homebrew formula and Scoop manifest.
	3.	Create GitHub Release with changelog & SHA256.
	4.	Optional Debian .deb via cargo deb.

⸻

13. Roadmap & Milestones

Phase	Duration	Deliverables
0. Init	1 wk	repo scaffold, CI, basic --help
1. Core View	3 wk	open env, list dbs, key view (read-only)
2. CRUD & Txn	4 wk	RW flow, undo stack, config file
3. Query Engine	2 wk	prefix/regex, search bar, JSON decode
4. Visuals	2 wk	stats pane, histograms via ratatui charts
5. Export/Import	2 wk	JSON/CSV, progress bar
6. Polish & Docs	2 wk	man page, README gifs, packaging


⸻

14. Future Enhancements
	•	Remote mode: connect to LMDB over SSH using an agent process.
	•	gRPC server: expose CRUD for automation.
	•	WebAssembly build: run in browser (wasm-tui experiments).
	•	Plugin API: custom decoders (e.g., Protocol Buffers), value visualizers.

⸻

Inspired by the snappy UX of lazygit/lazydocker and the JSON-first philosophy of jqp, lmdb-tui aims to make navigating million-key data stores as fluid as browsing code.
