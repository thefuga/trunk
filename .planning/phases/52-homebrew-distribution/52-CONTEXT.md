# Phase 52: Homebrew Distribution - Context

**Gathered:** 2026-03-26
**Status:** Ready for planning

<domain>
## Phase Boundary

Publish a Homebrew cask formula so macOS users can install Trunk via `brew install --cask joaofnds/tap/trunk`. The cask lives in the existing joaofnds/homebrew-tap repository and references .dmg artifacts from GitHub Releases with architecture-specific blocks for ARM and Intel.

</domain>

<decisions>
## Implementation Decisions

### Formula Update Process
- **D-01:** Automated via release workflow — a job in release.yml computes SHA256 of the .dmg files and pushes an updated trunk.rb to homebrew-tap after builds complete
- **D-02:** The release workflow auto-publishes the GitHub Release (removes draft status) after all builds complete, then updates the tap — fully automated pipeline

### Tap Repository
- **D-03:** Tap repo already exists at joaofnds/homebrew-tap with established Casks/ and Formula/ directories
- **D-04:** README updated to list trunk alongside existing entries (astro, asdf-install-latest)

### DMG Artifact URLs
- **D-05:** Same pattern as existing astro.rb cask — on_intel/on_arm blocks with GitHub Release .dmg URLs
- **D-06:** Hardcoded .dmg naming pattern (based on Tauri's productName "trunk") rather than dynamic discovery from release assets

### Claude's Discretion
- Exact .dmg filename pattern to hardcode (inspect tauri-action output to determine)
- How the release workflow authenticates to push to homebrew-tap (PAT, deploy key, or GitHub App)
- Cask metadata (desc, homepage, app stanza vs binary stanza for .dmg)
- SHA256 computation approach in the workflow job

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Requirements
- `.planning/REQUIREMENTS.md` — DIST-01 defines success criteria for this phase

### Existing Release Workflow
- `.github/workflows/release.yml` — Current release pipeline producing .dmg files for both macOS architectures; creates draft GitHub Release via tauri-action

### Tauri Configuration
- `src-tauri/tauri.conf.json` — productName "trunk", bundle targets "all", identifier "com.joaofnds.trunk"

### Tap Repository (external)
- `/Users/joaofnds/code/homebrew-tap/Casks/astro.rb` — Reference cask showing on_intel/on_arm pattern with GitHub Release URLs and SHA256 hashes
- `/Users/joaofnds/code/homebrew-tap/README.md` — Tap README to update with trunk entry

### Prior Phase Context
- `.planning/phases/51-cross-platform-release-pipeline/51-CONTEXT.md` — Release workflow decisions, runner selection, artifact naming

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `Casks/astro.rb` in homebrew-tap — exact template for architecture-specific cask with GitHub Release URLs
- `.github/workflows/release.yml` — release pipeline to extend with publish + cask update jobs

### Established Patterns
- tauri-action@v0 with tagName/releaseName/releaseDraft for release creation
- actions/upload-artifact@v4 for build artifacts
- Concurrency groups: `release-${{ github.ref }}` with cancel-in-progress
- `contents: write` permission already granted in release workflow

### Integration Points
- New job(s) in `.github/workflows/release.yml` — publish release + update homebrew-tap
- New file: `Casks/trunk.rb` in joaofnds/homebrew-tap repository
- Updated: `README.md` in joaofnds/homebrew-tap repository

</code_context>

<specifics>
## Specific Ideas

- User wants the same cask pattern as astro.rb — proven working template in the same tap repo
- Full automation: tag push → build → publish release → update cask formula — no manual steps

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 52-homebrew-distribution*
*Context gathered: 2026-03-26*
