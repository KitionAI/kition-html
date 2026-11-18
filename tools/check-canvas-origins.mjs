import {chromium} from '@playwright/test';
import {readFile} from 'node:fs/promises';
import assert from 'node:assert/strict';

// Diagnose Canvas bitmap phases using only Chrome. This does not crop, align,
// mask, or otherwise alter the full-image conformance acceptance comparison.
const root=new URL('../',import.meta.url);
const browser=await chromium.launch({headless:true,args:['--force-color-profile=srgb']});
let passed=0;
try {
 const page=await browser.newPage();
 for(const family of ['Inter','Lora','JetBrainsMono']){
  const bytes=await readFile(new URL(`conformance/assets/${family}-Variable.ttf`,root));
  await page.setContent(`<style>@font-face{font-family:Probe;src:url(data:font/ttf;base64,${bytes.toString('base64')})}</style><canvas width="1200" height="160"></canvas>`);
  for(const size of [12,16,24])for(const scale of [1,1.5,2])for(const axis of ['x','y'])for(const operation of ['fillText','strokeText']){
   const classes=await page.evaluate(async({size,scale,axis,operation})=>{
    await document.fonts.load(`${size}px Probe`);
    const canvas=document.querySelector('canvas'),ctx=canvas.getContext('2d');
    canvas.width=canvas.width;ctx.font=`${size}px Probe`;ctx.scale(scale,scale);
    for(let i=0;i<16;i++)ctx[operation]('H',(16+i*68+(axis==='x'?i/16:0))/scale,(80+(axis==='y'?i/16:0))/scale);
    const masks=[];
    for(let i=0;i<16;i++)masks.push([...ctx.getImageData(16+i*68-4,16,64,90).data].join(','));
    return masks.map(mask=>masks.indexOf(mask));
   },{size,scale,axis,operation});
   const phases=Array.from({length:16},(_,i)=>axis==='x'?Math.floor(i/4+.5):Math.floor(i/16+.5));
   const expected=phases.map(phase=>phases.indexOf(phase));
   assert.deepEqual(classes,expected,`${family} ${size}px scale ${scale}, ${axis}, ${operation}`);
   passed++;
   console.log(JSON.stringify({family,size,scale,axis,operation,classes}));
  }
 }
} finally {await browser.close();}
console.log(`${passed}/108 Chrome Canvas origin profiles passed`);
