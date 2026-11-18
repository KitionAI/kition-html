import {readFile,writeFile,mkdir} from 'node:fs/promises';
import {createHash} from 'node:crypto';
import {fileURLToPath} from 'node:url';
import path from 'node:path';
import {PNG} from 'pngjs';
import {comparePixels} from './pixels.mjs';
import {similarityPolicy,passesSimilarity,summarizeSimilarity} from './similarity.mjs';

const root=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'..');
const [run,...rawArgs]=process.argv.slice(2);
const updateBaseline=!rawArgs.includes('--no-update-baseline');
const args=rawArgs.filter(arg=>arg!=='--no-update-baseline');
const required=args[0]??'all';
if(!run||!/^[a-zA-Z0-9_-][a-zA-Z0-9_.-]*$/.test(run)
    || !['all','chrome','native-wasm'].includes(required) || args.length>1){
  throw new Error('usage: node tools/assess-similarity.mjs <run> [all|chrome|native-wasm] [--no-update-baseline]');
}
const folder=path.join(root,'artifacts',run);
const readJson=async file=>JSON.parse(await readFile(file,'utf8'));
const native=await readJson(path.join(folder,'report.json'));
const wasm=await readJson(path.join(folder,'wasm/report.json'));
if((!process.argv.includes('--no-update-baseline') && native.cases.length<100)||wasm.cases.length!==native.cases.length){
  throw new Error('Assessment requires the complete native and WASM corpus');
}
const ids=native.cases.map(c=>c.id);
if(new Set(ids).size!==ids.length||new Set(wasm.cases.map(c=>c.id)).size!==ids.length
    || wasm.cases.some(c=>!ids.includes(c.id))
    || ids.some(id=>!/^[a-zA-Z0-9_-]+$/.test(id))){
  throw new Error('Case identifiers must be unique, matching and portable');
}
const destination=path.join(folder,`similarity-999-${required}`);
await mkdir(destination);
const sha=bytes=>createHash('sha256').update(bytes).digest('hex');
const results=[];
for(const entry of native.cases){
  const result={id:entry.id,category:entry.category};
  results.push(result);
  try{
    if(entry.error||wasm.cases.find(c=>c.id===entry.id)?.error){
      throw new Error('Captured run contains a rendering error');
    }
    const files=[`${entry.id}/chrome.png`,`${entry.id}/rust.png`,`wasm/${entry.id}.png`];
    const bytes=await Promise.all(files.map(file=>readFile(path.join(folder,file))));
    result.imageSha256=Object.fromEntries(files.map((file,index)=>[file,sha(bytes[index])]));
    const [chrome,rust,worker]=bytes.map(data=>PNG.sync.read(data));
    result.nativeChrome=comparePixels(chrome,rust).metrics;
    result.wasmChrome=comparePixels(chrome,worker).metrics;
    result.nativeWasm=comparePixels(rust,worker).metrics;
  }catch{
    result.error='Unable to assess all three captured images; inspect the source run.';
  }
}
const targets=['nativeChrome','wasmChrome','nativeWasm'];
const summary=Object.fromEntries(targets.map(key=>[key,summarizeSimilarity(results.map(c=>c[key]))]));
const report={run,similarityPolicy,required,summary,cases:results};
await writeFile(path.join(destination,'report.json'),`${JSON.stringify(report,null,2)}\n`);
const escape=value=>String(value).replaceAll('&','&amp;').replaceAll('<','&lt;').replaceAll('>','&gt;').replaceAll('"','&quot;');
const score=value=>Number.isFinite(value)?`${(value*100).toFixed(6)}%`:'Error';
const cell=metrics=>metrics?`<td>${score(metrics.foregroundSimilarity)}<br><small>Canvas ${score(metrics.similarity)}</small><br><b>${passesSimilarity(metrics)?'PASS':'FAIL'}</b></td>`:'<td>ERROR</td>';
const rows=[...results].sort((a,b)=>(a.nativeChrome?.foregroundSimilarity??-1)-(b.nativeChrome?.foregroundSimilarity??-1))
  .map(c=>`<tr><th><a href="../${c.id}/overlay.png">${escape(c.id)}</a><br><small>${escape(c.category)}</small></th>${targets.map(key=>cell(c[key])).join('')}</tr>`).join('');
await writeFile(path.join(destination,'index.html'),`<!doctype html><html lang="en"><meta charset="utf-8"><title>Kition HTML 99.9% acceptance</title><style>body{font:14px/1.5 system-ui;margin:24px;background:#fafaf9;color:#1a1a1a}main{padding:20px;background:white;border:1px solid #e5e3df;border-radius:12px;overflow:auto}h1{margin-top:0}table{border-collapse:collapse;width:100%}th,td{text-align:left;padding:12px;border-bottom:1px solid #e5e3df}small{color:#5d5b54}a{color:#5645d4}</style><main><h1>99.9% similarity acceptance</h1><p>Every case must reach 99.9% on the union of non-white pixels. Score = 1 minus mean maximum RGBA channel error / 255. Full-canvas scores are diagnostic. This is a color-error metric, not SSIM or a measure of functional compatibility.</p><p>No rounding before acceptance, alignment, antialias masks, or excluded cases. Empty identical white images score 100%; render errors fail. Exact differences remain in the original galleries.</p><ul>${targets.map(key=>`<li>${key}: ${summary[key].passed}/${summary[key].total} pass</li>`).join('')}</ul><p>Reassessed existing PNGs from ${escape(run)}; no renderer changes or new screenshots. <a href="report.json">Metrics and image hashes</a> · <a href="../index.html">Chrome gallery</a> · <a href="../wasm/index.html">Native/WASM gallery</a></p><table><thead><tr><th>Feature</th><th>Native / Chrome</th><th>WASM / Chrome</th><th>Native / WASM</th></tr></thead><tbody>${rows}</tbody></table></main></html>`);
if(updateBaseline)await writeFile(path.join(root,'conformance/similarity-baseline.json'),`${JSON.stringify({run,similarityPolicy,summary},null,2)}\n`);
console.log(JSON.stringify(summary,null,2));
console.log(`Report: artifacts/${run}/similarity-999-${required}/index.html`);
const gates=required==='native-wasm'?['nativeWasm']:required==='chrome'?['nativeChrome','wasmChrome']:targets;
if(gates.some(key=>summary[key].failed))process.exitCode=1;
