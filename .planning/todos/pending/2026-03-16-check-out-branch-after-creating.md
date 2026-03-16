---
created: 2026-03-16T14:33:22.613Z
title: Check out branch after creating
area: ui
files: []
---

## Problem

After creating a new branch via the UI, the user is not automatically switched to that branch. This requires an extra manual step to check out the newly created branch, which is a friction point in the workflow. Users expect that creating a branch implies they want to work on it immediately.

## Solution

After the branch creation operation succeeds, automatically run `git checkout <new-branch>` (or the equivalent programmatic call) so the user lands on the new branch without an extra click or command.
