---
phase: 52-homebrew-distribution
plan: 01
subsystem: infra
tags: [github-actions, homebrew, cask, ci-cd, release]

requires:
  - phase: 51-cross-platform-release-pipeline
    provides: "4-platform build matrix with tauri-action producing DMG artifacts"
provides:
  - "3-job release pipeline: build -> publish -> update-tap"
  - "Auto-publish draft releases after all builds complete"
  - "Homebrew cask generation with SHA256 hashes pushed to homebrew-tap"
affects: []

tech-stack:
  added: []
  patterns: ["heredoc-based cask template with sed placeholder replacement"]

key-files:
  created: []
  modified:
    - ".github/workflows/release.yml"
    - "/Users/joaofnds/code/homebrew-tap/README.md"

key-decisions:
  - "Heredoc cask template with sed placeholder replacement for version, SHA256, and Tauri version"
  - "Prerelease tags (containing -) skip tap update via GitHub Actions if condition"

patterns-established:
  - "Release pipeline chain: build -> publish -> update-tap with job-level needs dependencies"

requirements-completed: [DIST-01]

duration: ~15min
completed: 2026-03-26
---

# Phase 52: Homebrew Distribution Summary

**3-job release pipeline (build -> publish -> update-tap) with auto-generated Homebrew cask formula pushed to joaofnds/homebrew-tap**

## Performance

- **Duration:** ~15 min
- **Tasks:** 3 (2 auto + 1 human-verify)
- **Files modified:** 2

## Accomplishments
- Extended release.yml with `publish` job that auto-publishes draft releases via `gh release edit --draft=false`
- Added `update-tap` job that downloads DMGs, computes SHA256 hashes, generates a Homebrew cask formula with on_intel/on_arm blocks, and pushes to homebrew-tap
- Updated homebrew-tap README with trunk entry in new Casks section
- Verified pipeline with v0.10.0-test1 tag: all 4 builds passed, publish succeeded, update-tap correctly skipped for prerelease

## Task Commits

1. **Task 1: Add publish and update-tap jobs to release workflow** - `43b3382` (feat)
2. **Task 2: Update homebrew-tap README with trunk cask entry** - `914c1f7` (docs, homebrew-tap repo)
3. **Task 3: Verify full pipeline with test release** - Human-verified: v0.10.0-test1 tag triggered successful build + publish, update-tap skipped for prerelease as expected

## Files Created/Modified
- `.github/workflows/release.yml` - Added publish and update-tap jobs after existing build job
- `/Users/joaofnds/code/homebrew-tap/README.md` - Added Casks section with trunk entry

## Decisions Made
- Used heredoc template with sed placeholder replacement for cask generation (simple, no extra tooling)
- Prerelease tags skip tap update via `if: ${{ !contains(github.ref_name, '-') }}`

## Deviations from Plan
None - plan executed as specified.

## Issues Encountered
None.

## Next Phase Readiness
- Full release pipeline operational: tag push -> build -> publish -> tap update
- HOMEBREW_TAP_TOKEN secret configured in trunk repo
- Ready for milestone completion

---
*Phase: 52-homebrew-distribution*
*Completed: 2026-03-26*
