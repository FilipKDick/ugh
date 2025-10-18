# ugh — Multi-Agent Developer CLI

`ugh` is a productivity-focused command-line assistant that helps engineers move from local changes to fully prepared Jira work items in a single hop. It inspects your git workspace, asks an LLM for a concise summary, creates a Jira ticket, and checks out the correctly-named branch so you can start committing immediately.

> **Quick install**
>
> ```bash
> curl -fsSL https://raw.githubusercontent.com/FilipKDick/ugh/master/install.sh | bash
> ```
>
> The installer auto-detects macOS (Intel/Apple Silicon) or Linux (x86_64), downloads the latest release artifact, and copies `ugh` into `/usr/local/bin` (falling back to `~/.local/bin` if necessary). To pin a version set `UGH_INSTALL_VERSION=v0.1.2`; to install from a fork set `UGH_INSTALL_REPO=yourorg/ugh` before running the command.

## Highlights
- **Git-aware ticket workflow** – summarizes uncommitted changes, calls Gemini 2.5 for a title/description, and spins up branches in the `type/JIRA-123/slug` format.
- **Pluggable services** – swap LLM providers and issue trackers via traits; only Gemini + Jira are implemented today.
- **Resilient UX** – network hiccups fall back to heuristic drafts, recent LLM results are cached locally, and a config wizard runs automatically if required details are missing.

## Installation
Build from source with Rust 1.79+:

```bash
cargo install --path .
```

This places the binary under `~/.cargo/bin`; add that directory to your `PATH` so the `ugh` command is available across repositories. For teammate installs, publish release binaries (see the installation guidance in `README`).

## Configuration
Run the guided setup once:

```bash
ugh config init
```

The wizard stores settings in `~/.config/ugh/config.json` (or the platform-equivalent). Set the following values when prompted:

- Jira base URL, email, API token, default project key, preferred issue type
- Gemini API key and model (defaults to `gemini-2.5-flash`)

Environment variables such as `UGH_JIRA_TOKEN` override the config file for CI or ad-hoc sessions. Draft responses are cached in `draft_cache.json` under the same config directory; delete it to force fresh LLM output.

## Usage
- `ugh ticket [--board PROJECT]` – Generates the Jira ticket and checks out the branch. On first run in a repo, the command will launch the config wizard if credentials are missing.
- `ugh config show` – Displays non-secret configuration values with masked tokens.

The workflow produces console output similar to:

```
Ticket DEMO-123 created. Branch ready: feature/DEMO-123/update-checkout-flow
View ticket: https://acme.atlassian.net/browse/DEMO-123
```

## Development
- Format and lint: `cargo fmt && cargo clippy --all-targets --all-features`
- Validate compilation: `cargo check`
- Tests live in the repo alongside modules; run `cargo test` as they are added.

We aim to minimize dependencies and keep logic modular (services under `src/services`, infrastructure clients under `src/infra`). Contributions are welcome—please document design decisions in PR descriptions so fellow agents can follow the architectural patterns.
