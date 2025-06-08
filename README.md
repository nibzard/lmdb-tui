# lmdb-tui

**lmdb-tui** is a terminal user interface for exploring [LMDB](https://symas.com/lmdb/) databases. It targets Rust stable `1.78` and runs on Linux, macOS and Windows.

The project is currently in early development. The [Todo.md](Todo.md) file tracks tasks derived from the [specification](SPECS.md).

## Features (planned)

- **Open environments** and list named databases (FR-01).
- **Browse records** with a scrollable key/value view (FR-02).
- **Safe CRUD operations** inside a single write transaction (FR-03).
- **Read‑only sessions** to avoid accidental writes (FR-04).
- **Rich queries**: prefix, range, regex and JSONPath filters (FR-05).
- **Statistics panes** showing environment and DB stats (FR-07).
- **Export/Import** databases to JSON or CSV (FR-09).
- **Configurable key bindings** and themes (FR-10).
- **Embedded help screen** with searchable command palette (FR-12).

See `SPECS.md` for detailed requirements and future ideas.

## Installation & Usage

### Prerequisites

**macOS users** need:
- **Rust 1.78+**: Install via [Rustup](https://rustup.rs/) or Homebrew
- **Xcode Command Line Tools**: `xcode-select --install`

### Building from Source

1. **Install Rust** (if not already installed):
   ```bash
   # Using Rustup (recommended)
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source ~/.cargo/env
   
   # Or using Homebrew
   brew install rust
   ```

2. **Clone and build**:
   ```bash
   git clone https://github.com/nibzard/lmdb-tui.git
   cd lmdb-tui
   cargo build --release
   ```

3. **Install globally** (optional):
   ```bash
   cargo install --path .
   # Or add target/release/lmdb-tui to your PATH
   ```

### Basic Usage

**View help and options**:
```bash
lmdb-tui --help
```

**Open an LMDB environment** (interactive TUI mode):
```bash
lmdb-tui /path/to/lmdb/environment
```

**List databases in plain text**:
```bash
lmdb-tui --plain /path/to/lmdb/environment
```

**List databases as JSON**:
```bash
lmdb-tui --json /path/to/lmdb/environment
```

**Open read-only** (safe mode):
```bash
lmdb-tui --read-only /path/to/lmdb/environment
```

### Keybindings (Default)

| Key | Action |
|-----|--------|
| `q` | Quit application |
| `?` | Toggle help screen |
| `/` | Enter query mode |
| `↑/↓` | Navigate databases |
| `Esc` | Exit current view |

### Examples

**Explore a Bitcoin Core chainstate**:
```bash
# Bitcoin Core stores LMDB data in ~/.bitcoin/chainstate
lmdb-tui --read-only ~/.bitcoin/chainstate
```

**Browse Lightning Network data**:
```bash
# LND stores data in LMDB format
lmdb-tui ~/.lnd/data/graph/mainnet
```

**Quick database inspection**:
```bash
# List databases without opening TUI
lmdb-tui --plain /path/to/env | head -10
```

## Configuration

### Configuration File

`lmdb-tui` loads configuration from `~/.config/lmdb-tui/config.toml` on startup. If the file doesn't exist, defaults are used.

### Example Configuration

```toml
[keybindings]
quit = "q"          # Quit application
up = "k"            # Navigate up (Vim-style)
down = "j"          # Navigate down (Vim-style)
help = "?"          # Toggle help screen
query = "/"         # Enter query mode

[theme]
selected_fg = "White"   # Selected item text color
selected_bg = "Blue"    # Selected item background color
```

### Supported Keys

**Navigation**: `up`, `down`, `left`, `right`, `home`, `end`, `pageup`, `pagedown`  
**Control**: `enter`, `space`, `tab`, `backspace`, `delete`, `esc`  
**Characters**: Any single character (e.g., `"q"`, `"x"`, `"/"`)

### Supported Colors

**Basic colors**: `black`, `red`, `green`, `yellow`, `blue`, `magenta`, `cyan`, `white`  
**Extended**: `gray`/`grey`, `reset`  
**Case insensitive**: `Red`, `RED`, `red` all work

### macOS Configuration Path

The configuration file location follows XDG standards:
- **Default**: `~/.config/lmdb-tui/config.toml`
- **Custom**: Set `XDG_CONFIG_HOME` environment variable

```bash
# Create config directory
mkdir -p ~/.config/lmdb-tui

# Create example config
cat > ~/.config/lmdb-tui/config.toml << 'EOF'
[keybindings]
quit = "q"
up = "k"
down = "j"
help = "?"
query = "/"

[theme]
selected_fg = "Black"
selected_bg = "Yellow"
EOF
```

## Packaging

Release binaries can be produced with the helper scripts in `scripts/`.
To build static artifacts for Linux, macOS and Windows targets, run:

```bash
scripts/cross_build.sh
```

Homebrew and Scoop manifests can be generated with:

```bash
python scripts/generate_manifests.py
```

The generated files are written to the `dist/` directory.

### WebAssembly Build

An experimental WebAssembly build can be produced with [`wasm-pack`](https://rustwasm.github.io/wasm-pack/):

```bash
scripts/wasm_build.sh
```

This generates a `pkg/` directory with `lmdb_tui.js` and `lmdb_tui_bg.wasm` which can be served by a static web server.

## Troubleshooting

### macOS Issues

**"xcrun: error: invalid active developer path"**:
```bash
# Install Xcode Command Line Tools
xcode-select --install
```

**Permission denied when accessing LMDB files**:
```bash
# Use read-only mode for system databases
lmdb-tui --read-only /path/to/protected/db

# Or fix permissions (if you own the files)
chmod -R u+r /path/to/lmdb/environment
```

**Terminal colors not working**:
```bash
# Ensure your terminal supports 256 colors
echo $TERM
# Should show: xterm-256color or similar

# If not, set it:
export TERM=xterm-256color
```

**Cargo build fails**:
```bash
# Update Rust toolchain
rustup update stable

# Clear cargo cache and rebuild
cargo clean
cargo build --release
```

### Environment Variables

**Useful environment variables for macOS**:
```bash
# Enable debug logging
export DEBUG=1
lmdb-tui /path/to/env

# Use custom pager
export PAGER=less
lmdb-tui --help

# Custom config location
export XDG_CONFIG_HOME=/path/to/config
lmdb-tui /path/to/env
```

## Documentation

Read the full documentation at <https://lmdb.nibzard.com>.
For tools that consume our documentation automatically, see
[llms.txt](https://lmdb.nibzard.com/llms.txt) for a brief index.

## Contributing

Contributions are welcome! Please check `Todo.md` for open tasks. Before submitting a pull request, run:

```bash
cargo fmt --all
cargo clippy --all-targets --all-features
cargo test
```

## License

This project is licensed under the Apache 2.0 license. See [`LICENSE`](LICENSE) for details.
