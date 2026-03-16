---
created: 2026-03-16T19:44:06.230Z
title: Remove top bar left offset in fullscreen mode
area: ui
files: []
---

## Problem

When using cmd-ctrl-f to expand the window to fullscreen, the top bar retains the left offset that was originally added to make room for the window traffic light controls (close/minimize/maximize). In fullscreen mode, those controls are not visible, so the offset creates awkward empty space on the left side of the top bar.

## Solution

Detect when the window is in fullscreen mode and conditionally remove the left offset from the top bar. The offset should only apply when the window is in normal (non-fullscreen) windowed mode where the traffic light controls are present.
