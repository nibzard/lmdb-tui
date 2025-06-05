# AI Agent Guidelines

These rules help maintainers and automated agents contribute productively.

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

