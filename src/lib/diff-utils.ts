/**
 * Represents a segment of text for invisible character rendering.
 * When showInvisibles is active, space/tab characters are split into
 * separate segments with substitution characters.
 */
export interface InvisibleSegment {
	text: string;
	isInvisible: boolean;
	isTrailing: boolean;
}

/**
 * Detects the index where trailing whitespace begins in a string.
 * Returns the string length if there is no trailing whitespace.
 */
export function trailingWhitespaceStart(text: string): number {
	let i = text.length;
	while (i > 0 && (text[i - 1] === " " || text[i - 1] === "\t")) {
		i--;
	}
	return i;
}

/**
 * Splits a text segment into invisible/visible sub-segments.
 * Spaces are replaced with middle dot (U+00B7), tabs with rightwards arrow (U+2192).
 * Only spaces and tabs are handled -- no line ending markers.
 *
 * CRITICAL: This function must be called AFTER slicing line.content by span offsets.
 * Never call it before slicing -- that would break byte offset alignment.
 *
 * @param text - Already-sliced text segment
 * @param isTrailingRegion - Whether this segment falls within trailing whitespace
 */
export function splitInvisibles(
	text: string,
	isTrailingRegion: boolean,
): InvisibleSegment[] {
	if (!text) return [];

	const segments: InvisibleSegment[] = [];
	let current = "";
	let currentIsInvisible = false;

	for (const ch of text) {
		const invisible = ch === " " || ch === "\t";
		if (invisible !== currentIsInvisible && current) {
			segments.push({
				text: currentIsInvisible
					? current.replace(/ /g, "\u00B7").replace(/\t/g, "\u2192")
					: current,
				isInvisible: currentIsInvisible,
				isTrailing: currentIsInvisible && isTrailingRegion,
			});
			current = "";
		}
		current += ch;
		currentIsInvisible = invisible;
	}

	if (current) {
		segments.push({
			text: currentIsInvisible
				? current.replace(/ /g, "\u00B7").replace(/\t/g, "\u2192")
				: current,
			isInvisible: currentIsInvisible,
			isTrailing: currentIsInvisible && isTrailingRegion,
		});
	}

	return segments;
}
