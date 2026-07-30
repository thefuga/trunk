# GitHub Write Mechanics

Load this reference after the draft passes self-check and before asking for final
confirmation. It covers create-mode writes, refine-mode writes, Project updates,
and read-back verification.

## Pre-Write Checklist

- Run `viber plan validate` against the confirmed `viber-delivery-plan/v1`
  draft. Do not proceed while it reports errors.
- Re-run the repo identity guard: local `origin` and `gh repo view` must resolve
  to the same `OWNER/REPO` that will receive writes.
- Confirm the selected adapter and native Project field IDs/options are known, or
  issue-only mode was explicitly selected.
- Confirm the dependency graph has a topological create-order.
- For planned process sub-issues, confirm the parent deliverable issue and every
  child title/body are included in the preview. The non-dispatchable mechanism is
  GitHub's native parent relationship; the daemon excludes `issue.parent` from
  dispatch candidates (design 0035).
- Show the full preview and receive explicit operator confirmation.

If the operator declines, make zero writes.

## Create Mode

Create in dependency order so blockers exist before blocked issues reference
them.

1. Create the milestone first when one is used:

   ```sh
   gh api repos/OWNER/REPO/milestones \
     -f title='<title>' \
     -f description='<context>'
   ```

2. Create each issue, capturing number, URL, and node/database ID:

   ```sh
   gh issue create --repo OWNER/REPO \
     --title '<title>' \
     --body-file <draft-body> \
     [--milestone '<title>']
   ```

3. For each Viber-backed deliverable with planned process steps, create each
   process-step child issue, then attach it to the parent deliverable issue with
   GitHub's sub-issues API. The API requires the numeric REST issue `id`, not the
   issue number:

   ```sh
   child_url=$(gh issue create --repo OWNER/REPO \
     --title '<child title>' \
     --body-file <child-draft-body> \
     [--milestone '<title>'])
   child_number=${child_url##*/}
   child_id=$(gh api repos/OWNER/REPO/issues/"$child_number" --jq .id)

   gh api -X POST repos/OWNER/REPO/issues/<parent-number>/sub_issues \
     -F sub_issue_id="$child_id"
   ```

   Do not rely on labels, issue type, or Project fields to make the child
   non-dispatchable. The native parent link is the mechanism. If the sub-issues
   endpoint is unavailable, stop and report the created parent and child issues;
   do not silently fall back to dispatchable standalone process issues.

4. Wire native dependencies. Confirm the current endpoint shape from
   `docs/github/issues/using-issues/creating-issue-dependencies.md` or live API
   docs before calling:

   ```sh
   gh api repos/OWNER/REPO/issues/<blocked>/dependencies/blocked_by \
     -F issue_id="$(gh api repos/OWNER/REPO/issues/<blocker> --jq .id)"
   ```

   If the dependencies API is unavailable on the host, stop and report the
   created objects and the missing native edge writes.

5. Apply the selected destination adapter:

   - If Project automation should auto-add issues, read back first. Add items
     explicitly only when the expected item is absent.
   - For Project v2 item writes, use the clean token environment:

     ```sh
     env -u GH_TOKEN -u GITHUB_TOKEN gh project item-add <number> \
       --owner <owner> \
       --url <issue-url>

     env -u GH_TOKEN -u GITHUB_TOKEN gh project item-edit \
       --id <item-id> \
       --project-id <project-id> \
       --field-id <field-id> \
       --single-select-option-id <option-id>
     ```

   - Set only the fields confirmed in the preview.
   - For Viber-backed process sub-issues, either verify the Project auto-added
     each child into a non-active landing Status or explicitly set that same
     landing Status. This is a belt-and-suspenders parking step; the daemon's
     hard guard is still the native parent relationship.
   - For issue-only mode, perform no Project commands.

## Refine Mode

Overwrite the existing issue body only after confirmation:

```sh
gh issue edit <N> --repo OWNER/REPO \
  --body-file <refined-body> \
  [--title '<refined title>']
```

Read the issue back and compare title/body with the confirmed replacement. Do
not add a comment as the refinement; the issue body is the artifact.

## Closing Assertion

After all writes, build a read-back table and diff it against the confirmed plan:

- issue titles, URLs, and bodies;
- milestone assignment when used;
- parent/sub-issue relationships for process steps;
- native `blockedBy` edges;
- Project membership;
- every confirmed Project field value;
- intentionally unset fields;
- refine-mode title/body replacement.

If any read-back value differs from the confirmed plan, stop with the mismatch,
the objects already created or edited, and the command that failed or read back
unexpectedly. Apply the SKILL.md gotchas; do not perform an unconfirmed repair.

## Partial Failure

There is no automated recovery manifest or idempotency key. On any failed `gh`
call or failed closing assertion, stop and report:

- objects created or edited, with numbers/URLs;
- native dependency or Project writes that succeeded;
- native dependency or Project writes that failed or were not attempted;
- exact read-back mismatch;
- recommended operator reconciliation step.
