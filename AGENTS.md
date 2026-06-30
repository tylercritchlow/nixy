<!-- Adapted from the pi.dev repository's AGENTS.md.
     Original author: Mario Zechner (badlogic) — github.com/earendil-works/pi-mono
     Licensed under the MIT License (see below).

MIT License

Copyright (c) 2025 Mario Zechner

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
-->

# AGENTS.md

## Project

Nixy is a Nix-native AI coding agent written in Rust (edition 2024).
The environment is defined declaratively in `flake.nix`; always use it.

## Conversational Style

- Keep answers short and concise.
- No emojis in commits, issues, PR comments, or code.
- No fluff or cheerful filler text. Technical prose only, be direct.
- Answer a question before making edits or running implementation commands.
- When responding to feedback, say whether you agree or disagree before
  describing what you changed.

## Code Quality

- Read files in full before wide-ranging changes, before editing files you
  have not fully inspected, and when asked to audit. Do not rely on search
  snippets for broad changes.
- Follow existing style in `src/` (ratatui + crossterm, no external UI deps).
- No comments in code unless explicitly requested.
- Inline single-line helpers that have only one call site.
- Avoid `unwrap`/`expect` in non-test code; prefer explicit error handling.
- Never hardcode key checks (e.g. `matches!(key, ... "ctrl+x")`). Add defaults
  to the relevant keybinding config so they stay configurable.
- Always ask before removing functionality or code that appears intentional.
- Do not preserve backward compatibility unless the user asks for it.
- Never downgrade code to fix errors from outdated deps; update the dep
  (in `flake.nix` / `Cargo.toml`) instead.

## Commands

- Enter dev shell: `nix develop .#` (or `direnv allow`).
- Build: `cargo build`  /  `nix build .#`.
- Run: `cargo run`.
- Lint: `cargo clippy -- -D warnings`.
- Format: `cargo fmt`  /  `nix fmt`.
- Test: `cargo test`.
- After code changes (not docs): run `cargo clippy -- -D warnings` and
  `cargo fmt` (full output, no tail). Fix all errors, warnings, and infos
  before committing.
- Never run commands that touch the host environment. Prefer
  `nix develop .# --pure --command <cmd>` for anything that needs tools.
- If a dependency is missing, fix `flake.nix` rather than installing globally.
- For ad-hoc scripts, `write` them to a temp file (e.g. `/tmp`), run, edit if
  needed, remove when done. Don't embed multi-line scripts in `bash` commands.
- Never commit unless the user asks.

## Dependencies and Lockfiles

- Treat `Cargo.lock` and `flake.lock` changes as reviewed code. Direct deps
  stay pinned to exact versions.
- Refresh locks with the project's own tooling (`cargo update -p <crate>`,
  `nix flake update`) inside the dev shell, not by hand.
- If `flake.nix` changes, verify with `nix build .#` (or `nix develop .#`)
  before committing.

## Git

Multiple sessions may be running in this cwd at the same time, each modifying
different files. Git operations that touch files outside your own changes will
stomp on other sessions' work.

Committing:

- Only commit files YOU changed in THIS session.
- Stage explicit paths (`git add <path1> <path2>`); never `git add -A` /
  `git add .`.
- Before committing, run `git status` and verify you are only staging your
  files.
- Message format: a single descriptive line, no conventional-commit prefix
  (`feat:`, `fix(scope):`, …) and no body.

Never run (destroys other agents' work or bypasses checks):

- `git reset --hard`, `git checkout .`, `git clean -fd`, `git stash`,
  `git add -A`, `git add .`, `git commit --no-verify`.

If rebase conflicts occur:

- Resolve conflicts only in files you modified.
- If a conflict is in a file you did not modify, abort and ask the user.
- Never force push.

Do not commit `target/`, `.direnv/`, `.rust-toolchain/`, or `.DS_Store`.

## Issues and PRs

When reviewing PRs:

- Do not run `gh pr checkout`, `git switch`, or otherwise move the worktree to
  the PR branch unless the user explicitly asks.
- Use `gh pr view`, `gh pr diff`, `gh api`, and local `git show`/`git diff`
  against fetched refs to inspect PR metadata, commits, and patches without
  changing branches.
- If you need PR file contents, use `git show <ref>:<path>` without switching
  branches.

When posting issue/PR comments:

- Write the comment to a temp file and post with `gh issue/pr comment
  --body-file` (never multi-line markdown via `--body`).
- Keep comments concise, technical, in the user's tone.
- End every AI-posted comment with the AI-generated disclaimer line specified
  by the originating prompt.

When closing issues via commit:

- Include `fixes #<number>` or `closes #<number>` in the message so merging
  auto-closes the issue. For multiple issues, repeat the keyword per issue
  (`closes #1, closes #2`); a shared keyword (`closes #1, #2`) only closes the
  first.

## Testing the TUI with tmux

Run the TUI in a controlled terminal (from the repo root):

```bash
tmux new-session -d -s nixy-test -x 80 -y 24
tmux send-keys -t nixy-test "cargo run" Enter
sleep 3 && tmux capture-pane -t nixy-test -p     # capture after startup
tmux send-keys -t nixy-test "your prompt here" Enter
tmux send-keys -t nixy-test Escape               # special keys (also C-c for ctrl+c)
tmux kill-session -t nixy-test
```

## User Override

If the user's instructions conflict with any rule in this document, ask for
explicit confirmation before overriding. Only then execute their instructions.
