<script lang="ts">
import { safeInvoke, type TrunkError } from "../lib/invoke.js";
import { showToast } from "../lib/toast.svelte.js";
import type { RefsResponse, StashEntry } from "../lib/types.js";
import BranchRow from "./BranchRow.svelte";
import BranchSection from "./BranchSection.svelte";
import InputDialog from "./InputDialog.svelte";
import RemoteGroup from "./RemoteGroup.svelte";

interface Props {
	repoPath: string;
	onrefreshed?: () => void;
	onstashselect?: (oid: string) => void;
	onrefnavigate?: (refNameOrOid: string) => void;
	refreshSignal?: number;
	onopenrebaseeditor?: (baseOid: string, inclusive?: boolean) => void;
}

let {
	repoPath,
	onrefreshed,
	onstashselect,
	onrefnavigate,
	refreshSignal,
	onopenrebaseeditor,
}: Props = $props();

let refs = $state<RefsResponse | null>(null);
let loading = $state(false);
let loadSeq = 0;
let search = $state("");
let checkingOutBranch = $state<string | null>(null);
let checkoutError = $state<{ branch: string; message: string } | null>(null);
let localExpanded = $state(true);
let remoteExpanded = $state(false);
let tagsExpanded = $state(false);
let stashesExpanded = $state(false);
let showStashForm = $state(false);
let stashName = $state("");
let stashSaving = $state(false);
let stashCreateError = $state<string | null>(null);
let stashEntryErrors = $state<Record<number, string | null>>({});
let showCreateInput = $state(false);
let newBranchName = $state("");
let createError = $state<string | null>(null);

let filteredLocal = $derived(
	search
		? (refs?.local ?? []).filter((b) =>
				b.name.toLowerCase().includes(search.toLowerCase()),
			)
		: (refs?.local ?? []),
);

let filteredRemote = $derived(
	search
		? (refs?.remote ?? []).filter((b) =>
				b.name.toLowerCase().includes(search.toLowerCase()),
			)
		: (refs?.remote ?? []),
);

let filteredTags = $derived(
	search
		? (refs?.tags ?? []).filter((t) =>
				t.short_name.toLowerCase().includes(search.toLowerCase()),
			)
		: (refs?.tags ?? []),
);

let filteredStashes = $derived<StashEntry[]>(
	search
		? (refs?.stashes ?? []).filter((s) =>
				s.name.toLowerCase().includes(search.toLowerCase()),
			)
		: (refs?.stashes ?? []),
);

// Group remote branches by remote name: { "origin": ["main", "dev"] }
let remoteGroups = $derived(
	filteredRemote.reduce<Record<string, string[]>>((acc, b) => {
		const slash = b.name.indexOf("/");
		const remote = slash >= 0 ? b.name.slice(0, slash) : "unknown";
		const short = slash >= 0 ? b.name.slice(slash + 1) : b.name;
		if (!acc[remote]) acc[remote] = [];
		acc[remote].push(short);
		return acc;
	}, {}),
);

// Load refs on mount and when repoPath changes
$effect(() => {
	const path = repoPath;
	loadRefs(path);
});

// Reload refs when parent signals a refresh (e.g. context menu actions)
$effect(() => {
	if (refreshSignal !== undefined && refreshSignal > 0) {
		loadRefs(repoPath);
	}
});

// Dismiss error when search changes
$effect(() => {
	if (search) checkoutError = null;
});

async function loadRefs(path: string) {
	const seq = ++loadSeq;
	loading = true;
	try {
		const result = await safeInvoke<RefsResponse>("list_refs", { path });
		if (seq === loadSeq) {
			refs = result;
		}
	} catch {
		if (seq === loadSeq) {
			refs = null;
		}
	} finally {
		if (seq === loadSeq) {
			loading = false;
		}
	}
}

async function handleCheckout(branchName: string) {
	// Dismiss any existing error first
	checkoutError = null;
	checkingOutBranch = branchName;
	try {
		await safeInvoke<void>("checkout_branch", { path: repoPath, branchName });
		await loadRefs(repoPath);
		onrefreshed?.();
		showToast(`Checked out ${branchName}`, "success");
	} catch (e) {
		const err = e as TrunkError;
		if (err.code === "dirty_workdir") {
			checkoutError = {
				branch: branchName,
				message:
					"Cannot checkout — working tree has uncommitted changes. Commit or stash your changes first.",
			};
		}
		showToast("Checkout failed", "error");
	} finally {
		checkingOutBranch = null;
	}
}

async function handleCheckoutRemoteBranch(fullName: string) {
	const shortName = fullName.slice(fullName.indexOf("/") + 1);
	checkoutError = null;
	checkingOutBranch = fullName;
	try {
		await safeInvoke<void>("create_branch", {
			path: repoPath,
			name: shortName,
			fromOid: fullName,
		});
		await loadRefs(repoPath);
		onrefreshed?.();
	} catch (e) {
		showToast((e as TrunkError).message ?? "Checkout failed", "error");
	} finally {
		checkingOutBranch = null;
	}
}

async function handleCreateBranch() {
	const trimmed = newBranchName.trim();
	if (!trimmed) return;
	createError = null;
	try {
		await safeInvoke<void>("create_branch", { path: repoPath, name: trimmed });
		showCreateInput = false;
		newBranchName = "";
		await loadRefs(repoPath);
		onrefreshed?.();
		showToast(`Checked out ${trimmed}`, "success");
	} catch (e) {
		const err = e as TrunkError;
		if (err.code === "dirty_workdir") {
			showToast(
				"Branch created (checkout skipped — uncommitted changes)",
				"success",
			);
			showCreateInput = false;
			newBranchName = "";
			await loadRefs(repoPath);
			onrefreshed?.();
		} else {
			createError = err.message;
		}
	}
}

function autoFocus(node: HTMLElement) {
	node.focus();
	return {};
}

async function handleStashSave() {
	stashSaving = true;
	stashCreateError = null;
	try {
		await safeInvoke("stash_save", {
			path: repoPath,
			message: stashName.trim(),
		});
		showStashForm = false;
		stashName = "";
		await loadRefs(repoPath);
	} catch (e) {
		const err = e as TrunkError;
		if (err.code === "nothing_to_stash") {
			stashCreateError = "Nothing to stash — working tree is clean";
		} else {
			stashCreateError = err.message ?? "Failed to create stash";
		}
	} finally {
		stashSaving = false;
	}
}

async function showStashEntryMenu(e: MouseEvent, stashIndex: number) {
	e.preventDefault();
	const { Menu, MenuItem } = await import("@tauri-apps/api/menu");
	const menu = await Menu.new({
		items: [
			await MenuItem.new({
				text: "Pop",
				action: () => {
					handleStashPop(stashIndex).catch(() => {});
				},
			}),
			await MenuItem.new({
				text: "Apply",
				action: () => {
					handleStashApply(stashIndex).catch(() => {});
				},
			}),
			await MenuItem.new({
				text: "Drop",
				action: () => {
					handleStashDrop(stashIndex).catch(() => {});
				},
			}),
		],
	});
	await menu.popup();
}

async function handleStashPop(index: number) {
	stashEntryErrors = { ...stashEntryErrors, [index]: null };
	try {
		await safeInvoke("stash_pop", { path: repoPath, index });
		await loadRefs(repoPath);
	} catch (e) {
		const err = e as TrunkError;
		stashEntryErrors = {
			...stashEntryErrors,
			[index]: err.message ?? "Failed to pop stash",
		};
	}
}

async function handleStashApply(index: number) {
	stashEntryErrors = { ...stashEntryErrors, [index]: null };
	try {
		await safeInvoke("stash_apply", { path: repoPath, index });
		await loadRefs(repoPath);
	} catch (e) {
		const err = e as TrunkError;
		stashEntryErrors = {
			...stashEntryErrors,
			[index]: err.message ?? "Failed to apply stash",
		};
	}
}

async function handleStashDrop(index: number) {
	const { ask } = await import("@tauri-apps/plugin-dialog");
	const confirmed = await ask(`Drop stash@{${index}}? This cannot be undone.`, {
		title: "Confirm Drop",
		kind: "warning",
	});
	if (!confirmed) return;
	stashEntryErrors = { ...stashEntryErrors, [index]: null };
	try {
		await safeInvoke("stash_drop", { path: repoPath, index });
		await loadRefs(repoPath);
	} catch (e) {
		const err = e as TrunkError;
		stashEntryErrors = {
			...stashEntryErrors,
			[index]: err.message ?? "Failed to drop stash",
		};
	}
}

// --- Branch/Tag context menu support ---

interface DialogConfig {
	title: string;
	fields: {
		key: string;
		label: string;
		placeholder?: string;
		required?: boolean;
		defaultValue?: string;
	}[];
	onsubmit: (values: Record<string, string>) => void;
}
let dialogConfig = $state<DialogConfig | null>(null);
function closeDialog() {
	dialogConfig = null;
}

async function handleDeleteBranch(branchName: string) {
	const { ask } = await import("@tauri-apps/plugin-dialog");
	const confirmed = await ask(
		`Delete branch '${branchName}'? This cannot be undone.`,
		{
			title: "Delete Branch",
			kind: "warning",
		},
	);
	if (!confirmed) return;
	try {
		await safeInvoke("delete_branch", { path: repoPath, branchName });
		await loadRefs(repoPath);
		onrefreshed?.();
		showToast(`Deleted branch ${branchName}`, "success");
	} catch (e) {
		const err = e as TrunkError;
		showToast(err.message ?? "Failed to delete branch", "error");
	}
}

function handleRenameBranch(branchName: string) {
	dialogConfig = {
		title: "Rename Branch",
		fields: [
			{
				key: "name",
				label: "New name",
				required: true,
				defaultValue: branchName,
			},
		],
		onsubmit: async (values) => {
			closeDialog();
			const newName = values.name.trim();
			if (!newName || newName === branchName) return;
			try {
				await safeInvoke("rename_branch", {
					path: repoPath,
					oldName: branchName,
					newName,
				});
				await loadRefs(repoPath);
				onrefreshed?.();
				showToast(`Renamed branch to ${newName}`, "success");
			} catch (e) {
				const err = e as TrunkError;
				showToast(err.message ?? "Failed to rename branch", "error");
			}
		},
	};
}

async function handleDeleteTag(tagName: string) {
	const { ask } = await import("@tauri-apps/plugin-dialog");
	const confirmed = await ask(
		`Delete tag '${tagName}'? This cannot be undone.`,
		{
			title: "Delete Tag",
			kind: "warning",
		},
	);
	if (!confirmed) return;
	try {
		await safeInvoke("delete_tag", { path: repoPath, tagName });
		await loadRefs(repoPath);
		onrefreshed?.();
		showToast(`Deleted tag ${tagName}`, "success");
	} catch (e) {
		const err = e as TrunkError;
		showToast(err.message ?? "Failed to delete tag", "error");
	}
}

async function handleMergeBranch(branch: string) {
	try {
		await safeInvoke("merge_branch", { path: repoPath, branch });
		// No toast on success -- graph refresh via repo-changed event is sufficient
		await loadRefs(repoPath);
		onrefreshed?.();
	} catch (e) {
		const err = e as TrunkError;
		showToast(err.message ?? "Merge failed", "error");
	}
}

async function handleRebaseBranch(ontoBranch: string) {
	try {
		await safeInvoke("rebase_branch", { path: repoPath, ontoBranch });
		// No toast on success -- graph refresh via repo-changed event is sufficient
		await loadRefs(repoPath);
		onrefreshed?.();
	} catch (e) {
		const err = e as TrunkError;
		showToast(err.message ?? "Rebase failed", "error");
	}
}

async function handleInteractiveRebase(branchName: string) {
	try {
		const forkPoint = await safeInvoke<string>("get_fork_point", {
			path: repoPath,
			branch: branchName,
		});
		onopenrebaseeditor?.(forkPoint);
	} catch (e) {
		const err = e as TrunkError;
		showToast(err.message ?? "Failed to detect fork point", "error");
	}
}

async function handleDeleteRemoteBranch(fullRefName: string) {
	const { ask } = await import("@tauri-apps/plugin-dialog");
	const confirmed = await ask(
		`Delete remote branch '${fullRefName}'? This will remove it from the remote.`,
		{ title: "Delete Remote Branch", kind: "warning" },
	);
	if (!confirmed) return;
	try {
		await safeInvoke("delete_remote_branch", {
			path: repoPath,
			branchName: fullRefName,
		});
		await loadRefs(repoPath);
		onrefreshed?.();
		showToast(`Deleted remote branch ${fullRefName}`, "success");
	} catch (e) {
		const err = e as TrunkError;
		showToast(err.message ?? "Failed to delete remote branch", "error");
	}
}

async function showBranchContextMenu(
	_e: MouseEvent,
	branchName: string,
	isHead: boolean,
) {
	const { Menu, MenuItem, PredefinedMenuItem } = await import(
		"@tauri-apps/api/menu"
	);
	const headBranchName = refs?.local.find((b) => b.is_head)?.name;
	const menu = await Menu.new({
		items: [
			await MenuItem.new({
				text: "Checkout",
				enabled: !isHead,
				action: () => {
					handleCheckout(branchName);
				},
			}),
			...(!isHead && headBranchName
				? [
						await MenuItem.new({
							text: `Merge ${branchName} into ${headBranchName}`,
							action: () => {
								handleMergeBranch(branchName).catch(() => {});
							},
						}),
						await MenuItem.new({
							text: `Rebase ${headBranchName} onto ${branchName}`,
							action: () => {
								handleRebaseBranch(branchName).catch(() => {});
							},
						}),
						await MenuItem.new({
							text: `Interactive Rebase ${branchName}...`,
							action: () => {
								handleInteractiveRebase(branchName).catch(() => {});
							},
						}),
					]
				: []),
			await PredefinedMenuItem.new({ item: "Separator" }),
			await MenuItem.new({
				text: "Rename…",
				action: () => {
					handleRenameBranch(branchName);
				},
			}),
			await MenuItem.new({
				text: "Delete",
				enabled: !isHead,
				action: () => {
					handleDeleteBranch(branchName).catch(() => {});
				},
			}),
		],
	});
	await menu.popup();
}

async function showTagContextMenu(_e: MouseEvent, tagShortName: string) {
	const { Menu, MenuItem } = await import("@tauri-apps/api/menu");
	const menu = await Menu.new({
		items: [
			await MenuItem.new({
				text: "Delete",
				action: () => {
					handleDeleteTag(tagShortName).catch(() => {});
				},
			}),
		],
	});
	await menu.popup();
}

async function showRemoteContextMenu(_e: MouseEvent, fullRefName: string) {
	const { Menu, MenuItem, PredefinedMenuItem } = await import(
		"@tauri-apps/api/menu"
	);
	const headBranchName = refs?.local.find((b) => b.is_head)?.name;
	const menu = await Menu.new({
		items: [
			...(headBranchName
				? [
						await MenuItem.new({
							text: `Merge ${fullRefName} into ${headBranchName}`,
							action: () => {
								handleMergeBranch(fullRefName).catch(() => {});
							},
						}),
						await MenuItem.new({
							text: `Rebase ${headBranchName} onto ${fullRefName}`,
							action: () => {
								handleRebaseBranch(fullRefName).catch(() => {});
							},
						}),
						await MenuItem.new({
							text: `Interactive Rebase ${fullRefName}...`,
							action: () => {
								handleInteractiveRebase(fullRefName).catch(() => {});
							},
						}),
						await PredefinedMenuItem.new({ item: "Separator" }),
					]
				: []),
			await MenuItem.new({
				text: "Delete",
				action: () => {
					handleDeleteRemoteBranch(fullRefName).catch(() => {});
				},
			}),
		],
	});
	await menu.popup();
}
</script>

<aside data-testid="branch-sidebar" style="
  width: 100%;
  min-width: 0;
  background: var(--color-bg);
  display: flex;
  flex-direction: column;
  overflow: hidden;
">
  <!-- Search input (sticky at top) -->
  <div style="padding: 6px 8px; border-bottom: 1px solid var(--color-border);">
    <input
      type="text"
      placeholder="Filter branches…"
      bind:value={search}
      style="
        width: 100%;
        box-sizing: border-box;
        background: var(--color-surface);
        border: 1px solid var(--color-border);
        color: var(--color-text);
        font-size: 12px;
        padding: 4px 8px;
        border-radius: 4px;
        outline: none;
      "
    />
  </div>

  <!-- Sections (scrollable) -->
  <div style="flex: 1; overflow-y: auto;">
    <!-- Local branches (expanded by default, show + button) -->
    {#if loading || filteredLocal.length > 0 || (refs?.local.length ?? 0) > 0}
      <BranchSection
        label="Local"
        count={refs?.local.length ?? 0}
        expanded={localExpanded}
        ontoggle={() => (localExpanded = !localExpanded)}
        showCreateButton={true}
        oncreate={() => { showCreateInput = true; }}
      >
        {#if showCreateInput}
          <div style="padding: 2px 8px 4px;">
            <input
              data-testid="branch-create-input"
              type="text"
              placeholder="New branch name"
              bind:value={newBranchName}
              use:autoFocus
              style="
                width: 100%;
                box-sizing: border-box;
                background: var(--color-surface);
                border: 1px solid var(--color-accent);
                color: var(--color-text);
                font-size: 12px;
                padding: 2px 6px;
                height: 26px;
                border-radius: 3px;
                outline: none;
              "
              onkeydown={(e) => {
                if (e.key === 'Enter') handleCreateBranch();
                if (e.key === 'Escape') { showCreateInput = false; newBranchName = ''; createError = null; }
              }}
            />
            {#if createError}
              <div style="color: #f87171; font-size: 11px; margin-top: 2px;">{createError}</div>
            {/if}
          </div>
        {/if}
        {#each filteredLocal as branch (branch.name)}
          <BranchRow
            name={branch.name}
            kind="local"
            isHead={branch.is_head}
            isLoading={checkingOutBranch === branch.name}
            isError={checkoutError?.branch === branch.name}
            errorText={checkoutError?.message}
            ahead={branch.ahead}
            behind={branch.behind}
            onclick={() => onrefnavigate?.(branch.name)}
            ondblclick={() => handleCheckout(branch.name)}
            oncontextmenu={(e) => showBranchContextMenu(e, branch.name, branch.is_head)}
          />
        {/each}
      </BranchSection>
    {/if}

    <!-- Remote branches (collapsed by default, grouped by remote) -->
    {#if (refs?.remote.length ?? 0) > 0}
      <BranchSection
        label="Remote"
        count={refs?.remote.length ?? 0}
        expanded={remoteExpanded}
        ontoggle={() => (remoteExpanded = !remoteExpanded)}
      >
        {#each Object.entries(remoteGroups) as [remoteName, branches] (remoteName)}
          <RemoteGroup
            {remoteName}
            {branches}
            checkingOut={checkingOutBranch}
            errorBranch={checkoutError?.branch ?? null}
            errorText={checkoutError?.message ?? ''}
            oncheckout={(fullName) => onrefnavigate?.(fullName)}
            ondblclick={handleCheckoutRemoteBranch}
            oncontextmenu={(e, fullName) => showRemoteContextMenu(e, fullName)}
          />
        {/each}
      </BranchSection>
    {/if}

    <!-- Tags (collapsed by default; hidden if empty) -->
    {#if (refs?.tags.length ?? 0) > 0}
      <BranchSection
        label="Tags"
        count={refs?.tags.length ?? 0}
        expanded={tagsExpanded}
        ontoggle={() => (tagsExpanded = !tagsExpanded)}
      >
        {#each filteredTags as tag (tag.name)}
          <BranchRow
            name={tag.short_name}
            kind="tag"
            onclick={() => onrefnavigate?.(tag.short_name)}
            oncontextmenu={(e) => showTagContextMenu(e, tag.short_name)}
          />
        {/each}
      </BranchSection>
    {/if}

    <!-- Stashes — always visible so '+' button is accessible -->
    <BranchSection
      label="Stashes"
      count={filteredStashes.length}
      expanded={stashesExpanded}
      ontoggle={() => (stashesExpanded = !stashesExpanded)}
      showCreateButton={true}
      oncreate={() => { showStashForm = !showStashForm; stashCreateError = null; stashName = ''; stashesExpanded = true; }}
    >
      <!-- Inline create form -->
      {#if showStashForm}
        <div class="stash-form">
          <input
            type="text"
            placeholder="Stash name (optional)"
            bind:value={stashName}
            onkeydown={(e) => e.key === 'Enter' && handleStashSave()}
            disabled={stashSaving}
            class="stash-name-input"
          />
          <button
            onclick={handleStashSave}
            disabled={stashSaving}
            class="stash-save-btn"
          >{stashSaving ? 'Stashing…' : 'Stash'}</button>
        </div>
        {#if stashCreateError}
          <p class="stash-error">{stashCreateError}</p>
        {/if}
      {/if}

      <!-- Stash list entries -->
      {#each filteredStashes as stash (stash.index)}
        <div
          class="stash-row"
          role="button"
          tabindex="0"
          onclick={() => onrefnavigate?.(stash.oid)}
          onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); onrefnavigate?.(stash.oid); } }}
          oncontextmenu={(e) => showStashEntryMenu(e, stash.index)}
        >
          <span class="stash-index">{stash.short_name}</span>
          <span class="stash-message">{stash.name}</span>
        </div>
        {#if stashEntryErrors[stash.index]}
          <p class="stash-error stash-entry-error">{stashEntryErrors[stash.index]}</p>
        {/if}
      {/each}
    </BranchSection>
  </div>

  {#if dialogConfig}
    <InputDialog
      title={dialogConfig.title}
      fields={dialogConfig.fields}
      onsubmit={dialogConfig.onsubmit}
      oncancel={closeDialog}
    />
  {/if}
</aside>

<style>
  .stash-form {
    display: flex;
    gap: 4px;
    padding: 4px 8px;
  }

  .stash-name-input {
    flex: 1;
    font-size: 12px;
    padding: 2px 6px;
    background: var(--color-input-bg, #1a1a1a);
    border: 1px solid var(--color-border, #333);
    color: var(--color-text);
    border-radius: 3px;
  }

  .stash-save-btn {
    font-size: 11px;
    padding: 2px 8px;
    cursor: pointer;
    background: var(--color-accent, #0d7a5f);
    color: white;
    border: none;
    border-radius: 3px;
  }

  .stash-row {
    display: flex;
    gap: 8px;
    padding: 4px 12px;
    font-size: 12px;
    cursor: default;
  }

  .stash-row:hover {
    background: var(--color-hover);
  }

  .stash-index {
    color: var(--color-text-muted, #888);
    flex-shrink: 0;
  }

  .stash-message {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    color: var(--color-text);
  }

  .stash-error {
    font-size: 11px;
    color: var(--color-error, #e05252);
    padding: 2px 12px 4px;
    margin: 0;
  }

  .stash-entry-error {
    padding-left: 24px;
  }
</style>
