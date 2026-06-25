import type { RecentRepo } from "./store.js";

/**
 * Case-insensitive substring filter for the recents picker.
 *
 * Empty or whitespace-only queries pass the input through unchanged.
 * Otherwise an entry is kept iff its name, legacy path, or descriptor display
 * path contains the lowercased query as a substring. Order is preserved — this
 * is a filter, not a sort.
 */
export function filterRecents(
	repos: RecentRepo[],
	query: string,
): RecentRepo[] {
	const trimmed = query.trim();
	if (trimmed === "") return repos;
	const q = trimmed.toLowerCase();
	return repos.filter((r) => {
		const displayPath = r.repoDescriptor?.display_path ?? r.path;
		return (
			r.name.toLowerCase().includes(q) ||
			r.path.toLowerCase().includes(q) ||
			displayPath.toLowerCase().includes(q)
		);
	});
}
