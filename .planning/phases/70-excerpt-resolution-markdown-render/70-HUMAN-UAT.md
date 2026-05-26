---
status: partial
phase: 70-excerpt-resolution-markdown-render
source: [70-VERIFICATION.md]
started: 2026-05-26T14:50:00Z
updated: 2026-05-26T14:50:00Z
---

## Current Test

[awaiting human testing]

## Tests

### 1. End-to-End DOC-01 Generate Flow

expected: Open a repo with ≥1 review comment, click "Generate" in ReviewPanel, panel transitions to the preview face showing a well-formed markdown document with commit references, fenced code excerpts, and comment text. "Back to comments" returns to the list face without losing the preview cache.
result: [pending]

### 2. Multi-Hunk Diff Excerpt Correctness (CR-01)

expected: Create a review comment anchored to a specific hunk in a file with ≥2 diff hunks in the same commit. Generate the review doc and inspect the fenced excerpt — only lines from the anchored hunk should appear. No lines from other hunks may leak into the block.
result: [pending]

### 3. Cross-Repo Preview Isolation (WR-01)

expected: Generate a review doc for repo A, switch to repo B without closing the app. Either (a) the preview is cleared when switching repos, or (b) a re-generate is required and produces repo B content. User should never see repo A content in repo B's context.
result: [pending]

## Summary

total: 3
passed: 0
issues: 0
pending: 3
skipped: 0
blocked: 0

## Gaps
