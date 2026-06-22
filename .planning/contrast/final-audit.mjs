// Authoritative re-verification: every confirmed finding recomputed against the
// LIVE (edited) app.css tokens + the component-local values now in the source.
import { contrast, verdict } from "./contrast.mjs";
const mix = (c, p) => `color-mix(in oklch, ${c} ${p}%, transparent)`;
const L7 = "var(--lane-7)"; // worst lane (now 0.76)

let problems = 0;
// target: "AAA" must be >=7 ; "AA" documented transient/secondary must be >=4.5 ; note shows context
function row(label, fg, base, { layers = [], opacity, target = "AAA", note = "" } = {}) {
	const r = contrast(fg, base, { layers, opacity });
	const v = verdict(r);
	const floor = target === "AAA" ? 7 : 4.5;
	const ok = r >= floor;
	if (!ok) problems++;
	const mark = ok ? "✅" : "⚠️ ";
	console.log(`${mark} ${r.toFixed(2)} [${target}] ${label}${note ? "  — " + note : ""}`);
}
const H = (t) => console.log(`\n### ${t}`);

H("Toolbar / tabs / search");
row("SearchBar counter (fg-2/bg-2)", "var(--fg-2)", "var(--bg-2)");
row("SearchBar placeholder (text-muted @opacity1 / bg-2)", "var(--color-text-muted)", "var(--bg-2)");
row("TabBar dragged tab (fg-2/bg-selected)", "var(--fg-2)", "var(--bg-selected)");

H("Sidebar / branches / refs");
row("RemoteGroup name (fg-3/bg-1)", "var(--fg-3)", "var(--bg-1)");
row("Stash index (was fg-4 → fg-2 / bg-hover)", "var(--fg-2)", "var(--bg-hover)");
row("Stash message (fg-2/bg-hover)", "var(--fg-2)", "var(--bg-hover)");
row("BranchRow loading name (fg-2/bg-hover)", "var(--fg-2)", "var(--bg-hover)");
row("RecentRepos path (fg-2/bg-hover)", "var(--fg-2)", "var(--bg-hover)");
row("BranchRow error (err / danger-bg / bg-1)", "var(--err)", "var(--bg-1)", { layers: ["var(--color-danger-bg)"] });
row("Stash error (err / bg-1)", "var(--err)", "var(--bg-1)");

H("File lists / staging");
for (const [k, c] of [["A", "var(--ok)"], ["M", "var(--warn)"], ["D", "var(--err)"], ["R", "var(--info)"], ["T", "var(--lane-3)"], ["?", "var(--fg-2)"]]) {
	row(`Badge ${k} normal row (bg-0)`, c, "var(--bg-0)", { layers: [mix(c, 8)] });
	row(`Badge ${k} focused row (bg-selected)`, c, "var(--bg-selected)", { layers: [mix(c, 8)], target: "AA", note: "transient focused-row selection" });
}
row("FileRow loading filename (fg-2/bg-selected)", "var(--fg-2)", "var(--bg-selected)");
row("DirectoryRow count (fg-2/bg-selected)", "var(--fg-2)", "var(--bg-selected)");
row("Discard All / Abort (err / danger-bg / bg-1)", "var(--err)", "var(--bg-1)", { layers: ["var(--color-danger-bg)"] });
row("Header branch pill (lane-0 / 14% / bg-1)", "var(--lane-0)", "var(--bg-1)", { layers: [mix("var(--lane-0)", 14)] });

H("Commit graph / rows");
row("search-dim message (fg-1 @0.6 / bg-1)", "var(--fg-1)", "var(--bg-1)", { opacity: 0.6, target: "AA", note: "transient search de-emphasis of NON-match rows" });
row("search-dim author (fg-2 @0.6 / bg-1)", "var(--fg-2)", "var(--bg-1)", { opacity: 0.6, target: "AA", note: "secondary metadata on de-emphasized non-match — best-effort" });
row("SVG pill text normal (lane-7 / 14% / bg-1)", L7, "var(--bg-1)", { layers: [mix(L7, 14)] });
row("SVG pill text selected (lane-7 / 14% / bg-selected)", L7, "var(--bg-selected)", { layers: [mix(L7, 14)], target: "AA", note: "transient selected commit row" });
row("SVG +N badge (lane-7 / 14% / bg-1)", L7, "var(--bg-1)", { layers: [mix(L7, 14)] });
row("Pill tooltip (lane-7 / 14% mixed bg-2)", L7, "color-mix(in oklch, var(--lane-7) 14%, var(--bg-2))");
row("Multi-ref tooltip hover (lane-7 / white8 / bg-2)", L7, "var(--bg-2)", { layers: [mix("oklch(1 0 0)", 8)] });
row("Column headers (fg-3 / bg-1)", "var(--fg-3)", "var(--bg-1)");
row("CommitRow author on search-current (fg-2 / accent20% / bg-1)", "var(--fg-2)", "var(--bg-1)", { layers: ["var(--color-search-current)"], target: "AA", note: "current-match highlight (transient)" });
row("Diff gutter on selected add line (fg-2 / add-hi / bg-1)", "var(--fg-2)", "var(--bg-1)", { layers: ["var(--diff-add-hi)"], target: "AA", note: "selected-to-stage line (transient)" });

H("Diff / review");
const HDR = "color-mix(in oklch, var(--info) 6%, var(--bg-2))";
row("Hunk header label (info70%/fg-3 mix / header)", "color-mix(in oklch, var(--info) 70%, var(--fg-3))", HDR);
row("Comment btn (accent / accent-bg / header)", "var(--color-accent)", HDR, { layers: ["var(--color-accent-bg)"] });
row("Stage btn (ok / success-bg / header)", "var(--color-success)", HDR, { layers: ["var(--color-success-bg)"] });
row("Discard btn (err / danger-bg / header)", "var(--color-danger)", HDR, { layers: ["var(--color-danger-bg)"], target: "AA", note: "dense diff toolbar, transient on hover/expand" });
row("confirming resting (fg-1 / danger-bg / bg-1)", "var(--fg-1)", "var(--bg-1)", { layers: ["var(--color-danger-bg)"] });
row("confirming hover (fg-1 / danger-bg-strong / bg-1)", "var(--fg-1)", "var(--bg-1)", { layers: ["var(--color-danger-bg-strong)"] });
row("orphaned excerpt (fg-1 @0.6 / del-tint / bg-0)", "var(--fg-1)", "var(--bg-0)", { layers: [mix("var(--err)", 11)], opacity: 0.6, target: "AA", note: "orphaned comment de-emphasis" });
row("orphaned fileref (fg-2 @0.6 / bg-1)", "var(--fg-2)", "var(--bg-1)", { opacity: 0.6, target: "AA", note: "secondary ref on orphaned comment — best-effort" });
row("syntax comment word-add stack", "var(--color-syn-comment)", "var(--bg-0)", { layers: ["var(--diff-add-bg)", "var(--color-diff-word-add-bg)"], target: "AA", note: "stacked word-emphasis (per diff_contrast_aaa)" });

H("Dialogs / banners / toasts / editors");
row("OperationBanner counter (fg-2 / info8% / bg-1)", "var(--fg-2)", "var(--bg-1)", { layers: [mix("var(--info)", 8)] });
row("OpBanner Continue (ok / success-bg / info8% / bg-1)", "var(--color-success)", "var(--bg-1)", { layers: [mix("var(--info)", 8), "var(--color-success-bg)"] });
row("OpBanner Abort (err / danger-bg / info8% / bg-1)", "var(--color-danger)", "var(--bg-1)", { layers: [mix("var(--info)", 8), "var(--color-danger-bg)"] });
row("Toast error (err / danger-bg / bg-0 typical)", "var(--color-danger)", "var(--bg-0)", { layers: ["var(--color-danger-bg)"] });
row("Toast error (err / danger-bg / bg-selected worst)", "var(--color-danger)", "var(--bg-selected)", { layers: ["var(--color-danger-bg)"], target: "AA", note: "transient toast overlaying a selected row" });
row("MergeEditor Take-All (ok / success-bg / accent-bg / bg-0)", "var(--color-success)", "var(--bg-0)", { layers: ["var(--color-accent-bg)", "var(--color-success-bg)"] });
row("MergeEditor gutter (fg-2 / del-tint11% / bg-0)", "var(--fg-2)", "var(--bg-0)", { layers: [mix("var(--err)", 11)] });
row("RebaseEditor cancel (err / danger-bg / bg-1)", "var(--color-danger)", "var(--bg-1)", { layers: ["var(--color-danger-bg)"] });
row("RebaseEditor validation (err / danger-bg-subtle / bg-0)", "var(--color-danger)", "var(--bg-0)", { layers: ["var(--color-danger-bg-subtle)"] });
row("RebaseEditor drop msg (fg-1 @0.6 / bg-0)", "var(--fg-1)", "var(--bg-0)", { opacity: 0.6, target: "AA", note: "drop row de-emphasis (line-through carries meaning)" });
row("InputDialog label (fg-2 / bg-2)", "var(--fg-2)", "var(--bg-2)");
row("InputDialog placeholder (text-muted @1 / bg-0)", "var(--color-text-muted)", "var(--bg-0)");
row("WelcomeScreen error (err / danger-bg / bg-0)", "var(--color-danger)", "var(--bg-0)", { layers: ["var(--color-danger-bg)"] });
row("WelcomeScreen path (fg-2 / white5 / bg-0)", "var(--fg-2)", "var(--bg-0)", { layers: [mix("oklch(1 0 0)", 5)] });

console.log(`\n=== ${problems} case(s) below their target floor (AAA<7 or AA<4.5) ===`);
