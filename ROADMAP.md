# Roadmap

Nixy is early. The TUI shell exists (logo, input, status bar) but no agent
logic is wired up yet. This roadmap tracks the path from that shell to the
full Nix-native agent described in the README.

Status legend: `[ ]` planned, `[~]` in progress, `[x]` done.

---

## Phase 0 — TUI Foundations

The current terminal shell.

- [x] Ratatui + crossterm event loop
- [x] Logo, input, status components
- [x] Ctrl+C quit-nag, Ctrl+D quit
- [x] Configurable keybindings (`DEFAULT_APP_KEYBINDINGS`,
      `DEFAULT_EDITOR_KEYBINDINGS`) — no hardcoded key checks
- [x] Scrollback / message history buffer
- [x] Resizable panes (input vs. transcript)
- [x] Mouse wheel scrolling over transcript

## Phase 1 — Nix Introspection

Make the environment *the* context.

- [ ] Parse `flake.nix` / `flake.lock` into a structured manifest
      (packages, devShell inputs, tool versions)
- [ ] Detect project type (Cargo, npm, Poetry, Go modules, …) and surface
      the relevant toolchain from the flake
- [ ] Expose manifest to the agent as context (system prompt section)
- [ ] Bootstrap mode: for non-Nix projects, scan `package.json` /
      `requirements.txt` / etc. and generate a `flake.nix`

## Phase 2 — Sandboxed Execution

Radical isolation via `--pure`.

- [ ] Command runner wrapping `nix develop .# --pure --command <cmd>`
- [ ] Capture stdout / stderr / exit code streams into the TUI
- [ ] Timeout + cancellation support
- [ ] Allowlist of commands that may run outside the sandbox (git, etc.)
- [ ] Refuse to run anything not declared in the flake — surface the missing
      dependency as a self-healing action rather than a failure

## Phase 3 — Self-Healing Environments

- [ ] Mutate `flake.nix` to inject missing dependencies
- [ ] Dry-build evaluation: `nix build .# --dry-run`, parse errors
- [ ] Iterate on evaluation errors until the flake builds
- [ ] Refresh locks with project tooling (`cargo update -p`, `nix flake
      update`) inside the dev shell, never by hand
- [ ] Verify `nix build .#` succeeds before reporting environment ready

## Phase 4 — Workspace Layer

- [ ] Source file edits with structured apply/rollback
- [ ] Git diff tracking against worktree HEAD
- [ ] Multi-file refactor coordination
- [ ] Conflict-aware merging when multiple sessions touch the same repo

## Phase 5 — Agent Core

- [ ] Provider abstraction (local + remote models)
- [ ] Tool-calling protocol: read, edit, run-sandboxed, mutate-flake
- [ ] Conversation state + context window management
- [ ] Streaming responses into the TUI transcript
- [ ] Interrupt / resume mid-turn

## Phase 6 — Non-Invasive Orchestration

- [ ] Delegate to native build tools (cargo, npm, poetry, go) inside the
      pure dev shell; never replace them
- [ ] Test runner integration via the sandboxed command runner
- [ ] Lint / format passthrough with clippy, rustfmt, nixfmt
- [ ] Cross-project support: detect toolchain, route to the right native
      tool, hand it the exact system libraries from the flake

## Phase 7 — Hardening

- [ ] No `unwrap`/`expect` in non-test code; explicit error handling
- [ ] Audit every command path for host-environment leakage
- [ ] Reproducible agent runs: same flake + same prompt = same result
- [ ] Integration tests via tmux harness (see `AGENTS.md`)

---

## Non-Goals

- Replacing Cargo, npm, Poetry, or any native build tool.
- Supporting non-Nix environments as a first-class target. Nix is the
  guardrail layer; without it the safety guarantees do not hold.
- Global package installation. Ever.