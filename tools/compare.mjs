import { chromium } from '@playwright/test';
import { PNG } from 'pngjs';
import { createServer } from 'node:http';
import { readFile, writeFile, mkdir, readdir } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import path from 'node:path';
import { comparePixels } from './pixels.mjs';
import { summarize, reportHtml } from './report.mjs';
import {similarityPolicy} from './similarity.mjs';
import {captureCases} from './viewport.mjs';

const root=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'..');
const conformance=path.join(root,'conformance');
const args=process.argv.slice(2);
function option(name,fallback){const i=args.indexOf(name);return i<0?fallback:args[i+1];}
const runId=option('--run',new Date().toISOString().replaceAll(':','-'));
if(!/^[a-zA-Z0-9_.-]+$/.test(runId))throw new Error('Run id must be a portable directory name');
const out=path.join(root,'artifacts',runId);
// Each run is immutable; retries need a new run id.
await mkdir(path.join(root,'artifacts'),{recursive:true});
await mkdir(out,{recursive:false});
const binary=path.resolve(root,option('--binary','target/debug/kition-html'));
const corpus=option('--manifest','manifest.json');
const manifestPath=path.resolve(conformance,corpus);
if(!manifestPath.startsWith(conformance+path.sep))throw new Error('Manifest must be inside conformance');
const manifestBytes=await readFile(manifestPath);
const manifest=JSON.parse(manifestBytes);
const filter=option('--filter','');
const selected=captureCases(manifest.filter(c=>c.id.includes(filter)),args);
if(!selected.length)throw new Error('No matching cases');
if(corpus==='manifest.json' && !filter && selected.length<100)throw new Error('Full comparison requires at least 100 cases');
const sha=bytes=>createHash('sha256').update(bytes).digest('hex');
const portableError=text=>String(text).replaceAll(root,'<repo-root>').replaceAll(process.env.HOME??'\0','<home>');
const assets={};
for(const file of (await readdir(path.join(conformance,'assets'))).sort()){
  const bytes=await readFile(path.join(conformance,'assets',file));
  assets[file]=sha(bytes);
  await mkdir(path.join(out,'assets'),{recursive:true});
  await writeFile(path.join(out,'assets',file),bytes);
}
const server=createServer(async(req,res)=>{
  try{
    const pathname=decodeURIComponent(new URL(req.url,'http://127.0.0.1').pathname);
    const target=path.resolve(conformance,`.${pathname}`);
    if(!target.startsWith(`${conformance}${path.sep}`))throw new Error('outside fixtures');
    const bytes=await readFile(target);
    const ext=path.extname(target);
    const mime={'.html':'text/html; charset=utf-8','.css':'text/css','.svg':'image/svg+xml','.ttf':'font/ttf','.png':'image/png','.js':'text/javascript','.json':'application/json'}[ext]??'application/octet-stream';
    res.writeHead(200,{'Content-Type':mime,'Cache-Control':'no-store'});res.end(bytes);
  }catch{res.writeHead(404);res.end('Not found');}
});
await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));
const origin=`http://127.0.0.1:${server.address().port}`;
let browser;
const results=[];
const report={similarityPolicy,environment:{corpus,manifestSha256:sha(manifestBytes),blitzRevision:'21edc92d90d8f8bedadd3f9ecb14b1ad950952eb',
  renderer:'vello-cpu (filters enabled)',platform:process.platform,architecture:process.arch,
  binarySha256:sha(await readFile(binary)),assetSha256:assets,playwright:'1.59.1',
  cargoLockSha256:sha(await readFile(path.join(root,'Cargo.lock'))),
  javascript:'Chrome enabled; Rust Boa with supplied resources and a 16 ms initial virtual-time frame',comparison:'strict RGBA; no exclusions; significant means max channel delta > 16'},cases:results};
try{
  browser=await chromium.launch({headless:true,args:['--force-color-profile=srgb']});
  report.environment.chrome=browser.version();
  for(const [index,test] of selected.entries()){
    const folder=path.join(out,test.id);
    await mkdir(folder);
    const result={...test,sourceSha256:sha(await readFile(path.join(conformance,test.file)))};
    await writeFile(path.join(folder,'source.html'),await readFile(path.join(conformance,test.file)));
    results.push(result);
    const context=await browser.newContext({viewport:{width:test.width,height:test.height},deviceScaleFactor:test.scale,
      colorScheme:test.dark?'dark':'light',locale:'en-US',timezoneId:'UTC',reducedMotion:'reduce',serviceWorkers:'block'});
    const page=await context.newPage();
    const failures=[];
    page.on('pageerror',error=>failures.push(error.message));
    await context.route('**/*',route=>{
      const url=route.request().url();
      if(url.startsWith(`${origin}/`)||url.startsWith('data:'))return route.continue();
      failures.push(`blocked external resource: ${new URL(url).protocol}`);
      return route.abort();
    });
    try{
      await page.goto(`${origin}/${test.file}`,{waitUntil:'load',timeout:15000});
      if(test.ready)await page.locator(test.ready).waitFor({state:'attached',timeout:15000});
      await page.evaluate(async()=>{
        await Promise.all(['16px Inter','16px Lora','16px "JetBrains Mono"'].map(font=>document.fonts.load(font)));
        await document.fonts.ready;
        await Promise.all([...document.images].map(img=>img.decode()));
        await new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(resolve)));
      });
      if(failures.length)throw new Error(`Chrome fixture errors: ${failures.join('; ')}`);
      result.chromeGeometry=await page.locator('#stage').evaluate(stage=>[...stage.querySelectorAll('*')].filter(e=>!['SCRIPT','STYLE','DEFS'].includes(e.tagName)).map(e=>{
        const r=e.getBoundingClientRect();return {tag:e.tagName,id:e.id||undefined,class:e.getAttribute('class')||undefined,x:r.x,y:r.y,width:r.width,height:r.height};
      }));
      const chromeBytes=await page.screenshot({path:path.join(folder,'chrome.png'),animations:'disabled',caret:'hide',scale:'device'});
      // Repeated references detect hidden timing/font instability in every fixture.
      const chromeAgain=await page.screenshot({animations:'disabled',caret:'hide',scale:'device'});
      result.referenceStable=comparePixels(PNG.sync.read(chromeBytes),PNG.sync.read(chromeAgain)).metrics.changedPixels===0;
      if(!result.referenceStable)throw new Error('Chrome reference is not stable across consecutive frames');
      const command=['render',path.join(conformance,test.file),'--output',path.join(folder,'rust.png'),
        '--assets',path.join(conformance,'assets'),'--width',String(test.width),'--height',String(test.height),'--scale',String(test.scale)];
      if(test.dark)command.push('--dark');
      const start=performance.now();
      const rust=spawnSync(binary,command,{encoding:'utf8',timeout:30000,maxBuffer:2*1024*1024});
      result.nativeElapsedMs=Math.round(performance.now()-start);
      if(rust.status!==0)throw new Error(`Rust render failed (${rust.signal??rust.status}): ${portableError(rust.stderr||rust.error?.message||'unknown error').slice(0,2000)}`);
      const metadata=JSON.parse(rust.stdout.trim().split('\n').at(-1));
      result.diagnostics=metadata.diagnostics;
      result.rustGeometry=metadata.geometry;
      const reference=PNG.sync.read(chromeBytes);
      const actual=PNG.sync.read(await readFile(path.join(folder,'rust.png')));
      const {metrics,diff,overlay}=comparePixels(reference,actual);
      result.metrics=metrics;
      for(const [name,data] of [['diff',diff],['overlay',overlay]]){
        await writeFile(path.join(folder,`${name}.png`),PNG.sync.write({width:reference.width,height:reference.height,data}));
      }
      console.log(`[${index+1}/${selected.length}] ${test.id}: ${metrics.changedPixels===0?'EXACT':`${(metrics.changedRatio*100).toFixed(2)}% different`}`);
    }catch(error){
      result.error=portableError(error.message);
      console.log(`[${index+1}/${selected.length}] ${test.id}: ERROR ${result.error.slice(0,160)}`);
    }finally{await context.close();}
    await writeFile(path.join(folder,'metrics.json'),`${JSON.stringify(result,null,2)}\n`);
    report.summary=summarize(results);
    await writeFile(path.join(out,'report.json'),`${JSON.stringify(report,null,2)}\n`);
  }
  await writeFile(path.join(out,'index.html'),reportHtml(report));
  console.log(JSON.stringify(report.summary,null,2));
  console.log(`Report: artifacts/${runId}/index.html`);
  if(report.summary.errors || (!args.includes('--measure-only') && report.summary.similarity.failed)
      || (args.includes('--require-exact') && report.summary.different))process.exitCode=1;
}finally{
  if(browser)await browser.close();
  await new Promise(resolve=>server.close(resolve));
}
