# macOS Pre-Release E2E Validation Checklist

Manual validation checklist for macOS builds. Run before each release
since macOS E2E tests cannot run in CI (no reliable WKWebView WebDriver).

## Prerequisites

- [ ] Debug build available: `bun run tauri build -- --debug --no-bundle`
- [ ] Launch the app: `./src-tauri/target/debug/trunk`

## 1. Repository Opening

- [ ] Click "Open Repository" on welcome screen
- [ ] Select a git repository from the file dialog
- [ ] Verify: Commit graph loads with commit rows visible
- [ ] Verify: Branch sidebar shows local branches

## 2. Commit History Browsing (E2E-02)

- [ ] Verify: Commit rows display summary, author, date, SHA columns
- [ ] Click on a commit row
- [ ] Verify: Commit detail panel shows diff for selected commit
- [ ] Scroll through history
- [ ] Verify: Virtual list renders commits without visual glitches

## 3. Staging and Committing (E2E-03)

- [ ] Modify a file in the opened repository (external editor)
- [ ] Verify: Modified file appears in "Unstaged Files" section
- [ ] Click the "+" button on the unstaged file
- [ ] Verify: File moves to "Staged Files" section
- [ ] Type a commit message in the subject field
- [ ] Click "Commit" button
- [ ] Verify: Commit appears at top of commit graph
- [ ] Verify: Subject field is cleared after commit

## 4. Branch Operations (E2E-04)

- [ ] Verify: Local branches listed in sidebar
- [ ] Double-click a non-HEAD branch
- [ ] Verify: Branch checkout succeeds (branch becomes bold/accented)
- [ ] Click "+" button in Local section header
- [ ] Type a new branch name and press Enter
- [ ] Verify: New branch appears in sidebar
- [ ] Right-click a non-HEAD branch, select "Delete"
- [ ] Confirm deletion in dialog
- [ ] Verify: Branch removed from sidebar

## 5. General

- [ ] Verify: No console errors in WebKit developer tools
- [ ] Verify: Window title bar overlay renders correctly
- [ ] Verify: App responds to Cmd+T (new tab), Cmd+W (close tab)

---

*Last updated: Phase 58 (E2E Test Harness)*
*Covers: E2E-02, E2E-03, E2E-04 (manual equivalent of Linux CI tests)*
