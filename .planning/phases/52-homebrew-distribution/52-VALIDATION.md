---
phase: 52
slug: homebrew-distribution
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-26
---

# Phase 52 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Manual validation (no automated test framework for workflow + Homebrew) |
| **Config file** | N/A |
| **Quick run command** | `curl -s https://raw.githubusercontent.com/joaofnds/homebrew-tap/main/Casks/trunk.rb` |
| **Full suite command** | `brew install --cask joaofnds/tap/trunk && brew uninstall --cask trunk` |
| **Estimated runtime** | ~60 seconds (network-dependent) |

---

## Sampling Rate

- **After every task commit:** Review generated cask syntax manually
- **After every plan wave:** Trigger a test release (tag push) and verify full pipeline
- **Before `/gsd:verify-work`:** `brew install --cask joaofnds/tap/trunk` succeeds on a clean machine
- **Max feedback latency:** 120 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 52-01-01 | 01 | 1 | DIST-01 | smoke | `curl -s https://raw.githubusercontent.com/joaofnds/homebrew-tap/main/Casks/trunk.rb` | ❌ W0 | ⬜ pending |
| 52-01-02 | 01 | 1 | DIST-01 | smoke | Verify release workflow publishes release and updates tap | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `HOMEBREW_TAP_TOKEN` secret — fine-grained PAT scoped to joaofnds/homebrew-tap must be created and stored in trunk repo settings
- [ ] No automated unit tests possible — this is workflow + external service integration

*Existing CI infrastructure (Phase 50) covers workflow syntax validation via GitHub Actions.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Cask installs Trunk from .dmg | DIST-01 | Requires macOS + Homebrew + network access to GitHub Releases | `brew install --cask joaofnds/tap/trunk` then verify `trunk.app` appears in /Applications |
| Release workflow publishes and updates tap | DIST-01 | Requires tag push to trigger workflow, cross-repo push | Push a `v*` tag, wait for workflow, check homebrew-tap repo for updated trunk.rb |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 120s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
