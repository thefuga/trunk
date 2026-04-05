<script lang="ts">
import type {
	CommitDetail,
	ContentMode,
	DiffLine,
	DiffOrigin,
	FileDiff,
	LayoutMode,
} from "../../lib/types.js";
import FullFileView from "./FullFileView.svelte";
import HunkView from "./HunkView.svelte";
import SplitView from "./SplitView.svelte";

interface Props {
	contentMode: ContentMode;
	layoutMode: LayoutMode;
	fileDiffs: FileDiff[];
	commitDetail: CommitDetail | null;
	selectedPath: string | null;
	diffKind: "unstaged" | "staged" | "commit";
	loading: boolean;
	hunkOperationInFlight: boolean;
	ignoreWhitespace: boolean;
	showInvisibles: boolean;
	wordWrap: boolean;
	selectedHunkKey: string | null;
	selectedLineIndices: Set<number>;
	selectedCount: number;
	collapsedFiles: Set<string>;
	hunkElements: Record<string, HTMLDivElement>;
	onfilecollapsetoggle: (path: string) => void;
	onlineclick: (
		filePath: string,
		hunkIdx: number,
		lineIndex: number,
		origin: DiffOrigin,
		hunkLines: DiffLine[],
		e: MouseEvent,
	) => void;
	onstagehunk: (filePath: string, hunkIndex: number) => void;
	onunstagehunk: (filePath: string, hunkIndex: number) => void;
	ondiscardhunk: (filePath: string, hunkIndex: number) => void;
	onstagelines: (filePath: string, hunkIndex: number) => void;
	onunstagelines: (filePath: string, hunkIndex: number) => void;
	ondiscardlines: (filePath: string, hunkIndex: number) => void;
}

let {
	contentMode,
	layoutMode,
	fileDiffs,
	commitDetail,
	selectedPath,
	diffKind,
	loading,
	hunkOperationInFlight,
	ignoreWhitespace,
	showInvisibles,
	wordWrap,
	selectedHunkKey,
	selectedLineIndices,
	selectedCount,
	collapsedFiles,
	hunkElements,
	onfilecollapsetoggle,
	onlineclick,
	onstagehunk,
	onunstagehunk,
	ondiscardhunk,
	onstagelines,
	onunstagelines,
	ondiscardlines,
}: Props = $props();
</script>

<div style="flex: 1; overflow: auto; min-height: 0; container-type: inline-size; overscroll-behavior-x: none;">
  {#if fileDiffs.length === 0 && commitDetail === null && !loading}
    <div style="
      flex: 1;
      display: flex;
      align-items: center;
      justify-content: center;
      color: var(--color-text-muted);
      font-size: 13px;
    ">
      Select a file or commit to view its diff
    </div>
  {:else if layoutMode === "inline" && contentMode === "hunk"}
    <HunkView
      {fileDiffs}
      {selectedPath}
      {diffKind}
      {hunkOperationInFlight}
      {ignoreWhitespace}
      {showInvisibles}
      {wordWrap}
      {selectedHunkKey}
      {selectedLineIndices}
      {selectedCount}
      {collapsedFiles}
      {hunkElements}
      {onfilecollapsetoggle}
      {onlineclick}
      onstagehunk={onstagehunk}
      onunstagehunk={onunstagehunk}
      ondiscardhunk={ondiscardhunk}
      onstagelines={onstagelines}
      onunstagelines={onunstagelines}
      ondiscardlines={ondiscardlines}
    />
  {:else if layoutMode === "inline" && contentMode === "full"}
    <FullFileView {fileDiffs} {showInvisibles} {wordWrap} />
  {:else}
    <SplitView {contentMode} {fileDiffs} {selectedPath} {diffKind}
      {hunkOperationInFlight} {ignoreWhitespace} {showInvisibles} {wordWrap}
      {selectedHunkKey} {selectedLineIndices} {selectedCount}
      {collapsedFiles} {hunkElements}
      {onfilecollapsetoggle} {onlineclick}
      onstagehunk={onstagehunk} onunstagehunk={onunstagehunk} ondiscardhunk={ondiscardhunk}
      onstagelines={onstagelines} onunstagelines={onunstagelines} ondiscardlines={ondiscardlines} />
  {/if}
</div>
