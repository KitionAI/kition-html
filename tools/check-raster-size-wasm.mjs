import assert from 'node:assert/strict';
import {readFile} from 'node:fs/promises';
import init,{HtmlDocument} from '../engine/kition_html_kit.js';

await init({module_or_path:await readFile(new URL('../engine/kition_html_kit_bg.wasm',import.meta.url))});
const fonts=await Promise.all(['Lora','JetBrainsMono'].map(async name=>[name,await readFile(new URL(`../conformance/assets/${name}-Variable.ttf`,import.meta.url))]));
const encode=text=>new TextEncoder().encode(text);
function frame(html,scale=1) {
 const doc=HtmlDocument.open('raster.html',encode(html));
 try {
  for(const [name,bytes] of fonts)doc.setResource(`/assets/${name}.ttf`,bytes);
  doc.setViewport(384/scale,192/scale,false);
  const pixels=doc.render(0,0,scale,384,192).slice();
  assert.deepEqual(JSON.parse(doc.check()).diagnostics,[]);
  assert.ok(pixels.some((v,i)=>i%4!==3&&v<128),'glyph must be visible');
  assert.equal(new TextDecoder().decode(doc.save()),html);
  assert.equal(doc.dirty,false);
  return pixels;
 } finally {doc.free();}
}
function canvas(family,size,scale) {
 return frame(`<!doctype html><style>body{margin:0}canvas{display:block}</style><canvas id=c width=384 height=192></canvas><script>const ctx=document.getElementById('c').getContext('2d');ctx.font="${size}px '${family}'";ctx.scale(${scale},${scale});ctx.fillText('H',48/${scale},96/${scale});</script>`);
}
function same(a,b,context) {
 assert.equal(a.length,b.length);
 let max=0,changed=0;
 for(let i=0;i<a.length;i++){const delta=Math.abs(a[i]-b[i]);max=Math.max(max,delta);changed+=delta>0;}
 assert.ok(max<=1,`${context}: ${changed} changed channels; maximum delta ${max}`);
}
let checked=0;
for(const family of ['Lora','JetBrains Mono'])for(const size of [12,16,24,32]) {
 for(const scale of [.75,1.25,2]) {
  same(canvas(family,size,scale),canvas(family,size*scale,1),`${family} ${size}px Canvas ${scale}`);checked++;
 }
 for(const scale of [1,1.5,2]) {
  const html=`<!doctype html><style>body{margin:0}div{position:absolute;left:${48/scale}px;top:0;font:${size}px/48px '${family}'}i{display:inline-block;width:0;height:0}</style><div id=line>H<i id=baseline></i></div><script>const line=document.getElementById('line'),marker=document.getElementById('baseline');line.style.top=(${96/scale}-marker.getBoundingClientRect().y)+'px';</script>`;
  same(frame(html,scale),canvas(family,size*scale,1),`${family} ${size}px DOM DPR ${scale}`);checked++;
 }
}
console.log(`${checked}/48 WASM raster-size equivalences passed`);
