---
phase: 71
slug: output-clipboard-save-to-file
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-05-26
---

# Phase 71 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | vitest 4.x + @testing-library/svelte 5.3.1 + jsdom 29 |
| **Config file** | `vite.config.ts` (test block) + `vitest-setup.ts` (global stubs) |
| **Quick run command** | `npx vitest run src/components/ReviewDocPreview.test.ts` |
| **Full suite command** | `just check` |
| **Estimated runtime** | quick ~3s, full ~60s |

---

## Sampling Rate

- **After every task commit:** Run `npx vitest run src/components/ReviewDocPreview.test.ts`
- **After every plan wave:** Run `npx vitest run` (full vitest)
- **Before `/gsd:verify-work`:** `just check` must be green (fmt, biome, svelte-check, clippy, cargo-test, vitest)
- **Max feedback latency:** ~3 seconds (single test file)

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 71-01-W0-01 | 01 | 0 | OUT-01 | — | N/A | unit scaffold | `test -f src/components/ReviewDocPreview.test.ts` | ❌ W0 | ⬜ pending |
| 71-01-01 | 01 | 1 | OUT-01 | — | `writeText` invoked with the verbatim `markdown` prop (no re-derivation) | unit (component-mount, mocked plugin) | `npx vitest run src/components/ReviewDocPreview.test.ts -t "writes markdown prop"` | ❌ W0 | ⬜ pending |
| 71-01-02 | 01 | 1 | OUT-01 | — | Success state transition `idle → copied` is observable in DOM | unit (component-mount) | `npx vitest run src/components/ReviewDocPreview.test.ts -t "shows Copied affordance on success"` | ❌ W0 | ⬜ pending |
| 71-01-03 | 01 | 1 | OUT-01 | — | Affordance reverts after ~1500ms (timer cleanup verified) | unit (component-mount + fake timers) | `npx vitest run src/components/ReviewDocPreview.test.ts -t "reverts after timeout"` | ❌ W0 | ⬜ pending |
| 71-01-04 | 01 | 1 | OUT-01 | — | Re-click during the window clears the prior timer and extends the affordance (D-04) | unit (component-mount + fake timers) | `npx vitest run src/components/ReviewDocPreview.test.ts -t "remains clickable during window"` | ❌ W0 | ⬜ pending |
| 71-01-05 | 01 | 1 | OUT-01 | — | Failure surfaces via `showToast("Failed to copy: <msg>", "error")` — no silent failure | unit (component-mount, mocked rejection, mocked showToast) | `npx vitest run src/components/ReviewDocPreview.test.ts -t "shows error toast on failure"` | ❌ W0 | ⬜ pending |
| 71-01-06 | 01 | 1 | OUT-01 | — | Failure path does NOT flip `copied` (no false-success affordance) | unit (component-mount, mocked rejection) | `npx vitest run src/components/ReviewDocPreview.test.ts -t "does not flip copied on failure"` | ❌ W0 | ⬜ pending |
| 71-01-07 | 01 | 1 | OUT-01 | — | Non-Error rejection value is safely coerced via `String(e)` — no `"[object undefined]"`/`"Failed to copy: "` | unit (component-mount, plugin rejects with raw string) | `npx vitest run src/components/ReviewDocPreview.test.ts -t "coerces non-Error rejection"` | ❌ W0 | ⬜ pending |
| 71-01-08 | 01 | 1 | OUT-01 | — | `← Back to comments` button still invokes `onBack` (no displacement regression) | unit (component-mount) | `npx vitest run src/components/ReviewDocPreview.test.ts -t "back button still invokes onBack"` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `src/components/ReviewDocPreview.test.ts` — create the test file with the mock skeleton from RESEARCH.md "Recommended mock skeleton" (mocks `@tauri-apps/plugin-clipboard-manager` and `../lib/toast.svelte.js`; `vi.useFakeTimers` in `beforeEach`, `vi.useRealTimers` in `afterEach`).
- [ ] No new fixtures or framework installs required — `vitest-setup.ts` already stubs `ResizeObserver` and `Element.prototype.animate`, which is sufficient for `@testing-library/svelte` mounts of `ReviewDocPreview`.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Real OS clipboard receives the markdown | OUT-01 | jsdom mocks `writeText`; only the real Tauri plugin writes to the OS clipboard | In `just dev`: open a review session with ≥1 resolved comment, generate the preview, click Copy, switch to TextEdit/VS Code, ⌘V, confirm full markdown lands. |
| Visual styling parity with `← Back to comments` | OUT-01 (D-02) | jsdom can assert class names but not painted appearance | In `just dev`: open the preview, visually verify the Copy button border/padding/font-size/hover color match the back button. Screenshot if borderline. |
| "Copied" duration feels right | OUT-01 (D-03 / Discretion) | Subjective ~1.5s window | In `just dev`: click Copy, eyeball the revert; tune to 1200/1800 if it feels off. |
| Failure path surfaces a readable toast | OUT-01 (D-05) | Real plugin error message vs. mocked string | In `just dev`: temporarily revoke `clipboard-manager:allow-write-text` in `src-tauri/capabilities/default.json`, restart, click Copy, confirm the toast text is readable. **Restore capability before commit.** |
| Theme custom-property correctness across light/dark | OUT-01 (CLAUDE.md no-inline-colors rule) | CSS custom properties resolve at paint time | In `just dev`: toggle theme (if app supports it; otherwise verify the OS-level theme variant), confirm border/hover colors track the theme. |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references (`ReviewDocPreview.test.ts` created in W0)
- [ ] No watch-mode flags (`vitest run`, not `vitest`)
- [ ] Feedback latency < 5s (per-file quick run is ~3s)
- [ ] `nyquist_compliant: true` set in frontmatter (flip after planner consumes this and tasks are wired)

**Approval:** pending
