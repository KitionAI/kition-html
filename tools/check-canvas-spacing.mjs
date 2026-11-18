import assert from 'node:assert/strict';
import {readFile} from 'node:fs/promises';
import {chromium} from '@playwright/test';

// Read-only Chrome oracle: never rewrite the recorded measurements or fixtures.
const root=new URL('../',import.meta.url);
const states=JSON.parse(await readFile(new URL('conformance/canvas-spacing-cases.json',root),'utf8'));
const metrics=JSON.parse(await readFile(new URL('conformance/canvas-spacing-metrics.json',root),'utf8'));
const browser=await chromium.launch({headless:true,args:['--force-color-profile=srgb']});
try {
 const page=await browser.newPage();
 const inter=await readFile(new URL('conformance/assets/Inter-Variable.ttf',root));
 for(const sample of states){
  await page.setContent(`<style>@font-face{font-family:Inter;src:url(data:font/ttf;base64,${inter.toString('base64')})}</style><canvas id="canvas"></canvas>`);
  const result=await page.evaluate(async script=>{await document.fonts.load('32px Inter');const canvas=document.getElementById('canvas'),ctx=canvas.getContext('2d');ctx.font='32px Inter';return eval(`(()=>{${script}})()`);},sample.script);
  assert.equal(result,true,sample.id);
 }
 const grammar=JSON.parse(await readFile(new URL('conformance/canvas-spacing-grammar.json',root),'utf8'));
 await page.evaluate(grammar=>{const ctx=document.querySelector('canvas').getContext('2d');for(const prop of ['letterSpacing','wordSpacing'])for(const [input,expected] of Object.entries(grammar)){ctx[prop]='7px';ctx[prop]=input;if(ctx[prop]!==expected)throw Error(prop+' '+input+' '+ctx[prop]+' != '+expected)}},grammar);
 for(const family of ['Inter','Lora','JetBrainsMono']){
  const bytes=await readFile(new URL(`conformance/assets/${family}-Variable.ttf`,root));
  await page.setContent(`<style>@font-face{font-family:Probe;src:url(data:font/ttf;base64,${bytes.toString('base64')})}</style><canvas></canvas>`);
  for(const expected of metrics.filter(row=>row.family===family)){
   const actual=await page.evaluate(async sample=>{await document.fonts.load('32px Probe');const ctx=document.querySelector('canvas').getContext('2d');ctx.font='32px Probe';ctx.letterSpacing=sample.letter+'px';ctx.wordSpacing=sample.word+'px';return ctx.measureText(sample.text??String.fromCodePoint(...sample.codePoints)).width;},expected);
   assert.equal(actual,expected.width,`${family} ${expected.text??expected.codePoints} ${expected.letter}/${expected.word}`);
  }
 }
 console.log(`${states.length}/${states.length} Chrome spacing state cases, ${Object.keys(grammar).length*2} grammar assignments and ${metrics.length}/${metrics.length} metric profiles passed`);
} finally {await browser.close();}
