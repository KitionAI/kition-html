import {readFile,writeFile} from 'node:fs/promises';
import {fileURLToPath} from 'node:url';
import path from 'node:path';

const root=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'..');
const run=process.argv[2];
if(!run||!/^[a-zA-Z0-9_.-]+$/.test(run))throw new Error('usage: node tools/finalize-report.mjs <run>');
const folder=path.join(root,'artifacts',run);
const native=JSON.parse(await readFile(path.join(folder,'report.json'),'utf8'));
const wasm=JSON.parse(await readFile(path.join(folder,'wasm/report.json'),'utf8'));
if((!process.argv.includes('--no-update-baseline') && native.cases.length<100)||wasm.cases.length!==native.cases.length)throw new Error('The full native and WASM corpus must finish first');
const differing=wasm.cases.filter(c=>c.nativeDifference?.changedPixels);
const totalDifferentPixels=differing.reduce((sum,c)=>sum+c.nativeDifference.changedPixels,0);
const cards=differing.map(c=>`<article><h2>${c.id}</h2><p>${c.nativeDifference.changedPixels} unequal pixels; maximum channel delta ${c.nativeDifference.maxChannelDelta}/255.</p><div><figure><figcaption>Native Rust</figcaption><img src="../${c.id}/rust.png"></figure><figure><figcaption>WASM in Worker</figcaption><img src="${c.id}.png"></figure><figure><figcaption>Exact difference (amplified)</figcaption><img src="${c.id}.diff.png"></figure></div></article>`).join('');
await writeFile(path.join(folder,'wasm/index.html'),`<!doctype html><html lang="en"><meta charset="utf-8"><title>Kition HTML Native/WASM comparison</title><style>body{font:14px/1.5 system-ui;background:#fafaf9;color:#1a1a1a;margin:24px}header,article{padding:20px;background:white;border:1px solid #e5e3df;border-radius:12px;margin-bottom:20px}article div{display:grid;grid-template-columns:repeat(3,minmax(0,1fr));gap:12px}figure{margin:0}img{width:100%;image-rendering:pixelated}a{color:#5645d4}@media(max-width:800px){article div{grid-template-columns:1fr}}</style><header><h1>Native Rust versus WASM Worker</h1><p>${wasm.summary.nativePixelExact}/${wasm.summary.total} pixel-exact; ${differing.length} differing cases; ${totalDifferentPixels} unequal pixels; ${wasm.summary.errors} errors.</p><p>These differences remain visible in strict diagnostics. Acceptance uses 99.9% content similarity; strict equality is optional. No pixels or channel differences are removed from the metrics. All cases, including exact matches, are in <a href="report.json">the JSON report</a>.</p><p><a href="../index.html">Chrome compatibility gallery</a></p></header>${cards}</html>`);
const gallery=await readFile(path.join(folder,'index.html'),'utf8');
if(!gallery.includes('wasm/index.html'))await writeFile(path.join(folder,'index.html'),gallery.replace('</header>',`<p><a href="wasm/index.html">Native/WASM: ${wasm.summary.nativePixelExact}/${wasm.summary.total} exact, ${totalDifferentPixels} unequal pixels</a></p></header>`));
const summary={run,environment:native.environment,nativeChrome:native.summary,wasm:{...wasm.summary,totalDifferentPixels,wasmSha256:wasm.wasmSha256},
  cases:native.cases.map(c=>({id:c.id,category:c.category,requires:c.requires,width:c.width,height:c.height,scale:c.scale,sourceSha256:c.sourceSha256,
    chrome:c.metrics,diagnostics:c.diagnostics,wasm:wasm.cases.find(w=>w.id===c.id)?.nativeDifference}))};
if(!process.argv.includes('--no-update-baseline'))await writeFile(path.join(root,'conformance/baseline.json'),`${JSON.stringify(summary,null,2)}\n`);
console.log(`Finalized ${run}: native/Chrome ${native.summary.pixelExact}/${native.summary.total}; native/WASM ${wasm.summary.nativePixelExact}/${wasm.summary.total}`);
