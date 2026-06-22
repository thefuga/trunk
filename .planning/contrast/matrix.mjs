// Deterministic backbone: every app.css-resolvable fg/bg pair, with real WCAG ratios.
import { contrast, verdict } from "./contrast.mjs";

function row(label, fg, base, layers = [], large = false) {
	let r;
	try {
		r = contrast(fg, base, { layers });
	} catch (e) {
		return `| ${label} | ERR | ${e.message} |`;
	}
	const v = verdict(r, { large });
	const mark = v === "AAA" ? "✅" : v === "AA" ? "🟡" : "❌";
	const stack = layers.length ? ` + [${layers.join(", ")}]` : "";
	return `| ${label} | \`${fg}\` | \`${base}\`${stack} | ${r.toFixed(2)} | ${mark} ${v} |`;
}

function section(title, rows) {
	console.log(`\n### ${title}\n`);
	console.log("| element | fg | bg | ratio | verdict |");
	console.log("|---|---|---|---|---|");
	for (const r of rows) console.log(r);
}

const SURFACES = ["--bg-0", "--bg-1", "--bg-2", "--bg-3", "--bg-hover", "--bg-selected"];

// 1. Text ramp over every opaque surface
section(
	"Text ramp × surfaces (fg-4 disabled = exempt)",
	["--fg-0", "--fg-1", "--fg-2", "--fg-3", "--fg-4"].flatMap((fg) =>
		SURFACES.map((bg) => row(`${fg} on ${bg}`, `var(${fg})`, `var(${bg})`)),
	),
);

// 2. Semantic / accent colors used AS TEXT over surfaces
section(
	"Semantic & accent as text × surfaces",
	["--ok", "--warn", "--err", "--info", "--accent", "--accent-hi", "--accent-lo"].flatMap(
		(fg) => ["--bg-0", "--bg-1", "--bg-2", "--bg-3"].map((bg) =>
			row(`${fg} on ${bg}`, `var(${fg})`, `var(${bg})`),
		),
	),
);

// 3. Button text on accent/semantic fills
section("Button/fill text", [
	row("accent-fg on accent", "var(--accent-fg)", "var(--accent)"),
	row("on-accent on accent", "var(--color-on-accent)", "var(--accent)"),
	row("fg-0 on accent", "var(--fg-0)", "var(--accent)"),
	row("fg-0 on err (danger btn)", "var(--fg-0)", "var(--err)"),
	row("bg-0 on err (danger btn dark text)", "var(--bg-0)", "var(--err)"),
	row("fg-0 on ok (success btn)", "var(--fg-0)", "var(--ok)"),
	row("bg-0 on ok", "var(--bg-0)", "var(--ok)"),
	row("fg-0 on warn", "var(--fg-0)", "var(--warn)"),
	row("bg-0 on warn", "var(--bg-0)", "var(--warn)"),
	row("accent-fg on accent-hi", "var(--accent-fg)", "var(--accent-hi)"),
]);

// 4. Status letters (file rows render on bg-1 sidebar / bg-0 list)
section(
	"File-status letters (A/M/D…) over list surfaces",
	[
		["new", "--color-status-new"],
		["modified", "--color-status-modified"],
		["deleted", "--color-status-deleted"],
		["renamed", "--color-status-renamed"],
		["typechange", "--color-status-typechange"],
		["conflicted", "--color-status-conflicted"],
	].flatMap(([name, tok]) =>
		["--bg-0", "--bg-1", "--bg-hover", "--bg-selected"].map((bg) =>
			row(`${name} on ${bg}`, `var(${tok})`, `var(${bg})`),
		),
	),
);

// 5. Graph lane colors over graph bg (bg-1) and selected row
section(
	"Graph lane colors × graph surfaces",
	["--lane-0", "--lane-1", "--lane-2", "--lane-3", "--lane-4", "--lane-5", "--lane-6", "--lane-7"].flatMap(
		(l) => ["--bg-1", "--bg-selected"].map((bg) => row(`${l} on ${bg}`, `var(${l})`, `var(${bg})`)),
	),
);

// 6. Syntax palette over diff backgrounds (worst-case stacks). Base = bg-0 (editor well).
const SYN = [
	"keyword", "string", "comment", "number", "type", "function", "variable",
	"constant", "operator", "punctuation", "attribute", "tag", "property", "regex", "escape",
];
section(
	"Syntax palette × diff line tints (context / add / del / selected)",
	SYN.flatMap((s) => [
		row(`${s} context`, `var(--color-syn-${s})`, "var(--bg-0)"),
		row(`${s} add`, `var(--color-syn-${s})`, "var(--bg-0)", ["var(--diff-add-bg)"]),
		row(`${s} del`, `var(--color-syn-${s})`, "var(--bg-0)", ["var(--diff-del-bg)"]),
		row(`${s} add-selected`, `var(--color-syn-${s})`, "var(--bg-0)", ["var(--diff-add-hi)"]),
		row(`${s} del-selected`, `var(--color-syn-${s})`, "var(--bg-0)", ["var(--diff-del-hi)"]),
	]),
);

// 6b. Syntax palette over word-emphasis stacks (the allowed AA "stacked-emphasis" case)
section(
	"Syntax palette × word-emphasis stack (stacked-emphasis — AA allowed)",
	SYN.flatMap((s) => [
		row(`${s} word-add`, `var(--color-syn-${s})`, "var(--bg-0)", ["var(--diff-add-bg)", "var(--color-diff-word-add-bg)"]),
		row(`${s} word-del`, `var(--color-syn-${s})`, "var(--bg-0)", ["var(--diff-del-bg)", "var(--color-diff-word-delete-bg)"]),
	]),
);

// 7. Diff plain text (unhighlighted) over tints
section("Diff plain text (--color-diff-text) over tints", [
	row("diff-text context", "var(--color-diff-text)", "var(--bg-0)"),
	row("diff-text add", "var(--color-diff-text)", "var(--bg-0)", ["var(--diff-add-bg)"]),
	row("diff-text del", "var(--color-diff-text)", "var(--bg-0)", ["var(--diff-del-bg)"]),
	row("diff-text add-sel", "var(--color-diff-text)", "var(--bg-0)", ["var(--diff-add-hi)"]),
	row("diff-text del-sel", "var(--color-diff-text)", "var(--bg-0)", ["var(--diff-del-hi)"]),
	row("diff-add marker on add", "var(--color-diff-add)", "var(--bg-0)", ["var(--diff-add-bg)"]),
	row("diff-del marker on del", "var(--color-diff-delete)", "var(--bg-0)", ["var(--diff-del-bg)"]),
]);

// 8. Tinted-background text: muted-bg chips, banners, badges, search highlights
section("Text over tinted translucent backgrounds", [
	row("fg-2 on muted-bg over bg-1", "var(--fg-2)", "var(--bg-1)", ["var(--color-muted-bg)"]),
	row("fg-1 on muted-bg over bg-1", "var(--fg-1)", "var(--bg-1)", ["var(--color-muted-bg)"]),
	row("warn on warning-bg over bg-1", "var(--warn)", "var(--bg-1)", ["var(--color-warning-bg)"]),
	row("fg-1 on warning-bg over bg-1", "var(--fg-1)", "var(--bg-1)", ["var(--color-warning-bg)"]),
	row("warn on banner-warn-bg over bg-0", "var(--warn)", "var(--bg-0)", ["var(--color-banner-warning-bg)"]),
	row("fg-1 on banner-warn-bg over bg-0", "var(--fg-1)", "var(--bg-0)", ["var(--color-banner-warning-bg)"]),
	row("info on banner-info-bg over bg-0", "var(--info)", "var(--bg-0)", ["var(--color-banner-info-bg)"]),
	row("fg-1 on banner-info-bg over bg-0", "var(--fg-1)", "var(--bg-0)", ["var(--color-banner-info-bg)"]),
	row("err on danger-bg over bg-1", "var(--err)", "var(--bg-1)", ["var(--color-danger-bg)"]),
	row("err on danger-bg-subtle over bg-1", "var(--err)", "var(--bg-1)", ["var(--color-danger-bg-subtle)"]),
	row("ok on success-bg over bg-1", "var(--ok)", "var(--bg-1)", ["var(--color-success-bg)"]),
	row("fg-1 on accent-bg over bg-1", "var(--fg-1)", "var(--bg-1)", ["var(--color-accent-bg)"]),
	row("accent on accent-bg over bg-1", "var(--accent)", "var(--bg-1)", ["var(--color-accent-bg)"]),
	row("fg-1 on search-current over bg-0", "var(--fg-1)", "var(--bg-0)", ["var(--color-search-current)"]),
	row("fg-1 on search-match over bg-0", "var(--fg-1)", "var(--bg-0)", ["var(--color-search-match)"]),
	row("badge-warning on badge-warning-bg over bg-2", "var(--color-badge-warning)", "var(--bg-2)", ["var(--color-badge-warning-bg)"]),
]);

console.log("\n\n(legend: ✅ AAA ≥7  🟡 AA ≥4.5  ❌ FAIL <4.5 — large-text threshold not applied here)\n");
