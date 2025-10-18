# Repository Guidelines

## Project Structure & Module Organization
Source code lives in `src/`, with `src/main.rs` implementing the Clap-powered CLI entrypoint. Add new features as modules under `src/` and expose them via `mod` statements near the top of `main.rs` or a dedicated `lib.rs` if the binary grows. Integration tests belong in a future `tests/` directory, while build outputs remain in `target/` and should stay untracked.

## Build, Test, and Development Commands
- `cargo check` quickly validates the code without producing binaries.
- `cargo build --release` emits an optimized executable in `target/release/ugh`.
- `cargo run -- --board demo --description "sample"` runs the CLI with example flags; keep the double dash to forward arguments.
- `cargo fmt` formats sources using rustfmt; run before committing.
- `cargo clippy --all-targets --all-features` surfaces common Rust antipatterns.

## Coding Style & Naming Conventions
Follow standard Rust style: four-space indentation, snake_case for functions and variables, CamelCase for types, and screaming_snake_case for constants. Keep public APIs documented with Rustdoc comments. When adding Clap parameters, mirror the flag name (e.g., `--board`) to a snake_case struct field (`board`) for clarity. Run `cargo fmt` and `cargo clippy` locally; CI assumes a clean lint pass.

## Testing Guidelines
Use Rust’s built-in test framework. Prefer colocated unit tests under a `#[cfg(test)] mod tests` block in the relevant module, and create integration tests in `tests/` once the surface area expands. Name tests after the behavior under scrutiny (e.g., `parses_board_flag`). Achieve line coverage for new logic by exercising both happy paths and error handling via argument parsing.

## Commit & Pull Request Guidelines
The repository has no commit history yet, so adopt Conventional Commits (e.g., `feat: add board validation`) to keep logs searchable. Each pull request should explain the change, list manual or automated test runs, and link to any tracking issues. Include CLI usage examples or screenshots when altering user-facing behavior to simplify review.

## Agent Notes
Avoid mutating files under `target/`. If you add dependencies, update `Cargo.toml` and run `cargo check` to refresh `Cargo.lock`. Document any non-obvious design decisions in PR descriptions so future agents can onboard quickly.
