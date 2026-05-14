import type { RecentRepo } from "./store.js";

/**
 * Case-insensitive substring filter for the recents picker.
 *
 * Empty or whitespace-only queries pass the input through unchanged.
 * Otherwise an entry is kept iff its name OR path contains the lowercased
 * query as a substring. Order is preserved — this is a filter, not a sort.
 */
export function filterRecents(
	repos: RecentRepo[],
	query: string,
): RecentRepo[] {
	const trimmed = query.trim();
	if (trimmed === "") return repos;
	const q = trimmed.toLowerCase();
	return repos.filter(
		(r) => r.name.toLowerCase().includes(q) || r.path.toLowerCase().includes(q),
	);
}
