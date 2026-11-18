import {chromium} from '@playwright/test';
import {PNG} from 'pngjs';
import {readFile} from 'node:fs/promises';
import assert from 'node:assert/strict';

// Diagnostic only: group single-glyph masks inside Chrome to identify phases.
// Full-image conformance never crops, aligns, masks, or excludes any pixel.
const root = new URL('../', import.meta.url);
const browser = await chromium.launch({headless:true, args:['--force-color-profile=srgb']});
let passed = 0;
try {
  for (const dpr of [1,1.5,2]) {
    const page = await browser.newPage({viewport:{width:800,height:160}, deviceScaleFactor:dpr});
    for (const family of ['Inter','Lora','JetBrainsMono']) for (const size of [12,16,24]) {
      const bytes = await readFile(new URL(`conformance/assets/${family}-Variable.ttf`, root));
      await page.setContent(`<style>@font-face{font-family:Probe;src:url(data:font/ttf;base64,${bytes.toString('base64')})}body{margin:0}span{position:absolute;top:20px;font:${size}px/48px Probe;color:black}</style>`);
      const origins = await page.evaluate(async ({size,dpr}) => {
        await document.fonts.load(`${size}px Probe`);
        for (let i=0; i<16; i++) {
          const span = document.createElement('span');
          span.textContent = 'H';
          span.style.left = `${16+i*40+i/(16*dpr)}px`;
          document.body.appendChild(span);
        }
        return [...document.querySelectorAll('span')].map(span => span.getBoundingClientRect().x*dpr);
      }, {size,dpr});
      const png = PNG.sync.read(await page.screenshot());
      const masks = [];
      for (let i=0; i<16; i++) {
        const pixels = [];
        for (let y=20*dpr; y<68*dpr; y++) for (let x=0; x<32*dpr; x++) {
          pixels.push(png.data[(y*png.width+(16+i*40)*dpr+x)*4]);
        }
        masks.push(pixels.join(','));
      }
      const classes = masks.map(mask => masks.indexOf(mask));
      // Use measured CSS positions: Blink's 1/64 layout units matter at DPR 1.5.
      const phases = origins.map((x,i) => Math.floor((x-(16+i*40)*dpr)*4+.5));
      const expected = phases.map(phase => phases.indexOf(phase));
      assert.deepEqual(classes, expected, `${family} ${size}px at DPR ${dpr}`);
      passed++;
      console.log(JSON.stringify({dpr,family,size,classes,expected,match:true}));
    }
    await page.close();
  }
} finally {
  await browser.close();
}
console.log(`${passed}/27 Chrome glyph phase profiles passed`);
