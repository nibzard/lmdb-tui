# Contributing to lmdb-tui

Thank you for considering contributing to **lmdb-tui**! This project welcomes
patches and suggestions via pull requests.

## Development Workflow

1. Ensure you are using **Rust 1.78** or newer.
2. Format your code and run lints:
   ```bash
   cargo fmt --all
   cargo clippy --all-targets --all-features -- -D warnings
   ```
3. Run the test suite before submitting a PR:
   ```bash
   cargo test
   ```
4. Use conventional commit messages (`feat:`, `fix:`, `docs:` and so on) and keep
   each commit focused.
5. Reference the relevant task ID from `Todo.md` in your pull request description.

## License

By contributing to this repository you agree to license your work under the
terms of the Apache-2.0 license as specified in `LICENSE`.
