<script lang="ts">
import { FolderTree, List } from "@lucide/svelte";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";
import { copySha } from "../lib/clipboard.js";
import type {
	CommitDetail,
	FileDiff,
	FileStatus,
	FileStatusType,
} from "../lib/types.js";
import TreeFileList from "./TreeFileList.svelte";

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

let {
	commitDetail,
	fileDiffs,
	selectedFile,
	onfileselect,
	onclose,
	repoPath = "",
	treeViewEnabled = false,
	ontreeviewtoggle,
}: Props = $props();

const DIFF_STATUS_MAP: Record<string, FileStatusType> = {
	Added: "New",
	Deleted: "Deleted",
	Modified: "Modified",
	Renamed: "Renamed",
	Copied: "Modified",
	Untracked: "New",
	Unknown: "Modified",
};

let fileStatusList = $derived<FileStatus[]>(
	fileDiffs.map((fd) => ({
		path: fd.path,
		status: DIFF_STATUS_MAP[fd.status] ?? "Modified",
		is_binary: fd.is_binary,
	})),
);

async function showFileContextMenu(e: MouseEvent, filePath: string) {
	e.preventDefault();
	const { Menu, MenuItem } = await import("@tauri-apps/api/menu");
	const absPath = `${repoPath}/${filePath}`;
	const menu = await Menu.new({
		items: [
			await MenuItem.new({
				text: "Copy Relative Path",
				action: () => {
					writeText(filePath).catch(() => {});
				},
			}),
			await MenuItem.new({
				text: "Copy Absolute Path",
				action: () => {
					writeText(absPath).catch(() => {});
				},
			}),
		],
	});
	await menu.popup();
}

let authorDate = $derived(
	new Date(commitDetail.author_timestamp * 1000).toLocaleString(),
);

let parentShort = $derived(
	commitDetail.parent_oids.length > 0
		? commitDetail.parent_oids[0].slice(0, 7)
		: null,
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
      commit: <button type="button" title="Copy SHA" class="sha-copy" onclick={() => copySha(commitDetail.oid)}>{commitDetail.short_oid}</button>
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
        <div>parent: <button type="button" title="Copy SHA" class="sha-copy" onclick={() => copySha(commitDetail.parent_oids[0])}>{parentShort}</button></div>
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

<style>
  /* Click-to-copy SHA: reset the button to read as inline mono text. */
  .sha-copy {
    background: none;
    border: none;
    padding: 0;
    cursor: pointer;
    font-family: monospace;
    font-size: inherit;
    color: inherit;
  }
  .sha-copy:hover {
    text-decoration: underline;
  }
</style>
