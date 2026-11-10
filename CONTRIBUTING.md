# Contributing to Cloister

Thanks for helping. This guide covers setup, conventions and what a good pull request looks like.

## Setup

Follow the prerequisites in the [README](README.md#prerequisites), then:

```bash
pnpm install
pnpm tauri dev
```

## Before you open a pull request

Run the same checks as CI:

```bash
pnpm format:check && pnpm lint && pnpm check && pnpm test
cd src-tauri && cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test
```

## Conventions

**General**

- Keep files small and focused. If a file grows past roughly 250 lines, it probably has two jobs.
- Every module and component starts with a short comment explaining its purpose.
- Prefer clear names over comments; use comments to explain _why_, not _what_.

**Rust**

- Pure logic goes in `services/` or `features/` with unit tests. Code that touches Tauri goes in `runtime/` or
  `commands/`.
- Return `AppResult<T>`; never `unwrap()` outside tests.
- Never hold two `AppState` locks at the same time.
- New commands must be added to `build.rs`, `lib.rs` (`generate_handler!`) and a capability.

**Frontend**

- Svelte 5 runes only. Components read from `app` (the store) and call its methods.
- Only `lib/api/backend.ts` calls `invoke`.
- Use design tokens from `styles/tokens.css`; no raw colours in components.
- Every interactive element is a real `<button>`, `<a>` or form control with an accessible name.
- Animate only `transform` and `opacity`, under 200 ms, and respect reduced motion.

**Commits**

Use [Conventional Commits](https://www.conventionalcommits.org/): `feat:`, `fix:`, `docs:`, `refactor:`,
`test:`, `chore:`, with a scope where it helps (`fix(lock): …`). Keep the subject under 72 characters.

## Adding a service

See [docs/recipes.md](docs/recipes.md). Recipe pull requests are the easiest way to start.

## Reporting bugs

Open an issue with:

- your system and its version (for example Windows 11 24H2, macOS 15.1 or Ubuntu 24.04),
- the Cloister version (Settings › About),
- the service involved, what you expected and what happened,
- the relevant lines from the log, if there are any.

Logs are in:

| System  | Folder                                     |
| ------- | ------------------------------------------ |
| Windows | `%LOCALAPPDATA%\app.cloister.desktop\logs` |
| macOS   | `~/Library/Logs/app.cloister.desktop`      |
| Linux   | `~/.local/share/app.cloister.desktop/logs` |

Logs never contain message text, but they do name your services and can include page addresses. Read them
before posting.

For security problems, follow [SECURITY.md](SECURITY.md) instead of opening a public issue.
