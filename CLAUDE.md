# Trunk

Desktop Git GUI — Tauri 2 + Svelte 5 + Rust.

## Commands

```bash
just              # List all recipes
just dev          # Vite dev + Tauri watch
just build        # Production build
just check        # Run ALL checks (fmt, biome, svelte-check, clippy, cargo-test, vitest)
```

Run `just check` before every commit and push.

## Stack

- **Frontend:** Svelte 5 (runes: `$state`, `$derived`), Vite 6, TypeScript 5.6 strict, Tailwind CSS 4
- **Backend:** Tauri 2, git2 0.19 (libgit2), notify 7 (fs watcher), tokio 1
- **Frontend→Backend:** `invoke("command_name", args)` calls Rust `#[tauri::command]` fns
- **Paths:** `$lib` → `src/lib`, commands in `src-tauri/src/commands/`

## Rules

- Never inline colors — always use CSS custom properties from the theme
- Never fight layout with positioning hacks — use grid/flexbox so elements flow naturally
- All git operations go through git2 crate, no shelling out (except GIT_EDITOR for rebase/merge message editing)

## Get Shit Done (GSD)

This project uses GSD (`/gsd:*` slash commands) for planning and execution. All planning lives in `.planning/`.

### Navigation

| File | Purpose |
|------|---------|
| `.planning/STATE.md` | Current milestone, phase progress, where we stopped |
| `.planning/PROJECT.md` | Project definition, validated requirements, architecture decisions |
| `.planning/ROADMAP.md` | All phases with success criteria |
| `.planning/REQUIREMENTS.md` | Current milestone's numbered requirements |
| `.planning/RETROSPECTIVE.md` | Lessons learned across milestones |
| `.planning/phases/NN-name/` | Phase docs: CONTEXT, RESEARCH, PLANs, SUMMARYs, VERIFICATION |
| `.planning/milestones/` | Archived milestone docs |
| `.planning/todos/` | Tracked bugs and tasks (`pending/`, `done/`) |
| `.planning/debug/` | Open debugging notes |

### Key commands

- `/gsd:progress` — Check where we are (reads STATE.md)
- `/gsd:next` — What to do next
- `/gsd:plan-phase N` — Create plans for phase N
- `/gsd:execute-phase N` — Execute phase N's plans
- `/gsd:verify-work N` — Test phase N deliverables
- `/gsd:quick <task>` — Small self-contained task outside milestone phases
- `/gsd:do <intent>` — Routes freeform text to the right GSD command
- `/gsd:help` — Full command reference

### Workflow

```
new-project → [per phase: discuss → plan → execute → verify] → complete-milestone
```

### When resuming work

1. Read `.planning/STATE.md` for current position
2. Check `stopped_at` field to know exactly where we left off
3. Use `/gsd:progress` or `/gsd:next` to continue
