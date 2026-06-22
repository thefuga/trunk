import { contrast, compose, withOpacity, ratio, verdict } from "./contrast.mjs";
const mix=(c,p)=>`color-mix(in oklch, ${c} ${p}%, transparent)`;
const L7="oklch(0.76 0.02 75)";

// group opacity dims a composited (text-over-capsule-over-bg) toward bg? No:
// SVG group opacity composites the WHOLE group (capsule+text) over the backdrop behind the svg.
// Worst-case readability: text glyph vs the capsule directly under it, BOTH then composited at group α over backdrop.
// Group α scales fg and its local bg equally over the backdrop, so the TEXT-vs-CAPSULE ratio is what matters,
// plus the backdrop showing through. Approximate worst case: text α over (capsule α over backdrop).
function pillDim(alpha, capPct, base){
  const backdrop = compose(base, []);
  // capsule painted at (capPct% lane) then whole group at alpha over backdrop
  const capsule = compose(base, [mix(L7, capPct)]);            // capsule over backdrop (opaque approx)
  const effBg = base==="x"?capsule: capsule.map((c,i)=>alpha*c+(1-alpha)*backdrop[i]);
  const txt = L7;
  // text α over effBg, then group α over backdrop:
  const textOnCap = withOpacity(txt, 1, effBg);                // text fully opaque on capsule
  const grouped = textOnCap.map((c,i)=>alpha*c+(1-alpha)*backdrop[i]);
  return ratio(grouped, effBg);
}
console.log("## Graph search-dim SVG: pill text(lane-7) over capsule(14%) at GROUP opacity, on bg-1");
for(const a of [0.2,0.4,0.5,0.55,0.6,0.7]){
  const r=pillDim(a,14,"var(--bg-1)");
  console.log(`  group α=${a}: ${r.toFixed(2)} ${verdict(r)}`);
}

console.log("\n## Row/element opacity dims — text at α over opaque backdrop (new tokens)");
const FG1="var(--fg-1)", FG2="oklch(0.73 0.005 260)";
function solve(fg,base,layers,target){let lo=0,hi=1,b=1;for(let i=0;i<50;i++){const m=(lo+hi)/2;const r=contrast(fg,base,{layers,opacity:m});if(r>=target){b=m;hi=m}else lo=m}return b;}
const cases=[
  ["CommitRow search-dim msg fg-1 / bg-1", FG1,"var(--bg-1)",[]],
  ["CommitRow search-dim msg fg-1 / bg-selected", FG1,"var(--bg-selected)",[]],
  ["CommitRow search-dim author fg-2 / bg-1", FG2,"var(--bg-1)",[]],
  ["RebaseEditor drop msg fg-1 / bg-0", FG1,"var(--bg-0)",[]],
  ["RebaseEditor drop msg fg-1 / bg-selected", FG1,"var(--bg-selected)",[]],
  ["RebaseEditor drop date fg-2 / bg-selected", FG2,"var(--bg-selected)",[]],
  ["Orphaned fileref fg-2 / bg-1", FG2,"var(--bg-1)",[]],
  ["Orphaned excerpt fg-1 / del-tint bg-0", FG1,"var(--bg-0)",[mix("var(--err)",11)]],
];
for(const [l,fg,base,layers] of cases){
  console.log(`  ${l}: AA@α≥${solve(fg,base,layers,4.5).toFixed(2)}  (val@α=0.6 -> ${contrast(fg,base,{layers,opacity:0.6}).toFixed(2)})`);
}
