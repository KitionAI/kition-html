import {chromium} from '@playwright/test';
import {readFile} from 'node:fs/promises';
import {PNG} from 'pngjs';
import assert from 'node:assert/strict';

// Independent Chrome oracle. Whole bitmaps must match, with no alignment or masks.
// Inter is excluded because its optical-size axis changes outlines with CSS size.
const root=new URL('../',import.meta.url);
const browser=await chromium.launch({headless:true,args:['--force-color-profile=srgb']});
let canvasChecks=0,domChecks=0;
try {
 for(const family of ['Lora','JetBrainsMono']) {
  const bytes=await readFile(new URL(`conformance/assets/${family}-Variable.ttf`,root));
  const face=`@font-face{font-family:Probe;src:url(data:font/ttf;base64,${bytes.toString('base64')})}`;
  const reference=await browser.newPage({viewport:{width:384,height:192}});
  await reference.setContent(`<style>${face}body{margin:0}canvas{display:block}</style><canvas width="384" height="192"></canvas>`);
  for(const size of [12,16,24,32]) {
   for(const scale of [.75,1.25,2]) {
    const match=await reference.evaluate(async({size,scale})=>{
     await document.fonts.load(`${size}px Probe`);await document.fonts.load(`${size*scale}px Probe`);
     const c=document.querySelector('canvas'),ctx=c.getContext('2d'),frames=[];
     for(const direct of [false,true]) {
      c.width=c.width;ctx.font=`${direct?size*scale:size}px Probe`;
      if(!direct)ctx.scale(scale,scale);
      ctx.fillText('H',direct?48:48/scale,direct?96:96/scale);
      frames.push(ctx.getImageData(0,0,384,192).data);
     }
     return frames[0].every((v,i)=>v===frames[1][i]);
    },{size,scale});
    assert.ok(match,`${family} ${size}px Canvas scale ${scale}`);canvasChecks++;
   }
   for(const dpr of [1,1.5,2]) {
    await reference.evaluate(async size=>{
     await document.fonts.load(`${size}px Probe`);
     const c=document.querySelector('canvas');c.width=c.width;
     const ctx=c.getContext('2d');ctx.font=`${size}px Probe`;ctx.fillText('H',48,96);
    },size*dpr);
    const expected=PNG.sync.read(await reference.screenshot());
    const page=await browser.newPage({viewport:{width:384/dpr,height:192/dpr},deviceScaleFactor:dpr});
    try {
     await page.setContent(`<style>${face}body{margin:0}div{position:absolute;left:${48/dpr}px;top:0;font:${size}px/48px Probe}i{display:inline-block;width:0;height:0}</style><div>H<i></i></div>`);
     const baseline=await page.evaluate(async({size,dpr})=>{
      await document.fonts.load(`${size}px Probe`);
      const node=document.querySelector('div'),marker=document.querySelector('i');
      node.style.top=(96/dpr-marker.getBoundingClientRect().y)+'px';
      return marker.getBoundingClientRect().y*dpr;
     },{size,dpr});
     assert.equal(baseline,96,'fixture baseline must be exactly representable in CSS layout units');
     const actual=PNG.sync.read(await page.screenshot());
     assert.ok(actual.data.equals(expected.data),`${family} ${size}px DOM DPR ${dpr}`);domChecks++;
    } finally {await page.close();}
   }
  }
  await reference.close();
 }
} finally {await browser.close();}
console.log(`${canvasChecks}/24 Canvas scale and ${domChecks}/24 DOM DPR Chrome equivalences passed`);
