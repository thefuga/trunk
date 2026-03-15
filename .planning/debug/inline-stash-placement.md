---
status: awaiting_human_verify
trigger: "When a stash's parent commit is directly below it in the same column, the stash should render as a straight vertical dashed line going up from the parent — not branch off to the right into its own column"
created: 2026-03-14T00:00:00Z
updated: 2026-03-14T00:00:00Z
---

## Current Focus

hypothesis: CONFIRMED — The inline stash approach was over-engineered. Stashes should use the exact same algorithm as regular branch tips. The only difference is visual: dashed edges via stash_lanes.
test: Remove all inline stash special-casing (inline_stash_oids, inline_stash_colors, oid_position, can_inline, special Phase 1/4 branches). Let stashes flow through normal branch-tip codepath.
expecting: Stashes always branch right to own column with dashed visuals, identical to regular branches.
next_action: Human verification against ~/code/livefire/api

## Symptoms

expected: When a stash's parent commit is directly below it and the lane above the parent is unoccupied, the stash should render inline (same column as parent) with a straight vertical dashed line connecting them
actual: Stash always branches to the right into its own column regardless of lane occupancy
errors: No runtime errors — visual/algorithmic issue in graph layout
reproduction: Open any repo with git stashes. The stash always renders one column to the right of its parent commit, even when the parent's column is free above it.
started: This is the original behavior. It has never worked inline. Two prior fix attempts failed.

## Eliminated

- hypothesis: Post-process in TypeScript (move stash node + rewrite edges after buildGraphData)
  evidence: Failed - edges, nodes, connections are interdependent. Moving one without adjusting others creates visual desync. WIP-to-HEAD edge conflicts, collision detection issues.
  timestamp: prior-attempt-2

- hypothesis: Simply change stash column from parent_col+1 to parent_col in Rust
  evidence: Failed - broke colors (lane_colors.insert overwrote parent branch color), broke dashing (stash_lanes.insert at parent col made ALL edges in that column dashed), broke pending_parents (HEAD chain pre-reserves col 0, stash hit wrong codepath bypassing dashed logic)
  timestamp: prior-attempt-1

- hypothesis: Inline stash placement with can_inline check (active_lanes + pending_parents position-aware scan)
  evidence: Over-engineered. Added inline_stash_oids, inline_stash_colors, oid_position map, special Phase 1/4 handling. First version had a bug (only checked active_lanes, missed pending_parents). Second version with position-aware check passed tests but user rejected the entire approach — stashes should use the same algorithm as branches.
  timestamp: 2026-03-14T00:12:00Z

## Evidence

- timestamp: 2026-03-14T00:01:00Z
  checked: All 86 Rust tests pass as baseline
  found: Tests confirm current behavior: stash at col > parent, ForkRight on parent, dashed edges
  implication: Any change must keep all existing tests passing (or update tests that explicitly assert col > parent)

- timestamp: 2026-03-14T00:10:00Z
  checked: User visual verification of first inline fix attempt
  found: Stash on mid-chain HEAD parent was incorrectly placed inline at col 0
  implication: Inline approach has fundamental issues with HEAD chain pre-reservation

- timestamp: 2026-03-14T00:13:00Z
  checked: User direction change — abandon inline approach entirely
  found: Stashes should be treated as regular branch tips. Same algorithm, only dashed visuals different.
  implication: Remove ALL inline stash infrastructure, simplify Phase 1 and Phase 4

- timestamp: 2026-03-14T00:14:00Z
  checked: Removed all stash special-casing from lane algorithm
  found: |
    Removed: inline_stash_oids, inline_stash_colors, oid_position map, can_inline check,
    entire `else if is_stash` Phase 1 block, entire `else if is_stash` Phase 4 block,
    inline stash color handling in commit_color.
    
    Kept: parent filtering (first parent only), stash_lanes.insert for dashed visuals,
    is_stash flag on output, orphan stash guard in Phase 4.
    
    88 Rust tests pass, 121 TS tests pass. Tests updated to expect branch-right behavior.
  implication: Stash algorithm is now identical to regular branches — massively simplified

## Resolution

root_cause: |
  The stash lane algorithm was over-engineered with inline placement logic that didn't
  work correctly. The fundamental insight is that a stash IS a regular branch tip — it
  should flow through the exact same algorithm as any other commit. The only difference
  is visual: dashed square + dashed edges. All the inline_stash_oids, inline_stash_colors,
  oid_position, can_inline checks were unnecessary complexity that introduced bugs.

fix: |
  Removed all stash special-casing from the lane algorithm:
  
  1. Phase 1 (column assignment): Removed entire `else if is_stash` block (~120 lines).
     Stashes now fall through to the standard `else` branch for new chains/branch tips.
     Only addition: `stash_lanes.insert(c)` after column assignment for dashed visuals.
  
  2. Phase 4 (first-parent handling): Removed `else if is_stash` block. Stashes now
     use the normal parent claim codepath. Only kept orphan stash guard: if stash parent
     not in base_oid_set, don't keep lane alive (prevents ghost lanes).
  
  3. Removed declarations: inline_stash_oids, inline_stash_colors, oid_position map.
  
  4. Simplified commit_color: removed inline stash check, always uses lane_colors.
  
  5. Updated 3 tests from inline assertions to branch-right assertions:
     - stash_inline_when_lane_unoccupied → stash_branches_right_like_regular_branch
     - multiple_stashes_on_same_parent → both stashes branch right, 2 ForkRight edges
     - stash_inline_when_parent_is_head_tip → stash_branches_right_with_topic_branch
  
  6. Updated COMMIT-GRAPH-ARCHITECTURE.md to reflect simplified approach.

verification: |
  - 88 Rust tests pass (all existing, 3 updated for new behavior)
  - 121 TypeScript tests pass (no TS changes needed)
  - Tests verify:
    - Stash always branches right to own column
    - Stash has own color, dashed Straight edge
    - Parent has dashed ForkRight edge to stash column
    - Multiple stashes: each at own column, each ForkRight on parent
    - Mid-chain stash: branches right (HEAD chain at col 0 unaffected)
    - Orphan stash: lane ends cleanly, no ghost lanes
  - Pending: visual verification against ~/code/livefire/api

files_changed:
  - src-tauri/src/git/graph.rs
  - .planning/COMMIT-GRAPH-ARCHITECTURE.md
