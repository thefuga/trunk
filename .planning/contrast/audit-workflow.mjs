export const meta = {
	name: 'contrast-audit-app-wide',
	description: 'Audit every text surface in Trunk for WCAG AAA contrast; map fg/bg pairs and adversarially verify each failing finding with the deterministic helper',
	phases: [
		{ title: 'Audit', detail: 'one agent per UI surface group maps text→worst-case-bg and runs the contrast helper' },
		{ title: 'Verify', detail: 'independently re-derive the composited background for each AA/FAIL real-text finding' },
	],
}

const RAMP = `
THEME (src/app.css :root) — cool oklch ramp, hue ~260, electric-blue accent.
Surfaces (opaque, darkest→lightest): --bg-0 oklch(.08) body/editor/input wells; --bg-1 oklch(.10) panels (sidebar,graph,toolbar,tabbar); --bg-2 oklch(.13) chips,dialogs; --bg-3 oklch(.16) raised badges/avatars; --bg-hover oklch(.18); --bg-selected oklch(.22). Borders --line oklch(.22), --line-strong oklch(.30).
Text ramp: --fg-0 oklch(.98) primary; --fg-1 oklch(.86) body (=--color-text); --fg-2 oklch(.68) secondary (=--color-text-muted); --fg-3 oklch(.54) tertiary/captions; --fg-4 oklch(.38) disabled.
Accent: --accent oklch(.76 .15 245); --accent-hi; --accent-lo oklch(.5) dim; --accent-fg oklch(.14) dark-on-accent text; --accent-soft.
Semantic: --ok oklch(.74 .12 150); --warn oklch(.82 .14 90); --err oklch(.68 .16 25); --info oklch(.72 .1 235). Lanes --lane-0..7. Syntax --color-syn-*. Many legacy --color-* alias onto these.

KNOWN RESULTS (already computed centrally, do not recompute these — focus on MAPPING them to real elements + finding NEW component-local colors/opacity/stacks):
- --fg-1, --fg-0 over any surface: AAA. --fg-2 over bg-0/bg-1: AAA (7.1); over bg-2/bg-3/hover/selected: AA only (5.97–6.96).
- --fg-3 over EVERY surface: FAIL (3.4–4.1). --fg-4: FAIL everywhere (disabled, exempt only if truly disabled).
- --ok/--warn/--info/--accent/--accent-hi as text over surfaces: AAA. --err as text: AA (6.2–6.7). --accent-lo as text: FAIL (3.3–3.5).
- Dark text on bright fill: --accent-fg/--bg-0 on --accent/--ok/--warn: AAA. But --fg-0 (white) on --accent/--ok/--warn/--err fills: FAIL (1.6–2.9) — flag any button using light text on a colored fill.
- Status letters: new/modified/renamed/typechange/conflicted AAA; deleted (=--err) AA. Lanes mostly AAA; lane-5/lane-7 on bg-selected AA.
- Diff syntax palette over line tints: AAA; over word-emphasis stack: AA (allowed stacked-emphasis exemption).
`

const HELPER = `
CONTRAST HELPER (deterministic, authoritative — never eyeball a ratio, always run this):
  cd /Users/joaofnds/code/trunk/.planning/contrast
  node contrast.mjs '<fg-expr>' '<base-expr>' ['<layer-expr>'...]
It resolves var(--x), oklch(...), color-mix(in oklch, C p%, transparent), rgba(), #hex; composites translucent <layer> exprs over the opaque <base> in sRGB space (layers listed bottom→top); prints "<ratio>:1  <AAA|AA|FAIL>".
Standard: normal text AAA=7:1; large text (≥24px, or ≥18.66px bold) AAA=4.5:1.
Worst-case rule: the background behind a glyph is the DARKEST-contrast real surface it can sit on — include hover, selected, tinted, and STACKED translucent layers, and any \`opacity:<1>\` on the text (opacity composites the glyph toward its backdrop, lowering contrast). To model opacity, compute the composited bg first, then treat the text as color-mix(in oklch, <fg> <opacity*100>%, transparent) over that bg — or just report the opacity value and the bg, and note it.
`

const SCHEMA = {
	type: 'object',
	required: ['surface', 'findings'],
	properties: {
		surface: { type: 'string' },
		findings: {
			type: 'array',
			items: {
				type: 'object',
				required: ['element', 'file', 'fg', 'bg', 'ratio', 'verdict', 'category'],
				properties: {
					element: { type: 'string', description: 'what text this is (e.g. "branch name", "commit date", "input placeholder")' },
					file: { type: 'string', description: 'path:line' },
					fg: { type: 'string', description: 'foreground token/expr as written' },
					bg: { type: 'string', description: 'worst-case background: base expr + any stacked layers, described' },
					opacity: { type: ['number', 'null'], description: 'opacity on the text if <1, else null' },
					ratio: { type: 'number', description: 'WCAG ratio from the helper for the worst case' },
					verdict: { type: 'string', enum: ['AAA', 'AA', 'FAIL'] },
					category: { type: 'string', enum: ['real-text', 'placeholder', 'disabled', 'decorative', 'large-text'], description: 'real-text/placeholder must meet AAA; disabled & purely-decorative are exempt; large-text uses 4.5 threshold' },
					notes: { type: 'string', description: 'worst-case reasoning, the exact helper command, and a suggested fix if failing' },
				},
			},
		},
	},
}

const VERDICT_SCHEMA = {
	type: 'object',
	required: ['element', 'confirmed', 'correctedRatio', 'correctedVerdict', 'reasoning'],
	properties: {
		element: { type: 'string' },
		confirmed: { type: 'boolean', description: 'true if the original finding (its fg, worst-case bg, and verdict) is correct' },
		correctedFg: { type: 'string' },
		correctedBg: { type: 'string' },
		correctedRatio: { type: 'number' },
		correctedVerdict: { type: 'string', enum: ['AAA', 'AA', 'FAIL'] },
		exempt: { type: 'boolean', description: 'true if this is genuinely disabled or purely decorative text (WCAG 1.4.6 exempt)' },
		reasoning: { type: 'string', description: 'how you independently re-derived the worst-case background from the component source, and the helper command you ran' },
	},
}

const GROUPS = [
	{ surface: 'Toolbar / top bar / tab bar / search', files: 'src/components/Toolbar.svelte, TabBar.svelte, PullDropdown.svelte, SearchBar.svelte, diff/DiffToolbar.svelte' },
	{ surface: 'Sidebar / branches / refs / remotes', files: 'src/components/BranchSidebar.svelte, BranchSection.svelte, BranchRow.svelte, RemoteGroup.svelte, RefPill.svelte, RecentReposPicker.svelte' },
	{ surface: 'File lists / staging / trees', files: 'src/components/StagingPanel.svelte, FileRow.svelte, DirectoryRow.svelte, TreeFileList.svelte, VirtualList.svelte' },
	{ surface: 'Commit graph / commit rows / avatars', files: 'src/components/CommitGraph.svelte, CommitRow.svelte, Avatar.svelte' },
	{ surface: 'Commit detail / commit form / message editor', files: 'src/components/CommitDetail.svelte, CommitForm.svelte, MessageEditor.svelte' },
	{ surface: 'Diff viewer + code review', files: 'src/components/DiffViewer.svelte, DiffPanel.svelte, diff/HunkView.svelte, diff/FullFileView.svelte, diff/SplitView.svelte, diff/CommentComposer.svelte, ReviewPanel.svelte' },
	{ surface: 'Dialogs / banners / toasts / editors / welcome', files: 'src/components/InputDialog.svelte, OperationBanner.svelte, Toast.svelte, MergeEditor.svelte, RebaseEditor.svelte, WelcomeScreen.svelte, RepoView.svelte, src/App.svelte' },
]

const auditPrompt = (g) => `You are auditing the text contrast of one UI surface of Trunk (a Tauri+Svelte desktop Git GUI) against WCAG AAA. Work from the repo root /Users/joaofnds/code/trunk.

SURFACE: ${g.surface}
COMPONENTS: ${g.files}
${RAMP}
${HELPER}

TASK:
1. Read each component file. Find EVERY rendered text element (labels, names, hashes, dates, counts, messages, badges, pills, button text, menu items, placeholders, empty-state text, tooltips, headers, hints). Include text set via inline style="color:..." and via scoped CSS classes.
2. For each text element, determine its foreground color (resolve the token/expr) AND the real WORST-CASE background it can render on. The worst case includes: the panel/surface it sits in (bg-0..3), plus hover/selected row states, plus any translucent tint layers stacked behind it (chips, banners, badges, pills, search highlight), plus any \`opacity:<1>\` applied to the text itself or an ancestor. Resolve color-mix and stacked layers to their true composited background.
3. Run the helper CLI for every element whose contrast isn't obviously AAA from the KNOWN RESULTS. Record the exact command in notes.
4. Classify each: real-text (must be AAA 7:1), placeholder (must be AAA — NOT exempt), large-text (≥24px or ≥18.66px bold → 4.5:1), disabled (exempt if genuinely a disabled control), decorative (exempt if purely decorative, e.g. an icon paired with a text label, or a separator glyph).
5. Pay special attention to: --fg-3 and --fg-2 used for real informational text; --color-text-muted; any \`opacity:\` on real text (e.g. dimmed remote pills, ⌘↵ hints, search-dimmed rows); inline \`color:\` values; light text on colored fills (danger/success/warning buttons); placeholder colors; tinted/stacked backgrounds.

Report ONLY elements that are real-text or placeholder or large-text AND land at AA or FAIL, PLUS any element using opacity<1 on real text, PLUS any inline/non-token color, PLUS a few representative AAA passes to show coverage. Do not pad with obvious AAA passes. Be exhaustive about failures. Every ratio MUST come from the helper. Return the structured object.`

phase('Audit')
const audited = await pipeline(
	GROUPS,
	(g) => agent(auditPrompt(g), { label: `audit:${g.surface.split(' ')[0]}`, phase: 'Audit', schema: SCHEMA }),
	(result, g) => {
		if (!result || !result.findings) return { surface: g.surface, findings: [], verified: [] }
		const needVerify = result.findings.filter(
			(f) => (f.category === 'real-text' || f.category === 'placeholder' || f.category === 'large-text') && (f.verdict === 'AA' || f.verdict === 'FAIL'),
		)
		return parallel(
			needVerify.map((f) => () =>
				agent(
					`Adversarially verify ONE contrast finding for Trunk's ${g.surface}. Work from /Users/joaofnds/code/trunk. Default to skepticism: re-derive everything yourself from the component source; do not trust the numbers handed to you.

FINDING:
  element: ${f.element}
  location: ${f.file}
  claimed fg: ${f.fg}
  claimed worst-case bg: ${f.bg}
  claimed opacity: ${f.opacity}
  claimed ratio: ${f.ratio}  verdict: ${f.verdict}  category: ${f.category}
  notes: ${f.notes}
${RAMP}
${HELPER}

STEPS:
1. Open the component at ${f.file} and confirm the element exists and renders this text. Confirm the actual foreground color from the source.
2. Independently determine the TRUE worst-case background: trace the DOM nesting — which surface/panel it sits in, whether it can be on a hover/selected row, what translucent layers stack behind it, and whether any ancestor or the element has opacity<1. Do NOT assume the claimed bg is right; derive it fresh. If the real worst case is darker (more contrast) or lighter (less contrast) than claimed, say so.
3. Run the helper CLI yourself for your re-derived fg + worst-case bg (+opacity). Report the exact command and ratio.
4. Decide: is this genuinely real-text/placeholder (must be AAA), or is it actually exempt (truly disabled control, or purely decorative)? Set exempt accordingly.
5. confirmed=true only if the original fg, worst-case bg, AND verdict all hold. Otherwise give the corrected values.

Return the structured verdict.`,
					{ label: `verify:${f.element.slice(0, 24)}`, phase: 'Verify', schema: VERDICT_SCHEMA },
				).then((v) => ({ finding: f, verdict: v })),
			),
		).then((verifications) => ({ surface: g.surface, findings: result.findings, verified: verifications.filter(Boolean) }))
	},
)

// Surface summary for the orchestrator
const summary = audited.filter(Boolean).map((s) => ({
	surface: s.surface,
	total: s.findings.length,
	failOrAA: s.findings.filter((f) => f.verdict !== 'AAA').length,
	confirmedProblems: s.verified
		.filter((v) => v.verdict && !v.verdict.exempt && (v.verdict.correctedVerdict === 'FAIL' || v.verdict.correctedVerdict === 'AA'))
		.map((v) => ({
			element: v.finding.element,
			file: v.finding.file,
			fg: v.verdict.correctedFg || v.finding.fg,
			bg: v.verdict.correctedBg || v.finding.bg,
			opacity: v.finding.opacity,
			ratio: v.verdict.correctedRatio,
			verdict: v.verdict.correctedVerdict,
			category: v.finding.category,
			reasoning: v.verdict.reasoning,
		})),
	disagreements: s.verified
		.filter((v) => v.verdict && v.verdict.confirmed === false)
		.map((v) => ({ element: v.finding.element, claimed: v.finding.ratio, corrected: v.verdict.correctedRatio, exempt: v.verdict.exempt, why: v.verdict.reasoning })),
}))

return { summary }
