import { contrast, verdict } from "./contrast.mjs";
const ERR = "oklch(0.72 0.16 25)";
const FG2 = "oklch(0.73 0.005 260)";
const mix = (c, p) => `color-mix(in oklch, ${c} ${p}%, transparent)`;
const r = (fg, base, layers = []) => contrast(fg, base, { layers });

console.log("## danger-bg tint sweep: err(.72) text on err-N% over surfaces");
for (const p of [15, 12, 10, 9, 8, 7, 6]) {
	const a = r(ERR, "var(--bg-1)", [mix(ERR, p)]);
	const b = r(ERR, "var(--bg-0)", [mix(ERR, p)]);
	const c = r(ERR, "var(--bg-selected)", [mix(ERR, p)]); // toast worst
	const d = r(ERR, "var(--bg-2)", [mix("var(--info)", 6), mix(ERR, p)]); // diff discard stacked
	console.log(`  ${p}%: bg1=${a.toFixed(2)} bg0=${b.toFixed(2)} bg-sel(toast)=${c.toFixed(2)} diff-discard=${d.toFixed(2)}`);
}

console.log("\n## FileRow badge tint sweep on bg-selected (worst row), err→.72");
const status = { A: "var(--ok)", M: "var(--warn)", D: ERR, R: "var(--info)", T: "var(--lane-3)", "?": FG2 };
for (const p of [10, 8, 6, 5, 4, 0]) {
	const out = Object.entries(status).map(([k, c]) => `${k}=${r(c, "var(--bg-selected)", p ? [mix(c, p)] : []).toFixed(2)}`).join(" ");
	console.log(`  ${p}%: ${out}`);
}

console.log("\n## fg-2 L sweep on the lightest transient highlights");
for (const L of [0.73, 0.75, 0.77, 0.78]) {
	const fg = `oklch(${L} 0.005 260)`;
	const sc = r(fg, "var(--bg-1)", ["var(--color-search-current)"]); // accent 20%
	const gAdd = r(fg, "var(--bg-1)", [mix("var(--ok)", 18)]);
	const gDel = r(fg, "var(--bg-1)", [mix(ERR, 18)]);
	console.log(`  fg-2 L=${L}: search-current=${sc.toFixed(2)} gutter-add-sel=${gAdd.toFixed(2)} gutter-del-sel=${gDel.toFixed(2)}`);
}
console.log("  -- alt: lower search-current tint (accent N%) with fg-2=.73 --");
for (const p of [20, 16, 14, 12]) {
	console.log(`  accent ${p}%: ${r(FG2, "var(--bg-1)", [mix("var(--accent)", p)]).toFixed(2)}`);
}

console.log("\n## .end-button.confirming → SOLID err with DARK text (accent-fg / bg-0)");
console.log(`  bg-0 on solid err(.72): ${r("var(--bg-0)", ERR).toFixed(2)}`);
console.log(`  accent-fg on solid err(.72): ${r("var(--accent-fg)", ERR).toFixed(2)}`);
console.log(`  bg-0 on err-hover darker oklch(0.66 0.16 25): ${r("var(--bg-0)", "oklch(0.66 0.16 25)").toFixed(2)}`);

console.log("\n## diff word-emphasis comment (stacked-emphasis, AA allowed) with err→.72 del side");
console.log(`  syn-comment on add word stack: ${r("#88b974", "var(--bg-0)", [mix("var(--ok)", 18), mix("var(--ok)", 22)]).toFixed(2)}`);
console.log(`  syn-comment on del word stack(err.72): ${r("#88b974", "var(--bg-0)", [mix(ERR, 18), mix(ERR, 22)]).toFixed(2)}`);
