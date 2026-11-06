<!-- BACKLOG.MD GUIDELINES START -->

<!-- backlog.md-instructions-version: 1.55.3 -->

<CRITICAL_INSTRUCTION>

## Backlog.md Workflow

This project uses Backlog.md for task and project management.

**At the beginning of each conversation in this project, run `backlog instructions overview` before answering or taking action. Re-read it only if you have not read it yet in the current conversation.**

Use the overview to decide whether to search, read, create, or update Backlog tasks.

Before task lifecycle actions, read the matching detailed guide:

- `backlog instructions task-creation` before creating or splitting tasks
- `backlog instructions task-execution` before planning, changing status or assignee, adding a plan or implementation notes, or implementing task work
- `backlog instructions task-finalization` before checking acceptance criteria, writing final summaries, or moving tasks to terminal statuses

Use `backlog <command> --help` before running unfamiliar commands. Help shows options, fields, and examples.

Do not edit Backlog task, draft, document, decision, or milestone markdown files directly. Use the `backlog` CLI so metadata, relationships, and history stay consistent.

</CRITICAL_INSTRUCTION>

<!-- BACKLOG.MD GUIDELINES END -->

# bloppen — agent guide

`bloppen` reverse-engineers the Danish **Børneloppen** Android app
(`dk.boerneloppen.boerneloppen`) into a Rust API library and CLI. Read
[`re/prd.md`](re/prd.md) before starting work; it is the source of truth for
scope, milestones, and current status.

## The `backlog` command

Backlog.md is the task source of truth. Its upstream binary is dynamically
linked and does not run natively on NixOS, so this repository wraps it:

```sh
./bin/backlog instructions overview
./bin/backlog task list
./bin/backlog task create "..."
```

`just backlog ...` is equivalent. Do not edit task, draft, document, decision,
or milestone markdown by hand; use the wrapper so metadata stays consistent.

## Non-negotiable rules

1. **One task per agent, sequentially.** Assign it to yourself, set it In
   Progress, implement it, check its acceptance criteria, and move it to Done.
   Do not pick up a second task in the same turn.
2. **Every tangent becomes a backlog task.** If you notice a bug, a missing
   endpoint, an interesting native library, or a better decompiler, create a
   task and continue with the current one. Do not chase it inline.
3. **Commit between tasks.** One logical change per commit, after the phase
   gate has passed.
4. **All reverse-engineering artifacts go under `re/`.** Decompiled output and
   extracted splits are generated and gitignored; findings and scripts are
   tracked.
5. **Never commit the APK, extracted splits, decompiled output, network
   captures, or secrets.** They are covered by `.gitignore`; do not defeat it
   with `git add -f`.
6. **Tokens, PII, and credentials go in `secrets/`** (gitignored). Captured
   API responses often contain personal data: anonymise before quoting them in
   tracked notes.
7. **Use the flake.** Run commands as `nix develop --command <tool>` or enter
   the shell with `direnv allow`. Do not assume host tools exist.
8. **Do not bypass access controls.** Authenticate as yourself, use your own
   account, and stay within the app's intended API. No credential stuffing and
   no scraping other users' data.
9. **Run the regression gate before committing.** `just fmt-check` now, and
   `just e2e` once the CLI exists.

## Repository layout

| Path | Purpose |
| --- | --- |
| `apk/` | Downloaded XAPK and extracted splits (gitignored) |
| `re/` | PRD, scripts, analysis notes, decompiled output (generated parts gitignored) |
| `nix/` | Dendritic flake-parts modules, auto-imported by `import-tree` |
| `backlog/` | Backlog.md tasks, drafts, documents, decisions |
| `bin/` | Repository-local helper scripts |
| `secrets/` | Tokens, captured responses, anything with PII (gitignored) |

## Commands

```sh
nix develop              # enter the devshell (or `direnv allow`)
just fmt                 # format Nix, Rust, Markdown, TOML, YAML, shell
just fmt-check           # CI formatting check
just apk-info            # package/version/SDKs from the extracted base APK
just backlog task list   # task source of truth
```

## Phase loop

The workflow is the nine-phase loop in the `android-app-reverser` skill:

1. Project setup (complete)
2. Extract and decompile
3. Static analysis
4. Live API validation
5. Rust library
6. CLI
7. Integration testing
8. Advanced features (opt-in; ask first)
9. Security and cleanup

Do not populate backlog tasks for a later phase until the current phase's gate
has passed and the user has confirmed advancing.
