<script lang="ts">
  import { ArrowDown, ArrowUp, Laptop, Globe, Tag } from '@lucide/svelte';

  interface Props {
    name: string;
    kind?: 'local' | 'remote' | 'tag';
    isHead?: boolean;
    isLoading?: boolean;
    isError?: boolean;
    errorText?: string;
    ahead?: number;
    behind?: number;
    onclick?: () => void;
    oncontextmenu?: (e: MouseEvent) => void;
  }

  let {
    name,
    kind = 'local',
    isHead = false,
    isLoading = false,
    isError = false,
    errorText,
    ahead = 0,
    behind = 0,
    onclick,
    oncontextmenu,
  }: Props = $props();

  const kindIcon = { local: Laptop, remote: Globe, tag: Tag } as const;
  const KindIcon = $derived(kindIcon[kind]);

  let hovered = $state(false);
</script>

<div>
  <div
    role="button"
    tabindex="0"
    onclick={() => onclick?.()}
    onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') onclick?.(); }}
    oncontextmenu={(e) => { if (oncontextmenu) { e.preventDefault(); oncontextmenu(e); } }}
    onmouseenter={() => (hovered = true)}
    onmouseleave={() => (hovered = false)}
    style="
      height: 26px;
      padding: 0 12px;
      display: flex;
      align-items: center;
      overflow: hidden;
      cursor: pointer;
      background: {hovered ? 'var(--color-surface)' : 'transparent'};
      color: {isHead ? 'var(--color-accent)' : isLoading ? 'var(--color-text-muted)' : 'var(--color-text)'};
      font-weight: {isHead ? '600' : 'normal'};
      font-size: 13px;
    "
  >
    <span style="flex-shrink: 0; display: inline-flex; align-items: center; margin-right: 6px; color: var(--color-text-muted);">
      <KindIcon size={12} />
    </span>
    <span style="
      display: block;
      overflow: hidden;
      white-space: nowrap;
      text-overflow: ellipsis;
      min-width: 0;
      flex: 1;
    ">{name}{isLoading ? ' …' : ''}</span>
    {#if behind > 0 || ahead > 0}
      <span style="flex-shrink: 0; font-size: 11px; color: var(--color-text-muted); margin-left: 4px; display: inline-flex; align-items: center; gap: 2px;">
        {#if behind > 0}<span style="display: inline-flex; align-items: center;"><ArrowDown size={11} />{behind}</span>{/if}
        {#if ahead > 0}<span style="display: inline-flex; align-items: center; margin-left: 2px;"><ArrowUp size={11} />{ahead}</span>{/if}
      </span>
    {/if}
  </div>

  {#if isError}
    <div style="background: #3d1c1c; border: 1px solid #6b2a2a; color: #f87171; font-size: 11px; padding: 6px 10px; margin: 0 8px 4px; border-radius: 3px;">
      {errorText ?? 'Cannot checkout — working tree has uncommitted changes. Commit or stash your changes first.'}
    </div>
  {/if}
</div>
