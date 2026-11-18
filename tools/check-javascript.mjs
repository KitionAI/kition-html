// Exercise the built WASM SDK directly; browser Worker wiring is covered by host E2E.
import assert from 'node:assert/strict';
import {readFile} from 'node:fs/promises';
import init, {HtmlDocument} from '../engine/kition_html_kit.js';

await init({module_or_path:await readFile(new URL('../engine/kition_html_kit_bg.wasm',import.meta.url))});
const encode=text=>new TextEncoder().encode(text);
const source=script=>`<!doctype html><style>html,body{margin:0;height:100%;background:red}</style><body><script>${script}</script></body>`;
const purple=[86,69,212,255];
const frame=doc=>Array.from(doc.render(0,0,1,8,8).slice(0,4));
const diagnostics=doc=>JSON.parse(doc.check()).diagnostics;
let checked=0;
function check(html,run){
  const doc=HtmlDocument.open('script.html',encode(html));
  try {doc.setViewport(64,64,false);run(doc);checked++;} finally {doc.free();}
}
check(source("Promise.resolve().then(()=>document.body.style.background='#5645d4')"),doc=>{
  assert.deepEqual(frame(doc),purple);
  assert.deepEqual(diagnostics(doc),[]);
});
check(source("let n=0;document.body.onclick=()=>{document.body.style.background=++n===2?'#5645d4':'blue'}"),doc=>{
  doc.execute('html.click','{"x":4,"y":4}');
  doc.render(0,0,2,8,8);
  doc.execute('html.click','{"x":4,"y":4}');
  assert.deepEqual(frame(doc),purple);
  assert.equal(doc.dirty,false);
  assert.match(new TextDecoder().decode(doc.save()),/let n=0/);
});
check(source("setTimeout(()=>document.body.style.background='#5645d4',100)"),doc=>{
  assert.deepEqual(frame(doc),[255,0,0,255]);
  doc.execute('html.advance','{"millis":100}');
  assert.deepEqual(frame(doc),purple);
});
check(source('')+'<script type="module" src="/assets/main.js"></script>',doc=>{
  doc.setResource('/assets/main.js',encode("import {color} from './color.js';document.body.style.background=color"));
  doc.setResource('/assets/color.js',encode("export const color='#5645d4'"));
  assert.deepEqual(frame(doc),purple);
  assert.deepEqual(diagnostics(doc),[]);
});
check(source("fetch('/assets/color.txt').then(r=>r.text()).then(c=>document.body.style.background=c)"),doc=>{
  doc.setResource('/assets/color.txt',encode('#5645d4'));
  assert.deepEqual(frame(doc),purple);
});
for(const url of ['file:///etc/passwd','http://127.0.0.1:8080/private','https://example.com/private']){
  check(source('')+`<script src="${url}"></script>`,doc=>{
    assert.ok(diagnostics(doc).some(item=>item.message.includes('resource-unavailable')));
  });
}
for(const code of ['while(true){}','function again(){again()}again()','function again(){queueMicrotask(again)}again()']){
  check(source(code),doc=>assert.ok(diagnostics(doc).some(item=>item.code==='html.javascript-error')));
}
check(source('function again(){setTimeout(again,0)}again()'),doc=>{
  assert.ok(diagnostics(doc).some(item=>item.code==='html.javascript-timer-limit'));
});
check(source('')+'<style>@media(scripting:enabled){body{background:#5645d4}}</style><script nomodule>throw Error("unexpected nomodule")</script><noscript>Fallback</noscript>',doc=>{
  assert.deepEqual(frame(doc),purple);
  assert.deepEqual(diagnostics(doc),[]);
});
console.log(`${checked}/${checked} WASM JavaScript checks passed`);

const verify=code=>check(source(code)+'<style>body>*:not(script){display:none}</style>',doc=>{assert.deepEqual(frame(doc),purple);assert.deepEqual(diagnostics(doc),[])});
const success="document.body.style.background='#5645d4'";
verify(`const parent=document.createElement('div'),child=document.createElement('div');parent.appendChild(child);document.body.appendChild(parent);const seen=[];parent.addEventListener('x',()=>seen.push('capture'),true);child.addEventListener('x',()=>seen.push('target'));parent.addEventListener('x',()=>seen.push('bubble'));child.dispatchEvent(new Event('x',{bubbles:true}));if(seen.join(',')==='capture,target,bubble'){${success}}`);
verify(`const target=document.createElement('div');const list=target.children;target.appendChild(document.createElement('span'));if(list.length===1&&list===target.children){${success}}`);
verify(`if(matchMedia('(min-width: 32px)').matches&&!matchMedia('(min-width: 128px)').matches){${success}}`);
verify(`const input=document.createElement('input');input.type='checkbox';input.checked=true;let during=true;input.onclick=e=>{during=input.checked;e.preventDefault()};input.click();if(!during&&input.checked&&!input.hasAttribute('checked')){${success}}`);
verify(`const fieldset=document.createElement('fieldset');fieldset.innerHTML='<input type=checkbox>';fieldset.disabled=true;document.body.appendChild(fieldset);const input=fieldset.firstElementChild;input.click();if(!input.checked&&input.matches(':disabled')){${success}}`);
verify(`const form=document.createElement('form');form.innerHTML='<input value=original>';document.body.appendChild(form);const input=form.firstElementChild;input.value='edited';form.reset();input.defaultValue='next';if(input.value==='next'){${success}}`);
verify(`const label=document.createElement('label');label.innerHTML='<input type=checkbox>';document.body.appendChild(label);label.click();if(label.firstElementChild.checked){${success}}`);
verify(`const canvas=document.createElement('canvas'),ctx=canvas.getContext('2d');ctx.fillStyle='#5645d4';ctx.fillRect(0,0,8,8);if(ctx.getImageData(0,0,1,1).data.join(',')==='86,69,212,255'){${success}}`);
verify(`const canvas=document.createElement('canvas'),ctx=canvas.getContext('2d');ctx.fillStyle='red';ctx.fillRect(0,0,8,8);canvas.width=canvas.width;if(ctx.fillStyle==='#000000'&&ctx.getImageData(0,0,1,1).data[3]===0){${success}}`);
check(source('')+'<canvas width="64" height="64" style="position:absolute;left:0;top:0" id="canvas"></canvas><script>const ctx=document.getElementById("canvas").getContext("2d");ctx.fillStyle="#5645d4";ctx.fillRect(0,0,64,64)</script>',doc=>{
  assert.deepEqual(frame(doc),purple);assert.deepEqual(diagnostics(doc),[]);
  assert.equal(doc.dirty,false);
  assert.match(new TextDecoder().decode(doc.save()),/ctx.fillRect/);
});
console.log(`${checked}/${checked} extended WASM JavaScript checks passed`);

verify(`const progress=document.createElement('progress');progress.max=100;progress.value=65;if(progress.position===.65&&progress.value===65){progress.removeAttribute('value');if(progress.position===-1&&progress.matches(':indeterminate')){${success}}}`);
verify(`const meter=document.createElement('meter');meter.min=20;meter.max=100;meter.value=-1;meter.low=200;if(meter.value===20&&meter.low===100&&meter.getAttribute('value')==='-1'){${success}}`);
verify(`const source=document.createElement('canvas');source.width=8;source.height=8;const paint=source.getContext('2d');paint.fillStyle='#5645d4';paint.fillRect(0,0,8,8);const canvas=document.createElement('canvas'),ctx=canvas.getContext('2d');ctx.drawImage(source,0,0,32,32);paint.clearRect(0,0,8,8);if(ctx.getImageData(16,16,1,1).data.join(',')==='86,69,212,255'){${success}}`);
verify(`const canvas=document.createElement('canvas'),ctx=canvas.getContext('2d'),path=new Path2D('M0 0h24v24h-24z');ctx.fillStyle='#5645d4';ctx.fill(path);if(ctx.isPointInPath(path,24,24)&&!ctx.isPointInPath(path,30,30)&&ctx.getImageData(12,12,1,1).data[0]===86){${success}}`);
verify(`const canvas=document.createElement('canvas'),ctx=canvas.getContext('2d');ctx.rect(0,0,32,32);ctx.roundRect(24,8,-16,16,4);ctx.fill();if(ctx.getImageData(16,16,1,1).data[3]===0&&ctx.getImageData(2,2,1,1).data[3]===255){${success}}`);
check(source('')+'<script type="module" src="/assets/main.js"></script>',doc=>{
  doc.setResource('/assets/main.js',encode("import {color} from './color.js';document.body.style.background=color"));
  doc.setResource('/assets/color.js',encode("const color='#5645d4';export {color} // final export"));
  assert.deepEqual(frame(doc),purple);assert.deepEqual(diagnostics(doc),[]);
});
console.log(`${checked}/${checked} compatibility WASM JavaScript checks passed`);

verify(`const canvas=document.createElement('canvas'),ctx=canvas.getContext('2d');ctx.font='32px Inter';const metric=ctx.measureText('Canvas text');ctx.save();ctx.font='48px Inter';ctx.restore();if(metric instanceof TextMetrics&&metric.width>100&&metric.actualBoundingBoxAscent>0&&ctx.measureText('').width===0&&ctx.font==='32px Inter'){${success}}`);
verify(`const canvas=document.createElement('canvas'),ctx=canvas.getContext('2d');ctx.font='32px Inter';ctx.fillText('Canvas text',20,60);const data=ctx.getImageData(20,20,180,60).data;if(data.some((value,index)=>index%4===3&&value>0)){${success}}`);
verify(`const canvas=document.createElement('canvas'),ctx=canvas.getContext('2d'),path=new Path2D();path.moveTo(0,0);path.arcTo(32,0,32,32,8);path.lineTo(32,32);path.lineTo(0,32);path.closePath();if(ctx.isPointInPath(path,16,16)&&!ctx.isPointInPath(path,31,1)){${success}}`);
console.log(`${checked}/${checked} Canvas text and tangent arc WASM checks passed`);

verify(`const target=new EventTarget();let count=0;target.addEventListener('paint',event=>{count++;event.preventDefault()},{passive:true});if(target.dispatchEvent(new Event('paint',{cancelable:true}))&&count===1&&document instanceof EventTarget){${success}}`);
verify(`const controller=new AbortController(),target=new EventTarget();let count=0;const callback=()=>count++;target.addEventListener('paint',callback,{signal:controller.signal});controller.abort('done');target.dispatchEvent(new Event('paint'));target.addEventListener('paint',callback);target.dispatchEvent(new Event('paint'));if(count===1&&controller.signal.reason==='done'){${success}}`);
verify(`const controller=new AbortController();controller.signal.addEventListener('abort',event=>event.stopImmediatePropagation());const signal=AbortSignal.any([controller.signal]);controller.abort('done');if(signal.aborted&&signal.reason==='done'){${success}}`);
verify(`fetch('/not-supplied',{signal:AbortSignal.abort('done')}).then(()=>{throw Error('aborted fetch resolved')},reason=>{if(reason==='done'){${success}}});`);
console.log(`${checked}/${checked} event cancellation WASM checks passed`);


verify(`const input=document.createElement('input');input.type='NUMBER';const number=input.type;input.type='unknown-kind';if(number==='number'&&input.type==='text'&&input.getAttribute('type')==='unknown-kind'){${success}}`);
check('<!doctype html><style>body{margin:0}select{font:20px Inter;appearance:none;border:0;padding:0;color:black;width:64px;height:32px}</style><select id="choice"><option>Ready</option><option>Next</option></select><script>setTimeout(()=>document.getElementById("choice").selectedIndex=-1,100)</script>',doc=>{
  const hasInk=pixels=>pixels.some((channel,index)=>index%4!==3&&channel<128);
  assert.ok(hasInk(doc.render(0,0,1,64,32)));
  doc.execute('html.advance','{"millis":100}');
  assert.equal(hasInk(doc.render(0,0,1,64,32)),false);
  assert.deepEqual(diagnostics(doc),[]);
});
verify(`const input=document.createElement('input');input.type='CHECKBOX';input.value='saved';if(input.type==='checkbox'&&input.getAttribute('value')==='saved'){${success}}`);
console.log(`${checked}/${checked} form rendering WASM checks passed`);

check('<!doctype html><style>body{margin:0}div{width:10px;height:10px;background:#5645d4;transform:translate(20px,20px)}</style><div></div>',doc=>{
  doc.render(0,0,1,64,64);
  const scaled=doc.render(0,0,2,128,128);
  assert.deepEqual(Array.from(scaled.slice((45*128+45)*4,(45*128+45)*4+4)),purple);
  const restored=doc.render(0,0,1,64,64);
  assert.deepEqual(Array.from(restored.slice((25*64+25)*4,(25*64+25)*4+4)),purple);
});
console.log(`${checked}/${checked} persistent scale WASM checks passed`);

const selectStates=JSON.parse(await readFile(new URL('../conformance/select-state-cases.json',import.meta.url),'utf8'));
for(const sample of selectStates){
  const expected=JSON.stringify(sample.expected);
  const html=`<!doctype html><style>html,body{margin:0;height:100%;background:red}select{display:none}</style><body>${sample.body}<script>${sample.script};const expected=${expected};if(trace.length!==expected.length||trace.some((step,i)=>Object.keys(expected[i]).length!==Object.keys(step).length||Object.keys(step).some(key=>JSON.stringify(step[key])!==JSON.stringify(expected[i][key]))))throw Error('Unexpected selection trace');document.body.style.background='#5645d4';</script></body>`;
  check(html,doc=>{
    assert.deepEqual(diagnostics(doc),[],sample.id);
    assert.deepEqual(frame(doc),purple,sample.id);
    assert.equal(doc.dirty,false,sample.id);
    assert.equal(new TextDecoder().decode(doc.save()),html,sample.id);
  });
}
console.log(`${checked}/${checked} option state WASM checks passed`);

// Filled and stroked text use Canvas bitmap phases, independently of page DPR.
for (const operation of ['fillText','strokeText']) for (const scale of [1,1.5,2]) {
 verify(`const c=document.createElement('canvas');c.width=96;c.height=96;const x=c.getContext('2d');x.font='16px Inter';x.scale(${scale},${scale});function mask(dx,dy){x.save();x.resetTransform();x.clearRect(0,0,96,96);x.restore();x.${operation}('H',(16+dx)/${scale},(64+dy)/${scale});return x.getImageData(8,20,48,60).data}const a=mask(0,0),b=mask(.0625,.25),d=mask(.25,0),e=mask(.3125,.25);if(a.every((v,i)=>Math.abs(v-b[i])<=1)&&d.every((v,i)=>Math.abs(v-e[i])<=1)&&a.some((v,i)=>Math.abs(v-d[i])>1)){${success}}`);
}
console.log(`${checked}/${checked} Canvas origin WASM checks passed`);

const kerningStates=JSON.parse(await readFile(new URL('../conformance/canvas-kerning-cases.json',import.meta.url),'utf8'));
for(const sample of kerningStates){
 verify(`const canvas=document.createElement('canvas');document.body.appendChild(canvas);const ctx=canvas.getContext('2d');ctx.font='32px Inter';if((()=>{${sample.script}})()){${success}}`);
}
const kerningMetrics=JSON.parse(await readFile(new URL('../conformance/canvas-kerning-metrics.json',import.meta.url),'utf8'));
const kerningFonts=await Promise.all(['Lora','JetBrainsMono'].map(async family=>[family,await readFile(new URL(`../conformance/assets/${family}-Variable.ttf`,import.meta.url))]));
check(source(`const canvas=document.createElement('canvas'),ctx=canvas.getContext('2d');for(const sample of ${JSON.stringify(kerningMetrics)}){const family=sample.family==='JetBrainsMono'?'JetBrains Mono':sample.family;ctx.font=sample.size+'px "'+family+'"';for(const mode of ['auto','normal','none']){ctx.fontKerning=mode;const width=ctx.measureText(sample.text).width;if(Math.abs(width-sample.widths[mode])>1/64)throw Error('kerning metric: '+family+' '+sample.size+' '+sample.text+' '+mode)}}${success}`),doc=>{
 for(const [family,bytes] of kerningFonts)doc.setResource(`/assets/${family}.ttf`,bytes);
 assert.deepEqual(frame(doc),purple);assert.deepEqual(diagnostics(doc),[]);
 assert.equal(doc.dirty,false);
});
console.log(`${checked}/${checked} Canvas kerning WASM checks passed (15 state cases and 216 metric values within 1/64 CSS px)`);

const spacingStates=JSON.parse(await readFile(new URL('../conformance/canvas-spacing-cases.json',import.meta.url),'utf8'));
for(const sample of spacingStates){
 verify(`const canvas=document.createElement('canvas');document.body.appendChild(canvas);const ctx=canvas.getContext('2d');ctx.font='32px Inter';if((()=>{${sample.script}})()){${success}}`);
}
const spacingGrammar=JSON.parse(await readFile(new URL('../conformance/canvas-spacing-grammar.json',import.meta.url),'utf8'));
verify(`const ctx=document.createElement('canvas').getContext('2d');for(const prop of ['letterSpacing','wordSpacing'])for(const [input,expected]of Object.entries(${JSON.stringify(spacingGrammar)})){ctx[prop]='7px';ctx[prop]=input;if(ctx[prop]!==expected)throw Error('spacing grammar '+prop+' '+input)}${success}`);
const spacingMetrics=JSON.parse(await readFile(new URL('../conformance/canvas-spacing-metrics.json',import.meta.url),'utf8'));
check(source(`const ctx=document.createElement('canvas').getContext('2d');for(const sample of ${JSON.stringify(spacingMetrics)}){const family=sample.family==='JetBrainsMono'?'JetBrains Mono':sample.family;ctx.font='32px "'+family+'"';ctx.letterSpacing=sample.letter+'px';ctx.wordSpacing=sample.word+'px';const width=ctx.measureText(sample.text??String.fromCodePoint(...sample.codePoints)).width;if(Math.abs(width-sample.width)>.001)throw Error('spacing metric '+family+' '+width+' != '+sample.width)}${success}`),doc=>{
 for(const [family,bytes] of kerningFonts)doc.setResource(`/assets/${family}.ttf`,bytes);
 assert.deepEqual(frame(doc),purple);assert.deepEqual(diagnostics(doc),[]);
 assert.equal(doc.dirty,false);
});
console.log(`${checked}/${checked} Canvas spacing WASM checks passed (24 states, grammar and 288 widths within 0.001 CSS px)`);

const textCoercionStates=JSON.parse(await readFile(new URL('../conformance/canvas-text-coercion-cases.json',import.meta.url),'utf8'));
for(const sample of textCoercionStates){
 verify(`const canvas=document.createElement('canvas');document.body.appendChild(canvas);const ctx=canvas.getContext('2d');ctx.font='32px Inter';if((()=>{${sample.script}})()){${success}}`);
}
console.log(`${checked}/${checked} Canvas text coercion WASM checks passed (16 browser-verified cases)`);

const textArgumentStates=JSON.parse(await readFile(new URL('../conformance/canvas-text-arguments-cases.json',import.meta.url),'utf8'));
for(const sample of textArgumentStates){
 verify(`const canvas=document.createElement('canvas');document.body.appendChild(canvas);const ctx=canvas.getContext('2d');ctx.font='32px Inter';if((()=>{${sample.script}})()){${success}}`);
}
console.log(`${checked}/${checked} Canvas text argument WASM checks passed (29 browser-verified cases)`);

const numericStates=JSON.parse(await readFile(new URL('../conformance/canvas-numeric-cases.json',import.meta.url),'utf8'));
for(const sample of numericStates){
 verify(`const canvas=document.createElement('canvas');document.body.appendChild(canvas);const ctx=canvas.getContext('2d');ctx.font='32px Inter';if((()=>{${sample.script}})()){${success}}`);
}
console.log(`${checked}/${checked} Canvas numeric argument WASM checks passed (47 browser-verified cases)`);

const numericStateCases=JSON.parse(await readFile(new URL('../conformance/canvas-state-cases.json',import.meta.url),'utf8'));
for(const sample of numericStateCases){
 verify(`const canvas=document.createElement('canvas');document.body.appendChild(canvas);const ctx=canvas.getContext('2d');ctx.font='32px Inter';if((()=>{${sample.script}})()){${success}}`);
}
console.log(`${checked}/${checked} Canvas numeric state WASM checks passed (46 browser-verified cases)`);
