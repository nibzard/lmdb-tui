# Configuration

lmdb-tui looks for a user configuration file on startup. On Unix-like
systems the file lives at `~/.config/lmdb-tui/config.toml` (or
`config.yaml`). On Windows it resides under
`%APPDATA%\lmdb-tui\config.toml`.

The file may be written in **TOML** or **YAML**. The tables shown below
use TOML syntax but the keys are identical for YAML.

## General options

```toml
[general]
# Built-in theme: "light" or "dark"
theme = "light"
# Start in read-only mode by default
read_only = false
# Number of recent environments to remember
max_history = 5
```

## Key bindings

Actions can be bound to arbitrary keys. When omitted the defaults are
used.

```toml
[keybindings]
quit = "q"
move_up = "Up"
move_down = "Down"
commit_txn = "Ctrl+s"
abort_txn = "Ctrl+z"
search = "/"
```

Additional tables may be added in future versions for themes or plug-ins.

