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

See `SPECS.md` for detailed requirements and future ideas.

## Usage

Install Rust and build the project:

```bash
cargo build --release
```

Run `lmdb-tui` with `--help` to see available options:

```bash
lmdb-tui --help
```

To view the current version:

```bash
lmdb-tui --version
```

## Contributing

Contributions are welcome! Please check `Todo.md` for open tasks. Before submitting a pull request, run:

```bash
cargo fmt --all
cargo clippy --all-targets --all-features
cargo test
```

## License

This project is licensed under the Apache 2.0 license. See [`LICENSE`](LICENSE) for details.
