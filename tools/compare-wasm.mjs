import {chromium} from '@playwright/test';
import {PNG} from 'pngjs';
import {createServer} from 'node:http';
import {readFile,writeFile,mkdir,access} from 'node:fs/promises';
import {fileURLToPath} from 'node:url';
import {createHash} from 'node:crypto';
import path from 'node:path';
import {comparePixels} from './pixels.mjs';
import {similarityPolicy,summarizeSimilarity} from './similarity.mjs';

const root=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'..');
const run=process.argv[2];
if(!run||!/^[a-zA-Z0-9_.-]+$/.test(run))throw new Error('usage: node tools/compare-wasm.mjs <native-run-id>');
const baseline=path.join(root,'artifacts',run);
// The native report is checkpointed after every case; its gallery is written
// only when all selected cases finish. Never consume an in-progress snapshot.
await access(path.join(baseline,'index.html')).catch(()=>{
  throw new Error('Native capture has not finished: wait for its final gallery before comparing WASM');
});
const report=JSON.parse(await readFile(path.join(baseline,'report.json'),'utf8'));
const out=path.join(baseline,'wasm');
await mkdir(out);
const assets=[];
for(const name of Object.keys(report.environment.assetSha256)){
  assets.push([name,[...await readFile(path.join(baseline,'assets',name))]]);
}
const server=createServer(async(req,res)=>{
  try{
    const pathname=new URL(req.url,'http://127.0.0.1').pathname;
    if(pathname==='/'){res.writeHead(200,{'Content-Type':'text/html'});res.end('<!doctype html><title>Worker conformance host</title>');return;}
    if(!pathname.startsWith('/engine/')&&pathname!=='/tools/render.worker.mjs')throw new Error('Not found');
    const file=path.resolve(root,`.${pathname}`);
    if(!file.startsWith(`${root}${path.sep}`))throw new Error('Not found');
    res.writeHead(200,{'Content-Type':file.endsWith('.wasm')?'application/wasm':'text/javascript','Cache-Control':'public,max-age=3600'});
    res.end(await readFile(file));
  }catch{res.writeHead(404);res.end();}
});
await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));
let browser;
const results=[];
const wasmReport={similarityPolicy,wasmSha256:createHash('sha256').update(await readFile(path.join(root,'engine/kition_html_kit_bg.wasm'))).digest('hex'),
  execution:'dedicated browser Web Worker; CPU rendering; no browser DOM layout',cases:results};
try{
  browser=await chromium.launch({headless:true});
  const page=await browser.newPage();
  await page.goto(`http://127.0.0.1:${server.address().port}`);
  await page.evaluate(async()=>{
    window.kitionWorker=new Worker('/tools/render.worker.mjs',{type:'module'});
    await new Promise((resolve,reject)=>{
      const timer=setTimeout(()=>reject(new Error('WASM initialization timed out')),90000);
      window.kitionWorker.onmessage=event=>{if(event.data.ready){clearTimeout(timer);resolve();}};
      window.kitionWorker.onerror=event=>{clearTimeout(timer);reject(new Error(event.message));};
    });
  });
  await page.evaluate(assets=>new Promise(resolve=>{
    window.kitionWorker.onmessage=event=>{if(event.data.configured)resolve();};
    window.kitionWorker.postMessage({assets});
  }),assets);
  for(const [index,test] of report.cases.entries()){
    const result={id:test.id};results.push(result);
    try{
      if(!test.metrics)throw new Error('No native baseline');
      const html=await readFile(path.join(baseline,test.id,'source.html'),'utf8');
      const encoded=await page.evaluate(async({html,options})=>{
        return await new Promise((resolve,reject)=>{
          const timer=setTimeout(()=>reject(new Error('WASM render timed out')),30000);
          window.kitionWorker.onmessage=event=>{
            clearTimeout(timer);
            if(event.data.error)reject(new Error(event.data.error));
            else {
              const data=event.data;
              const bytes=new Uint8Array(data.rgba??data.png);
              let binary='';
              for(let offset=0;offset<bytes.length;offset+=32768){
                binary+=String.fromCharCode(...bytes.subarray(offset,offset+32768));
              }
              resolve({raw:!!data.rgba,width:data.width,height:data.height,base64:btoa(binary)});
            }
          };
          window.kitionWorker.onerror=event=>{clearTimeout(timer);reject(new Error(event.message));};
          window.kitionWorker.postMessage({html,options});
        });
      },{html,options:test});
      const bytes=Buffer.from(encoded.base64,'base64');
      const actual=encoded.raw?{width:encoded.width,height:encoded.height,data:bytes}:PNG.sync.read(bytes);
      const native=PNG.sync.read(await readFile(path.join(baseline,test.id,'rust.png')));
      const chrome=PNG.sync.read(await readFile(path.join(baseline,test.id,'chrome.png')));
      const comparison=comparePixels(native,actual);
      result.nativeDifference=comparison.metrics;
      result.chromeDifference=comparePixels(chrome,actual).metrics;
      await writeFile(path.join(out,`${test.id}.png`),PNG.sync.write(actual));
      await writeFile(path.join(out,`${test.id}.diff.png`),PNG.sync.write({...actual,data:comparison.diff}));
      console.log(`[${index+1}/${report.cases.length}] ${test.id}: ${comparison.metrics.changedPixels} native/WASM pixels differ`);
    }catch(error){result.error=String(error.message);console.log(`[${index+1}/${report.cases.length}] ${test.id}: ERROR ${result.error.slice(0,160)}`);}
    wasmReport.summary={total:results.length,errors:results.filter(c=>c.error).length,
      nativeSimilarity:summarizeSimilarity(results.map(c=>c.nativeDifference)),
      chromeSimilarity:summarizeSimilarity(results.map(c=>c.chromeDifference)),
      nativePixelExact:results.filter(c=>c.nativeDifference?.changedPixels===0).length,
      chromePixelExact:results.filter(c=>c.chromeDifference?.changedPixels===0).length};
    await writeFile(path.join(out,'report.json'),`${JSON.stringify(wasmReport,null,2)}\n`);
  }
  console.log(JSON.stringify(wasmReport.summary,null,2));
  if(wasmReport.summary.errors||wasmReport.summary.nativeSimilarity.failed
      || (process.argv.includes('--require-exact') && wasmReport.summary.nativePixelExact!==results.length))process.exitCode=1;
}finally{
  if(browser)await browser.close();
  await new Promise(resolve=>server.close(resolve));
}
