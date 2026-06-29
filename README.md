# Nixy

> A next-generation, Nix-native AI coding agent.

Nixy treats your codebase **and** its development environment as a single,
immutable, mathematical function. Instead of guessing what tools live on your
host machine, Nixy reads the project's `flake.nix` and gains a 100% accurate,
declarative manifest of every compiler, runtime, and system package available.

The environment *is* the context.

---

## Why Nixy?

Traditional coding agents poll the host OS, hallucinate tool versions, and run
commands in whatever environment they happen to land in. Nixy is different in
four ways:

### 1. The Environment *Is* the Context
By reading `flake.nix` / `flake.lock`, Nixy instantly knows the exact versions
of every tool available. No polling, no guessing.

### 2. Radical Sandboxing via `--pure`
Every terminal command and test execution is wrapped in an isolated dev shell:

```bash
nix develop .# --pure --command <command>
```

Host environment variables are stripped. If a tool isn't declared in the
project flake, Nixy can't use it. The *"works on my machine"* hallucination is
eliminated by construction.

### 3. Self-Healing Environments
When a project needs a tricky native dependency (`ImageMagick`, a specific C++
toolchain, …), Nixy mutates `flake.nix` directly, runs a dry-build, reads
evaluation errors, and fixes its own configuration. You get a working codebase
**and** a perfectly reproducible environment.

### 4. Non-Invasive Orchestration
Nixy does not replace Cargo, npm, Poetry, or Go modules. Nix acts strictly as
the guardrail layer that hands those native tools the exact system libraries
they need. Introduced to a non-Nix project, Nixy bootstraps itself by scanning
`package.json` / `requirements.txt` / etc. and generating a flawless `flake.nix`.

---

## Architecture

```
                  ┌──────────────────────────────┐
                  │         Nixy Core            │
                  └──────────────┬───────────────┘
                                 │
           ┌─────────────────────┼─────────────────────┐
           ▼                     ▼                     ▼
┌────────────────────┐ ┌────────────────────┐ ┌────────────────────┐
│   Workspace Layer  │ │  Environment Layer │ │   Execution Layer  │
│                    │ │                    │ │                    │
│ • Edits source code│ │ • Mutates flakes   │ │ • Evaluates builds │
│ • Tracks git diffs │ │ • Updates locks    │ │ • Runs tests in    │
│                    │ │                    │ │   pure dev shells  │
└────────────────────┘ └────────────────────┘ └────────────────────┘
```

Nixy shifts the AI coding agent meta from **"fixing broken code in a broken
environment"** to **"executing perfect code in a deterministic environment."**

---

## Development

Nixy is itself a Nix-native Rust project.

### Prerequisites
- [Nix](https://nixos.org/) with flakes enabled
- (optional) [direnv](https://direnv.net/) — `use flake` is already configured in `.envrc`

### Enter the dev shell
```bash
nix develop .#
# or, with direnv installed:
direnv allow
```

### Build
```bash
cargo build
# or build via Nix:
nix build .#
```

### Run
```bash
cargo run
```

### Format
```bash
cargo fmt
nix fmt   # formats Nix files
```

---

## Project Layout

```
nixy/
├── flake.nix           # Nix flake: dev shell, build, formatter
├── Cargo.toml          # Rust manifest
├── src/
│   ├── main.rs         # Entry point
│   ├── app.rs          # TUI event loop & layout
│   └── components/     # UI components (logo, input, status)
└── .envrc              # direnv: use flake
```

## License

TBD