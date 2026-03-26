# Phase 52: Homebrew Distribution - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-03-26
**Phase:** 52-homebrew-distribution
**Areas discussed:** Formula update process, Tap repository setup, DMG artifact URLs

---

## Formula Update Process

### Q1: How should the cask formula be updated when you push a new release tag?

| Option | Description | Selected |
|--------|-------------|----------|
| Automated via release workflow | Add a job to release.yml that computes SHA256 of the .dmg files and pushes an updated trunk.rb to homebrew-tap after builds complete | ✓ |
| Manual updates | After each release, manually edit trunk.rb with new version, URLs, and SHA256 hashes | |
| Separate workflow in tap repo | A workflow in homebrew-tap triggered by release events from the trunk repo that self-updates the cask | |

**User's choice:** Automated via release workflow (Recommended)
**Notes:** None

### Q2: Should the workflow auto-publish the release before updating the cask?

| Option | Description | Selected |
|--------|-------------|----------|
| Auto-publish then update cask | The release workflow publishes the GitHub Release after all builds complete, then updates the tap — fully automated pipeline | ✓ |
| Keep draft, update cask manually | Builds stay as draft releases; you publish manually, then the cask update either triggers separately or is also manual | |
| You decide | Claude determines the best approach for release publishing and cask update sequencing | |

**User's choice:** Auto-publish then update cask (Recommended)
**Notes:** None

---

## Tap Repository Setup

### Q3: Should the README be updated to list trunk alongside astro?

| Option | Description | Selected |
|--------|-------------|----------|
| Yes, update README | Add trunk to the README table alongside astro and asdf-install-latest | ✓ |
| Skip README update | Just add the cask file — README is not critical | |
| You decide | Claude determines whether and how to update the README | |

**User's choice:** Yes, update README (Recommended)
**Notes:** User directed to check /Users/joaofnds/code/homebrew-tap before discussion — tap repo already exists with Casks/ and Formula/ directories and astro.rb as reference

---

## DMG Artifact URLs

### Q4: Should the cask update job discover .dmg filenames dynamically or hardcode a pattern?

| Option | Description | Selected |
|--------|-------------|----------|
| Discover dynamically | The update job queries the GitHub Release API for assets matching *.dmg, matches by architecture, and uses the actual URLs | |
| Hardcode pattern | Assume a fixed naming pattern like trunk_x.y.z_aarch64.dmg — simpler but fragile if Tauri changes naming | ✓ |
| You decide | Claude determines the best approach for .dmg URL resolution | |

**User's choice:** Hardcode pattern
**Notes:** None

---

## Claude's Discretion

- Exact .dmg filename pattern to hardcode
- Authentication method for pushing to homebrew-tap from release workflow
- Cask metadata details (desc, homepage, app stanza)
- SHA256 computation approach

## Deferred Ideas

None — discussion stayed within phase scope
