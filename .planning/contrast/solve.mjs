// Find the minimum oklch L (chroma/hue fixed) for a fg to hit a target ratio on a bg.
import { contrast } from "./contrast.mjs";

function solveL(C, H, baseExpr, layers, target) {
	let lo = 0, hi = 1, best = 1;
	for (let i = 0; i < 60; i++) {
		const mid = (lo + hi) / 2;
		const r = contrast(`oklch(${mid} ${C} ${H})`, baseExpr, { layers });
		if (r >= target) {
			best = mid;
			hi = mid;
		} else lo = mid;
	}
	return best;
}

const cases = [
	["fg-3 captions C=.006 H=260", 0.006, 260],
	["fg-2 secondary C=.005 H=260", 0.005, 260],
	["err text C=.16 H=25", 0.16, 25],
];
const surfaces = ["--bg-0", "--bg-1", "--bg-2", "--bg-3", "--bg-hover", "--bg-selected"];

for (const [label, C, H] of cases) {
	console.log(`\n## ${label}`);
	for (const s of surfaces) {
		const L = solveL(C, H, `var(${s})`, [], 7);
		console.log(`  7:1 on ${s}: L=${L.toFixed(3)}`);
	}
}

// err is also a fill; check dark-text-on-err contrast if we raise err's L
console.log("\n## --err raised: does --bg-0 dark text still pass on the lighter fill?");
for (const L of [0.68, 0.70, 0.72, 0.74]) {
	const onText = contrast(`oklch(${L} 0.16 25)`, "var(--bg-0)");
	const darkOn = contrast("var(--bg-0)", `oklch(${L} 0.16 25)`);
	console.log(`  err L=${L}: as-text-on-bg0=${onText.toFixed(2)}  bg0-dark-on-fill=${darkOn.toFixed(2)}`);
}
