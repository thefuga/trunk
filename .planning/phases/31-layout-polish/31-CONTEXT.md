# Phase 31: Layout Polish - Context

**Gathered:** 2026-03-15
**Status:** Ready for planning

<domain>
## Phase Boundary

Merge the window title bar and toolbar into a single unified bar using Tauri's `decorations: false` config. The result saves vertical space by eliminating the separate title bar area. Requirement: LAYOUT-02. This is a frontend + Tauri config change — no Rust backend commands needed.

</domain>

<decisions>
## Implementation Decisions

### Bar Merge Strategy
- Use Tauri's `decorations: false` to remove the native window title bar entirely
- Build a custom bar that contains both tab controls (repo name, close) and toolbar actions (undo, redo, pull, push, branch, stash, pop) in a single 36px-high row
- Set the merged bar area as the Tauri drag region using `data-tauri-drag-region` attribute
- Keep macOS traffic light buttons (close/minimize/maximize) in their default position — Tauri handles this automatically with `decorations: false`
- Bar height remains 36px (current toolbar height) — compact single row

### Content Layout in Merged Bar
- Left section: Traffic lights area (macOS auto-positioned) → Tab name + close button → flexible spacer
- Right section: Same as current toolbar order — Undo, Redo, Pull chevron, Push, Branch, Stash, Pop
- Cross-platform: same layout without traffic lights offset on Windows/Linux — no platform-specific layouts
- Welcome screen also uses merged bar (with just repo open controls, no toolbar actions)

### Claude's Discretion
- Exact padding around traffic lights area (Tauri may require specific CSS insets)
- How much left padding is needed on macOS to avoid overlapping traffic lights (typically ~70-80px)
- Whether the drag region includes the toolbar buttons or just the spacer area between tab and toolbar
- Transition animation (if any) when switching from welcome screen to repo view

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- `TabBar.svelte`: Current tab bar component (repo name + close button) — merge into the new unified bar
- `Toolbar.svelte`: Current toolbar component (all action buttons) — merge into the new unified bar
- `App.svelte` line 334-337: Current layout has TabBar and Toolbar in a flex row inside a 36px container — they're already adjacent

### Established Patterns
- Tauri `tauri.conf.json` for window config (decorations, title bar style)
- `data-tauri-drag-region` attribute for window drag areas (standard Tauri pattern)
- Current 36px bar height with flex layout and `var(--color-surface)` background

### Integration Points
- `src-tauri/tauri.conf.json`: Set `decorations: false` in the window config
- `src/App.svelte` lines 334-337: Merge TabBar and Toolbar rendering into unified bar
- `src/components/WelcomeScreen.svelte`: Needs custom title bar too (drag region + traffic lights)

</code_context>

<specifics>
## Specific Ideas

No specific requirements — standard Tauri custom title bar pattern well-documented

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>
