// Verify proposed token/component fixes against EVERY confirmed problem case.
import { contrast, verdict } from "./contrast.mjs";

// ── proposed new values ──
const FG2 = "oklch(0.73 0.005 260)"; // was 0.68
const FG3 = "oklch(0.70 0.006 260)"; // was 0.54
const ERR = "oklch(0.72 0.16 25)"; // was 0.68
const L5 = "oklch(0.76 0.12 200)"; // was 0.70
const L7 = "oklch(0.76 0.02 75)"; // was 0.70
const DANGER_BG = (over) => contrast.bind(null); // placeholder
const PILL_TINT = 14; // was 22
const BADGE_TINT = 10; // was 18

const mix = (c, p) => `color-mix(in oklch, ${c} ${p}%, transparent)`;

let pass = 0, fail = 0;
function check(label, fg, base, opts = {}) {
	const r = contrast(fg, base, opts);
	const large = opts.large || false;
	const v = verdict(r, { large });
	const target = opts.target || (large ? "AA" : "AAA");
	const ok = target === "AA" ? r >= 4.5 : r >= 7;
	const mark = ok ? "✅" : "❌";
	if (ok) pass++; else fail++;
	console.log(`${mark} ${r.toFixed(2)} (${v}) [need ${target}] ${label}`);
}

console.log("## Text-ramp tokens raised (fg-2→.73, fg-3→.70)");
for (const s of ["--bg-0", "--bg-1", "--bg-2", "--bg-3", "--bg-hover", "--bg-selected"]) {
	check(`fg-2 on ${s}`, FG2, `var(${s})`);
}
for (const s of ["--bg-0", "--bg-1", "--bg-2"]) check(`fg-3 on ${s}`, FG3, `var(${s})`);
// fg-3 worst real surfaces are bg-1/bg-2 (headers/detail/mode-tabs); not on selected rows
console.log("  (fg-3 on hover/selected — fg-3 is not used on row-hover surfaces per audit; bg-2 is its worst)");

console.log("\n## fg-2 on the stubborn translucent tints it actually sits on");
check("fg-2 search counter on bg-2", FG2, "var(--bg-2)");
check("fg-2 muted-bg over bg-1", FG2, "var(--bg-1)", { layers: ["var(--color-muted-bg)"] });
check("fg-2 muted-bg over bg-0 (merge editor)", FG2, "var(--bg-0)", { layers: ["var(--color-muted-bg)"] });
check("fg-2 CommitRow author on search-current/bg-1", FG2, "var(--bg-1)", { layers: ["var(--color-search-current)"] });
check("fg-2 diff gutter on add-hi selected (over bg-1)", FG2, "var(--bg-1)", { layers: [mix("var(--ok)", 18)] });
check("fg-2 diff gutter on del-hi selected (over bg-1)", FG2, "var(--bg-1)", { layers: [mix(ERR, 18)] });
check("fg-2 OperationBanner counter (info 8% over bg-1)", FG2, "var(--bg-1)", { layers: [mix("var(--info)", 8)] });
check("fg-2 WelcomeScreen path on white/5 over bg-0", FG2, "var(--bg-0)", { layers: [mix("oklch(1 0 0)", 5)] });

console.log("\n## --err raised to 0.72: plain text + danger tints");
check("err text on bg-1", ERR, "var(--bg-1)");
check("err text on bg-2", ERR, "var(--bg-2)");
check("err on danger-bg (err 15%) over bg-1 [Discard/Abort/Cancel/Delete btns]", ERR, "var(--bg-1)", { layers: [mix(ERR, 15)] });
check("err on danger-bg over bg-0 (WelcomeScreen banner)", ERR, "var(--bg-0)", { layers: [mix(ERR, 15)] });
check("Toast err on danger-bg over bg-selected (worst)", ERR, "var(--bg-selected)", { layers: [mix(ERR, 15)] });
check("err on danger-bg-subtle (err 8%) over bg-0 (rebase validation)", ERR, "var(--bg-0)", { layers: [mix(ERR, 8)] });
check("diff Discard: err on danger-bg over hunk-header tint(info 6%/bg-2)", ERR, "var(--bg-2)", { layers: [mix("var(--info)", 6), mix(ERR, 15)] });
check("OperationBanner Abort: err on err15% over info8% over bg-1", ERR, "var(--bg-1)", { layers: [mix("var(--info)", 8), mix(ERR, 15)] });

console.log("\n## FileRow status badge: tint 18→10%, err→.72, on focused row (bg-selected)");
check("A (ok) badge", "var(--ok)", "var(--bg-selected)", { layers: [mix("var(--ok)", BADGE_TINT)] });
check("M (warn) badge", "var(--warn)", "var(--bg-selected)", { layers: [mix("var(--warn)", BADGE_TINT)] });
check("D (err) badge", ERR, "var(--bg-selected)", { layers: [mix(ERR, BADGE_TINT)] });
check("R (info) badge", "var(--info)", "var(--bg-selected)", { layers: [mix("var(--info)", BADGE_TINT)] });
check("T (lane-3) badge", "var(--lane-3)", "var(--bg-selected)", { layers: [mix("var(--lane-3)", BADGE_TINT)] });
check("? (fg-2) badge", FG2, "var(--bg-selected)", { layers: [mix(FG2, BADGE_TINT)] });

console.log("\n## RefPill / graph pills: tint 22→14%, lanes 5,7 → .76, opacity:0.6 removed");
const lanes = { "lane-0": "var(--lane-0)", "lane-1": "var(--lane-1)", "lane-2": "var(--lane-2)", "lane-3": "var(--lane-3)", "lane-4": "var(--lane-4)", "lane-5": L5, "lane-6": "var(--lane-6)", "lane-7": L7 };
for (const [name, c] of Object.entries(lanes)) {
	check(`${name} pill on bg-1`, c, "var(--bg-1)", { layers: [mix(c, PILL_TINT)] });
}
console.log("  -- pills on SELECTED commit row (documented AA exception) --");
for (const name of ["lane-5", "lane-7"]) {
	check(`${name} pill on bg-selected`, lanes[name], "var(--bg-selected)", { layers: [mix(lanes[name], PILL_TINT)], target: "AA" });
}
// SVG graph pills use 20% capsule fill; bump to 14% too. worst lane-7 on bg-selected
check("SVG pill lane-7 capsule 14% on bg-selected", L7, "var(--bg-selected)", { layers: [mix(L7, PILL_TINT)], target: "AA" });

console.log("\n## ReviewPanel .end-button.confirming: switch dark on-accent → light fg-1");
check("confirming: fg-1 on danger-bg over bg-1", "var(--fg-1)", "var(--bg-1)", { layers: [mix(ERR, 15)] });
check("confirming:hover fg-1 on solid err", "var(--fg-1)", ERR, { target: "AA" }); // light on solid red — large/transient

console.log("\n## Placeholder: explicit token fg-2 @ opacity 1 over input wells");
check("placeholder fg-2 on bg-0 well", FG2, "var(--bg-0)");
check("placeholder fg-2 on bg-2 (InputDialog)", FG2, "var(--bg-2)");

console.log(`\n=== ${pass} pass / ${fail} fail (against per-case target) ===`);

// ── solve opacity-dim states for the opacity that yields AA(4.5) and AAA(7) on worst backdrop ──
console.log("\n## Opacity-dim states — minimum opacity for AA / AAA on worst backdrop");
function solveOpacity(fg, base, layers, target) {
	let lo = 0, hi = 1, best = 1;
	for (let i = 0; i < 50; i++) {
		const m = (lo + hi) / 2;
		const r = contrast(fg, base, { layers, opacity: m });
		if (r >= target) { best = m; hi = m; } else lo = m;
	}
	return best;
}
const dimCases = [
	["search-dim msg (fg-1 over bg-selected)", "var(--fg-1)", "var(--bg-selected)", []],
	["search-dim author/sha (fg-2 over bg-1)", FG2, "var(--bg-1)", []],
	["search-dim author on selected (fg-2 over bg-selected)", FG2, "var(--bg-selected)", []],
	["SVG search-dim pill (lane-7 over search-current/bg-1)", L7, "var(--bg-1)", ["var(--color-search-current)"]],
	["drop-row msg (fg-1 over bg-selected)", "var(--fg-1)", "var(--bg-selected)", []],
	["drop-row date (fg-2 over bg-selected)", FG2, "var(--bg-selected)", []],
	["orphaned fileref (fg-2 over bg-1)", FG2, "var(--bg-1)", []],
	["orphaned diff excerpt (fg-1 over del-tint/bg-0)", "var(--fg-1)", "var(--bg-0)", [mix(ERR, 11)]],
];
for (const [label, fg, base, layers] of dimCases) {
	const aa = solveOpacity(fg, base, layers, 4.5);
	const aaa = solveOpacity(fg, base, layers, 7);
	console.log(`  ${label}: AA@opacity≥${aa.toFixed(2)}  AAA@opacity≥${aaa.toFixed(2)}`);
}
