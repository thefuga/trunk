<script lang="ts">
  import type { FileDiff, CommitDetail, DiffStatus, FileStatus, FileStatusType } from '../lib/types.js';
  import { writeText } from '@tauri-apps/plugin-clipboard-manager';
  import TreeFileList from './TreeFileList.svelte';
  import { List, FolderTree } from '@lucide/svelte';

  const STATUS_ICONS: Record<DiffStatus, { symbol: string; color: string }> = {
    Added:     { symbol: '+',  color: '#4ade80' },
    Deleted:   { symbol: '−',  color: '#f87171' },
    Modified:  { symbol: '✎',  color: '#fb923c' },
    Renamed:   { symbol: '→',  color: '#60a5fa' },
    Copied:    { symbol: '⎘',  color: '#c084fc' },
    Untracked: { symbol: '?',  color: '#facc15' },
    Unknown:   { symbol: '?',  color: 'var(--color-text-muted)' },
  };

  interface Props {
    commitDetail: CommitDetail;
    fileDiffs: FileDiff[];
    selectedFile: string | null;
    onfileselect: (path: string) => void;
    onclose: () => void;
    repoPath?: string;
    treeViewEnabled?: boolean;
    ontreeviewtoggle?: () => void;
  }

  let { commitDetail, fileDiffs, selectedFile, onfileselect, onclose, repoPath = '', treeViewEnabled = false, ontreeviewtoggle }: Props = $props();

  const DIFF_STATUS_MAP: Record<string, FileStatusType> = {
    Added: 'New',
    Deleted: 'Deleted',
    Modified: 'Modified',
    Renamed: 'Renamed',
    Copied: 'Modified',
    Untracked: 'New',
    Unknown: 'Modified',
  };

  let fileStatusList = $derived<FileStatus[]>(
    fileDiffs.map(fd => ({
      path: fd.path,
      status: DIFF_STATUS_MAP[fd.status] ?? 'Modified',
      is_binary: fd.is_binary,
    }))
  );

  async function showFileContextMenu(e: MouseEvent, filePath: string) {
    e.preventDefault();
    const { Menu, MenuItem } = await import('@tauri-apps/api/menu');
    const absPath = repoPath + '/' + filePath;
    const menu = await Menu.new({
      items: [
        await MenuItem.new({ text: 'Copy Relative Path', action: () => { writeText(filePath).catch(() => {}); } }),
        await MenuItem.new({ text: 'Copy Absolute Path', action: () => { writeText(absPath).catch(() => {}); } }),
      ],
    });
    await menu.popup();
  }

  let authorDate = $derived(
    new Date(commitDetail.author_timestamp * 1000).toLocaleString()
  );

  let parentShort = $derived(
    commitDetail.parent_oids.length > 0 ? commitDetail.parent_oids[0].slice(0, 7) : null
  );
</script>

<div style="
  width: 100%;
  min-width: 0;
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
">

  <!-- Toolbar -->
  <div style="
    height: 24px;
    border-bottom: 1px solid var(--color-border);
    padding: 0 8px;
    display: flex;
    align-items: center;
    gap: 8px;
    flex-shrink: 0;
  ">
    <span style="
      font-size: 11px;
      color: var(--color-text-muted);
      font-family: monospace;
      flex: 1;
      overflow: hidden;
      text-overflow: ellipsis;
      white-space: nowrap;
    ">
      commit: {commitDetail.short_oid}
    </span>
    <button
      onclick={onclose}
      aria-label="Close commit detail"
      style="
        background: none;
        border: none;
        cursor: pointer;
        color: var(--color-text-muted);
        font-size: 16px;
        line-height: 1;
        padding: 2px 4px;
        border-radius: 3px;
        flex-shrink: 0;
      "
    >✕</button>
  </div>

  <!-- Scrollable content -->
  <div style="flex: 1; overflow-y: auto; min-height: 0;">

    <!-- Commit message -->
    <div style="
      padding: 10px 12px;
      border-bottom: 1px solid var(--color-border);
    ">
      <div style="
        font-size: 13px;
        font-weight: 600;
        color: var(--color-text);
        line-height: 1.4;
        margin-bottom: {commitDetail.body ? '6px' : '0'};
      ">
        {commitDetail.summary}
      </div>
      {#if commitDetail.body}
        <div style="
          font-size: 12px;
          color: var(--color-text-muted);
          white-space: pre-wrap;
          line-height: 1.5;
          margin-top: 4px;
        ">
          {commitDetail.body}
        </div>
      {/if}
    </div>

    <!-- Author + parent -->
    <div style="
      padding: 8px 12px;
      border-bottom: 1px solid var(--color-border);
      font-size: 11px;
      color: var(--color-text-muted);
    ">
      <div style="margin-bottom: 2px; color: var(--color-text);">
        {commitDetail.author_name}
        <span style="color: var(--color-text-muted);">&lt;{commitDetail.author_email}&gt;</span>
      </div>
      <div style="margin-bottom: {parentShort ? '2px' : '0'};">{authorDate}</div>
      {#if parentShort}
        <div>parent: <span style="font-family: monospace;">{parentShort}</span></div>
      {/if}
    </div>

    <!-- File list -->
    <div>
      <div style="
        height: 28px;
        padding: 0 12px;
        display: flex;
        align-items: center;
        border-bottom: 1px solid var(--color-border);
        flex-shrink: 0;
      ">
        <span style="font-size: 12px; font-weight: 500; color: var(--color-text); flex: 1;">
          {fileDiffs.length} file{fileDiffs.length === 1 ? '' : 's'} changed
        </span>
        {#if ontreeviewtoggle}
          <button
            role="switch"
            aria-checked={treeViewEnabled}
            aria-label={treeViewEnabled ? 'Switch to list view' : 'Switch to tree view'}
            title={treeViewEnabled ? 'List view' : 'Tree view'}
            onclick={(e) => { e.stopPropagation(); ontreeviewtoggle?.(); }}
            style="
              background: none;
              border: none;
              cursor: pointer;
              color: var(--color-text-muted);
              display: flex;
              align-items: center;
              justify-content: center;
              width: 20px;
              height: 20px;
              border-radius: 3px;
              flex-shrink: 0;
              padding: 0;
            "
          >
            {#if treeViewEnabled}
              <FolderTree size={14} />
            {:else}
              <List size={14} />
            {/if}
          </button>
        {/if}
      </div>
      <TreeFileList
        files={fileStatusList}
        treeMode={treeViewEnabled}
        actionLabel=""
        onfileaction={() => {}}
        onfileclick={(path) => onfileselect(path)}
        onfilecontextmenu={(e, path) => showFileContextMenu(e, path)}
      />
    </div>

  </div>
</div>
