<script lang="ts">
  import { safeInvoke, type TrunkError } from '../lib/invoke.js';
  import { showToast } from '../lib/toast.svelte.js';
  import type { MergeSides } from '../lib/types.js';
  import {
    parseConflictRegions,
    computeOutput,
    takeAllCurrent,
    takeAllIncoming,
    toggleHunk,
    toggleLine,
    getConflictIndices,
    type ConflictRegion,
  } from '../lib/merge-parser.js';
  import { Check, CircleCheck, CircleX, ChevronUp, ChevronDown, X } from '@lucide/svelte';

  interface Props {
    repoPath: string;
    filePath: string;
    onclose: () => void;
    onresolved: () => void;
  }

  let { repoPath, filePath, onclose, onresolved }: Props = $props();

  // ---------- Constants ----------
  const LINE_HEIGHT = 18;
  const HEADER_HEIGHT = 24;
  const OVERSCAN = 20;

  // ---------- State ----------
  let regions = $state<ConflictRegion[]>([]);
  let takenLines = $state<Set<string>>(new Set());
  let manualEdit = $state(false);
  let manualText = $state('');
  let loading = $state(true);
  let error = $state<string | null>(null);
  let focusedConflictIdx = $state(0);
  let saving = $state(false);

  let panelRefs: HTMLElement[] = [];
  let panelScrollTop = $state(0);
  let panelViewportHeight = $state(400);

  // ---------- Flat row types for virtualization ----------
  interface FlatRow {
    type: 'context' | 'conflict-header' | 'conflict-line' | 'padding';
    regionIdx: number;
    lineIdx: number;
    text: string;
    key: string;
    lineNum: number;
    conflictNum: number;
    height: number;
  }

  /** Flatten both panels together so conflict regions are padded to equal height (aligned headers). */
  function flattenAligned(regions: ConflictRegion[]): { ours: FlatRow[]; theirs: FlatRow[] } {
    const ours: FlatRow[] = [];
    const theirs: FlatRow[] = [];
    let oursLineNum = 1, theirsLineNum = 1;
    let conflictCount = 0;

    for (let i = 0; i < regions.length; i++) {
      const region = regions[i];
      if (region.type === 'context') {
        for (let j = 0; j < region.oursLines.length; j++) {
          ours.push({ type: 'context', regionIdx: i, lineIdx: j, text: region.oursLines[j], key: '', lineNum: oursLineNum + j, conflictNum: 0, height: LINE_HEIGHT });
        }
        for (let j = 0; j < region.theirsLines.length; j++) {
          theirs.push({ type: 'context', regionIdx: i, lineIdx: j, text: region.theirsLines[j], key: '', lineNum: theirsLineNum + j, conflictNum: 0, height: LINE_HEIGHT });
        }
        oursLineNum += region.oursLines.length;
        theirsLineNum += region.theirsLines.length;
      } else {
        conflictCount++;
        // Headers
        ours.push({ type: 'conflict-header', regionIdx: i, lineIdx: -1, text: '', key: '', lineNum: 0, conflictNum: conflictCount, height: HEADER_HEIGHT });
        theirs.push({ type: 'conflict-header', regionIdx: i, lineIdx: -1, text: '', key: '', lineNum: 0, conflictNum: conflictCount, height: HEADER_HEIGHT });
        // Conflict lines
        for (let j = 0; j < region.oursLines.length; j++) {
          ours.push({ type: 'conflict-line', regionIdx: i, lineIdx: j, text: region.oursLines[j], key: `ours-${i}-${j}`, lineNum: oursLineNum + j, conflictNum: 0, height: LINE_HEIGHT });
        }
        for (let j = 0; j < region.theirsLines.length; j++) {
          theirs.push({ type: 'conflict-line', regionIdx: i, lineIdx: j, text: region.theirsLines[j], key: `theirs-${i}-${j}`, lineNum: theirsLineNum + j, conflictNum: 0, height: LINE_HEIGHT });
        }
        // Pad the shorter side so next region starts at the same offset
        const diff = region.oursLines.length - region.theirsLines.length;
        if (diff > 0) {
          theirs.push({ type: 'padding', regionIdx: i, lineIdx: -2, text: '', key: '', lineNum: 0, conflictNum: 0, height: diff * LINE_HEIGHT });
        } else if (diff < 0) {
          ours.push({ type: 'padding', regionIdx: i, lineIdx: -2, text: '', key: '', lineNum: 0, conflictNum: 0, height: -diff * LINE_HEIGHT });
        }
        oursLineNum += region.oursLines.length;
        theirsLineNum += region.theirsLines.length;
      }
    }
    return { ours, theirs };
  }

  function computeOffsets(rows: FlatRow[]): number[] {
    const offsets = new Array(rows.length + 1);
    offsets[0] = 0;
    for (let i = 0; i < rows.length; i++) {
      offsets[i + 1] = offsets[i] + rows[i].height;
    }
    return offsets;
  }

  function getVisibleRange(scrollTop: number, viewportHeight: number, offsets: number[]): [number, number] {
    const totalRows = offsets.length - 1;
    if (totalRows === 0) return [0, 0];
    // Binary search for first visible row
    let lo = 0, hi = totalRows;
    while (lo < hi) {
      const mid = (lo + hi) >> 1;
      if (offsets[mid + 1] <= scrollTop) lo = mid + 1;
      else hi = mid;
    }
    const start = Math.max(0, lo - OVERSCAN);
    // Find last visible row
    const bottom = scrollTop + viewportHeight;
    lo = start;
    hi = totalRows;
    while (lo < hi) {
      const mid = (lo + hi) >> 1;
      if (offsets[mid] < bottom) lo = mid + 1;
      else hi = mid;
    }
    const end = Math.min(totalRows, lo + OVERSCAN);
    return [start, end];
  }

  // ---------- Derived ----------
  let conflictIndices = $derived(getConflictIndices(regions));
  let outputText = $derived.by(() => {
    if (manualEdit) return manualText;
    return computeOutput(regions, takenLines);
  });
  let hasPrev = $derived(focusedConflictIdx > 0);
  let hasNext = $derived(focusedConflictIdx < conflictIndices.length - 1);
  let hasConflicts = $derived(conflictIndices.length > 0);

  // Virtualization derived state
  let aligned = $derived(flattenAligned(regions));
  let oursFlat = $derived(aligned.ours);
  let theirsFlat = $derived(aligned.theirs);
  let oursOffsets = $derived(computeOffsets(oursFlat));
  let theirsOffsets = $derived(computeOffsets(theirsFlat));
  let oursTotalHeight = $derived(oursOffsets[oursFlat.length] ?? 0);
  let theirsTotalHeight = $derived(theirsOffsets[theirsFlat.length] ?? 0);
  let oursVisible = $derived(getVisibleRange(panelScrollTop, panelViewportHeight, oursOffsets));
  let theirsVisible = $derived(getVisibleRange(panelScrollTop, panelViewportHeight, theirsOffsets));

  // ---------- Data loading ----------
  $effect(() => {
    // Re-run when filePath changes
    const currentPath = filePath;
    loading = true;
    error = null;

    safeInvoke<MergeSides>('get_merge_sides', { path: repoPath, filePath: currentPath })
      .then((result) => {
        regions = parseConflictRegions(result.base, result.ours, result.theirs);
        takenLines = new Set();
        manualEdit = false;
        manualText = '';
        focusedConflictIdx = 0;
        panelScrollTop = 0;
        loading = false;
      })
      .catch(() => {
        // Merge state no longer available (e.g. git reset) — close the editor
        onclose();
      });
  });

  // ---------- Synchronized scroll ----------
  let scrolling = false;
  let scrollRaf = 0;

  function handleScroll(sourceIdx: number) {
    if (scrolling) return;
    scrolling = true;
    const source = panelRefs[sourceIdx];
    if (!source) { scrolling = false; return; }
    const st = source.scrollTop;
    // Sync other panels immediately (no DOM mutation, so no jitter)
    panelRefs.forEach((el, i) => {
      if (el && i !== sourceIdx) el.scrollTop = st;
    });
    // Defer virtualization state update to next frame so DOM mutations
    // don't happen mid-scroll (which causes jitter on the source panel)
    cancelAnimationFrame(scrollRaf);
    scrollRaf = requestAnimationFrame(() => {
      panelScrollTop = st;
      scrolling = false;
    });
  }

  // ---------- Event handlers ----------
  function handleTakeAllCurrent() {
    takenLines = takeAllCurrent(regions);
    manualEdit = false;
  }

  function handleTakeAllIncoming() {
    takenLines = takeAllIncoming(regions);
    manualEdit = false;
  }

  function handleToggleHunk(side: 'ours' | 'theirs', regionIdx: number) {
    takenLines = toggleHunk(side, regionIdx, regions, takenLines);
    manualEdit = false;
  }

  let lastClickedKey = $state<string | null>(null);

  function handleToggleLine(key: string, event?: MouseEvent) {
    if (event?.shiftKey && lastClickedKey) {
      // Parse keys: "side-regionIdx-lineIdx"
      const [side, regStr, lineStr] = key.split('-');
      const [lastSide, lastRegStr, lastLineStr] = lastClickedKey.split('-');
      if (side === lastSide && regStr === lastRegStr) {
        const from = Math.min(+lineStr, +lastLineStr);
        const to = Math.max(+lineStr, +lastLineStr);
        // Determine action: if target line is not taken, select the range; otherwise deselect
        const selecting = !takenLines.has(key);
        const result = new Set(takenLines);
        for (let j = from; j <= to; j++) {
          const k = `${side}-${regStr}-${j}`;
          if (selecting) result.add(k);
          else result.delete(k);
        }
        takenLines = result;
        manualEdit = false;
        lastClickedKey = key;
        return;
      }
    }
    takenLines = toggleLine(key, takenLines);
    manualEdit = false;
    lastClickedKey = key;
  }

  function handleOutputEdit(e: Event) {
    manualEdit = true;
    manualText = (e.target as HTMLTextAreaElement).value;
  }

  function handlePrevConflict() {
    if (!hasPrev) return;
    focusedConflictIdx--;
    scrollToConflict(focusedConflictIdx);
  }

  function handleNextConflict() {
    if (!hasNext) return;
    focusedConflictIdx++;
    scrollToConflict(focusedConflictIdx);
  }

  function scrollToConflict(idx: number) {
    const regionIndex = conflictIndices[idx];
    if (regionIndex == null) return;
    // Find the conflict header row in the flat array
    const rowIdx = oursFlat.findIndex(r => r.type === 'conflict-header' && r.regionIdx === regionIndex);
    if (rowIdx === -1) return;
    const targetTop = oursOffsets[rowIdx];
    const scrollTo = Math.max(0, targetTop - panelViewportHeight / 2 + HEADER_HEIGHT / 2);
    scrolling = true;
    for (const panel of panelRefs) {
      if (panel) panel.scrollTop = scrollTo;
    }
    panelScrollTop = scrollTo;
    requestAnimationFrame(() => { scrolling = false; });
  }

  async function handleSaveAndResolve() {
    saving = true;
    try {
      await safeInvoke('save_merge_result', {
        path: repoPath,
        filePath,
        content: outputText,
      });
      showToast('Resolved ' + filePath, 'success');
      onresolved();
    } catch (e) {
      const err = e as TrunkError;
      showToast(err.message ?? 'Save failed', 'error');
    } finally {
      saving = false;
    }
  }

  // ---------- Helpers ----------
  /** Check if all lines from one side of a conflict region are taken */
  function isHunkAllTaken(side: 'ours' | 'theirs', regionIdx: number): boolean {
    const region = regions[regionIdx];
    if (!region || region.type !== 'conflict') return false;
    const lines = side === 'ours' ? region.oursLines : region.theirsLines;
    if (lines.length === 0) return false;
    return lines.every((_, j) => takenLines.has(`${side}-${regionIdx}-${j}`));
  }
</script>

{#snippet conflictHeader(side: 'ours' | 'theirs', row: FlatRow)}
  <div
    onclick={() => handleToggleHunk(side, row.regionIdx)}
    style="
      width: 100%;
      height: {HEADER_HEIGHT}px;
      background: var(--color-surface);
      border-top: 1px solid var(--color-border);
      border-bottom: 1px solid var(--color-border);
      display: flex;
      align-items: center;
      padding: 0 8px;
      gap: 4px;
      cursor: pointer;
      font-size: 11px;
      color: var(--color-text-muted);
    "
  >
    {#if isHunkAllTaken(side, row.regionIdx)}
      <Check size={14} style="color: var(--color-merge-taken-check);" />
    {:else}
      <span style="width: 14px; height: 14px; display: inline-block;"></span>
    {/if}
    Conflict {row.conflictNum}
  </div>
{/snippet}

{#snippet conflictLine(row: FlatRow, bgColor: string)}
  {@const taken = takenLines.has(row.key)}
  <div
    onclick={(e: MouseEvent) => handleToggleLine(row.key, e)}
    class="merge-line"
    style="
      display: flex;
      height: {LINE_HEIGHT}px;
      background: {bgColor};
      cursor: pointer;
    "
  >
    <span style="
      width: 48px;
      flex-shrink: 0;
      text-align: right;
      padding-right: 8px;
      color: var(--color-text-muted);
      -webkit-user-select: none;
      user-select: none;
    ">{row.lineNum}</span>
    <span style="
      width: 20px;
      flex-shrink: 0;
      display: flex;
      align-items: center;
      justify-content: center;
    " class="icon-gutter">
      {#if taken}
        <span class="taken-icon"><Check size={14} style="color: var(--color-merge-taken-check);" /></span>
        <span class="remove-icon"><CircleX size={14} style="color: var(--color-merge-remove-icon);" /></span>
      {:else}
        <span class="untaken-icon"><CircleCheck size={14} style="color: var(--color-text-muted); opacity: 0.4;" /></span>
      {/if}
    </span>
    <span style="
      padding-left: 4px;
      white-space: pre;
      overflow-x: auto;
      flex: 1;
      min-width: 0;
      color: var(--color-text);
    ">{row.text}</span>
  </div>
{/snippet}

{#snippet contextLine(row: FlatRow)}
  <div style="
    display: flex;
    height: {LINE_HEIGHT}px;
    background: transparent;
  ">
    <span style="
      width: 48px;
      flex-shrink: 0;
      text-align: right;
      padding-right: 8px;
      color: var(--color-text-muted);
      -webkit-user-select: none;
      user-select: none;
    ">{row.lineNum}</span>
    <span style="width: 20px; flex-shrink: 0;"></span>
    <span style="
      padding-left: 4px;
      white-space: pre;
      overflow-x: auto;
      flex: 1;
      min-width: 0;
      color: var(--color-text);
    ">{row.text}</span>
  </div>
{/snippet}

<div style="
  height: 100%;
  display: flex;
  flex-direction: column;
  background: var(--color-bg);
">
  {#if loading}
    <!-- Loading state -->
    <div style="
      flex: 1;
      display: flex;
      align-items: center;
      justify-content: center;
      color: var(--color-text-muted);
      font-size: 13px;
    ">
      Loading merge editor...
    </div>
  {:else if error}
    <!-- Error state -->
    <div style="
      flex: 1;
      display: flex;
      flex-direction: column;
      align-items: center;
      justify-content: center;
      gap: 8px;
      color: var(--color-text-muted);
      font-size: 13px;
    ">
      <span style="color: var(--color-diff-delete);">{error}</span>
      <button
        onclick={() => { loading = true; error = null; safeInvoke<MergeSides>('get_merge_sides', { path: repoPath, filePath }).then((result) => { regions = parseConflictRegions(result.base, result.ours, result.theirs); takenLines = new Set(); manualEdit = false; manualText = ''; focusedConflictIdx = 0; loading = false; }).catch((e) => { error = (e as TrunkError).message ?? 'Failed to load'; loading = false; }); }}
        style="
          background: var(--color-surface);
          border: 1px solid var(--color-border);
          border-radius: 3px;
          color: var(--color-text);
          font-size: 12px;
          padding: 4px 12px;
          cursor: pointer;
        "
      >Retry</button>
    </div>
  {:else}
    <!-- Top row: Current + Incoming side by side (50% height) -->
    <div style="flex: 1; display: flex; min-height: 0;">

      <!-- Current (Ours) Panel -->
      <div style="flex: 1; display: flex; flex-direction: column; min-width: 0; border-right: 1px solid var(--color-border);">
        <!-- Header -->
        <div style="
          height: 28px;
          background: var(--color-merge-current-header);
          border-bottom: 1px solid var(--color-merge-current-border);
          display: flex;
          align-items: center;
          padding: 0 8px;
          gap: 8px;
          flex-shrink: 0;
        ">
          <span style="font-size: 12px; color: var(--color-text);">Current (Ours)</span>
          <span style="flex: 1;"></span>
          <button
            onclick={handleTakeAllCurrent}
            style="
              background: var(--color-btn-continue-bg);
              border: 1px solid var(--color-btn-continue-border);
              border-radius: 3px;
              color: var(--color-btn-continue);
              font-size: 11px;
              font-family: var(--font-sans, sans-serif);
              padding: 2px 8px;
              cursor: pointer;
              white-space: nowrap;
              flex-shrink: 0;
            "
          >Take All Current</button>
        </div>

        <!-- Virtualized scrollable content -->
        <div
          bind:this={panelRefs[0]}
          bind:clientHeight={panelViewportHeight}
          onscroll={() => handleScroll(0)}
          style="
            flex: 1;
            overflow-y: auto;
            font-family: var(--font-mono);
            font-size: 12px;
            line-height: {LINE_HEIGHT}px;
          "
        >
          <div style="height: {oursOffsets[oursVisible[0]]}px;"></div>
          {#each oursFlat.slice(oursVisible[0], oursVisible[1]) as row, idx (oursVisible[0] + idx)}
            {#if row.type === 'padding'}
              <div style="height: {row.height}px;"></div>
            {:else if row.type === 'conflict-header'}
              {@render conflictHeader('ours', row)}
            {:else if row.type === 'conflict-line'}
              {@render conflictLine(row, 'var(--color-diff-add-bg)')}
            {:else}
              {@render contextLine(row)}
            {/if}
          {/each}
          <div style="height: {oursTotalHeight - (oursOffsets[oursVisible[1]] ?? oursTotalHeight)}px;"></div>
        </div>
      </div>

      <!-- Incoming (Theirs) Panel -->
      <div style="flex: 1; display: flex; flex-direction: column; min-width: 0;">
        <!-- Header -->
        <div style="
          height: 28px;
          background: var(--color-merge-incoming-header);
          border-bottom: 1px solid var(--color-merge-incoming-border);
          display: flex;
          align-items: center;
          padding: 0 8px;
          gap: 8px;
          flex-shrink: 0;
        ">
          <span style="font-size: 12px; color: var(--color-text);">Incoming (Theirs)</span>
          <span style="flex: 1;"></span>
          <button
            onclick={handleTakeAllIncoming}
            style="
              background: var(--color-btn-continue-bg);
              border: 1px solid var(--color-btn-continue-border);
              border-radius: 3px;
              color: var(--color-btn-continue);
              font-size: 11px;
              font-family: var(--font-sans, sans-serif);
              padding: 2px 8px;
              cursor: pointer;
              white-space: nowrap;
              flex-shrink: 0;
            "
          >Take All Incoming</button>
        </div>

        <!-- Virtualized scrollable content -->
        <div
          bind:this={panelRefs[1]}
          onscroll={() => handleScroll(1)}
          style="
            flex: 1;
            overflow-y: auto;
            font-family: var(--font-mono);
            font-size: 12px;
            line-height: {LINE_HEIGHT}px;
          "
        >
          <div style="height: {theirsOffsets[theirsVisible[0]]}px;"></div>
          {#each theirsFlat.slice(theirsVisible[0], theirsVisible[1]) as row, idx (theirsVisible[0] + idx)}
            {#if row.type === 'padding'}
              <div style="height: {row.height}px;"></div>
            {:else if row.type === 'conflict-header'}
              {@render conflictHeader('theirs', row)}
            {:else if row.type === 'conflict-line'}
              {@render conflictLine(row, 'var(--color-diff-delete-bg)')}
            {:else}
              {@render contextLine(row)}
            {/if}
          {/each}
          <div style="height: {theirsTotalHeight - (theirsOffsets[theirsVisible[1]] ?? theirsTotalHeight)}px;"></div>
        </div>
      </div>
    </div>

    <!-- Bottom panel: Output (50% height) -->
    <div style="flex: 1; display: flex; flex-direction: column; min-height: 0; border-top: 1px solid var(--color-border);">
      <!-- Header -->
      <div style="
        height: 28px;
        background: var(--color-merge-output-header);
        border-bottom: 1px solid var(--color-merge-output-border);
        display: flex;
        align-items: center;
        padding: 0 8px;
        gap: 8px;
        flex-shrink: 0;
      ">
        <span style="font-size: 12px; color: var(--color-text);">Output</span>
        {#if manualEdit}
          <span style="font-size: 10px; color: var(--color-text-muted);">(manual edit)</span>
        {/if}
        <span style="flex: 1;"></span>

        {#if hasConflicts}
          <!-- Prev conflict -->
          <button
            onclick={handlePrevConflict}
            disabled={!hasPrev}
            aria-label="Previous conflict"
            style="
              background: none;
              border: none;
              cursor: {hasPrev ? 'pointer' : 'default'};
              color: {hasPrev ? 'var(--color-text)' : 'var(--color-text-muted)'};
              opacity: {hasPrev ? 1 : 0.4};
              padding: 2px;
              display: flex;
              align-items: center;
            "
          ><ChevronUp size={16} /></button>

          <!-- Conflict counter -->
          <span style="font-size: 11px; color: var(--color-text-muted); white-space: nowrap;">{focusedConflictIdx + 1}/{conflictIndices.length}</span>

          <!-- Next conflict -->
          <button
            onclick={handleNextConflict}
            disabled={!hasNext}
            aria-label="Next conflict"
            style="
              background: none;
              border: none;
              cursor: {hasNext ? 'pointer' : 'default'};
              color: {hasNext ? 'var(--color-text)' : 'var(--color-text-muted)'};
              opacity: {hasNext ? 1 : 0.4};
              padding: 2px;
              display: flex;
              align-items: center;
            "
          ><ChevronDown size={16} /></button>
        {/if}

        <!-- Save and Mark Resolved -->
        <button
          onclick={handleSaveAndResolve}
          disabled={saving}
          style="
            background: var(--color-btn-continue-bg);
            border: 1px solid var(--color-btn-continue-border);
            border-radius: 3px;
            color: var(--color-btn-continue);
            font-size: 11px;
            font-family: var(--font-sans, sans-serif);
            padding: 2px 8px;
            cursor: {saving ? 'not-allowed' : 'pointer'};
            opacity: {saving ? 0.4 : 1};
            white-space: nowrap;
            flex-shrink: 0;
          "
        >Save and Mark Resolved</button>

        <!-- Close button -->
        <button
          onclick={onclose}
          aria-label="Close merge editor"
          style="
            background: none;
            border: none;
            cursor: pointer;
            color: var(--color-text-muted);
            padding: 2px;
            display: flex;
            align-items: center;
          "
        ><X size={16} /></button>
      </div>

      <!-- Editable output textarea -->
      <textarea
        bind:this={panelRefs[2]}
        value={outputText}
        oninput={handleOutputEdit}
        onscroll={() => handleScroll(2)}
        style="
          flex: 1;
          width: 100%;
          resize: none;
          border: none;
          background: var(--color-bg);
          color: var(--color-text);
          font-family: var(--font-mono);
          font-size: 12px;
          line-height: {LINE_HEIGHT}px;
          padding: 4px 8px;
          outline: none;
          box-sizing: border-box;
        "
      ></textarea>
    </div>
  {/if}
</div>

<style>
  /* Icon hover: show remove icon on taken lines, show dimmed check on untaken */
  .icon-gutter .remove-icon {
    display: none;
  }
  .icon-gutter:hover .taken-icon {
    display: none;
  }
  .icon-gutter:hover .remove-icon {
    display: inline-flex;
  }
  .icon-gutter .untaken-icon {
    display: none;
  }
  .icon-gutter:hover .untaken-icon {
    display: inline-flex;
  }
</style>
