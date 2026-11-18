import assert from 'node:assert/strict';
import {readFile} from 'node:fs/promises';
import {chromium} from '@playwright/test';

// Read-only Chrome oracle: never rewrite the recorded measurements or fixtures.
const root=new URL('../',import.meta.url);
const states=JSON.parse(await readFile(new URL('conformance/canvas-state-cases.json',root),'utf8'));
const browser=await chromium.launch({headless:true,args:['--force-color-profile=srgb']});
try {
 const page=await browser.newPage();
 const inter=await readFile(new URL('conformance/assets/Inter-Variable.ttf',root));
 for(const sample of states){
  await page.setContent(`<style>@font-face{font-family:Inter;src:url(data:font/ttf;base64,${inter.toString('base64')})}</style><canvas id="canvas"></canvas>`);
  const result=await page.evaluate(async script=>{await document.fonts.load('32px Inter');const canvas=document.getElementById('canvas'),ctx=canvas.getContext('2d');ctx.font='32px Inter';return eval(`(()=>{${script}})()`);},sample.script);
  assert.equal(result,true,sample.id);
 }
 console.log(`${states.length}/${states.length} Chrome Canvas numeric state cases passed`);
} finally {await browser.close();}
