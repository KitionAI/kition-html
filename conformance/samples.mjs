import canvasStateSamples from './samples-canvas-state.mjs';
import canvasNumericSamples from './samples-canvas-numeric.mjs';
import canvasTextArgumentsSamples from './samples-canvas-text-arguments.mjs';
import canvasTextCoercionSamples from './samples-canvas-text-coercion.mjs';
import canvasSpacingSamples from './samples-canvas-spacing.mjs';
import canvasKerningSamples from './samples-canvas-kerning.mjs';
import rasterSizeSamples from './samples-raster-size.mjs';
import canvasOriginSamples from './samples-canvas-origins.mjs';
import glyphPhaseSamples from './samples-glyph-phases.mjs';
import rangeSamples from './samples-ranges.mjs';
import canvasImageSamples from './samples-canvas-images.mjs';
import canvasPathSamples from './samples-canvas-paths.mjs';
import canvasArcSamples from './samples-canvas-arcs.mjs';
import compositingSamples from './samples-compositing.mjs';
import canvasTextSamples from './samples-canvas-text.mjs';
import fieldsetSamples from './samples-fieldsets.mjs';
import canvasSamples from './samples-canvas.mjs';
import formSamples from './samples-forms.mjs';
import tablePaintSamples from './samples-tables-paint.mjs';
import eventSamples from './samples-events.mjs';
import eventOptionSamples from './samples-event-options.mjs';
import eventTargetSamples from './samples-event-targets.mjs';
import eventFieldSamples from './samples-event-fields.mjs';
import pageSamples from './samples-pages.mjs';
import selectSamples from './samples-selects.mjs';
import selectStateSamples from './samples-select-state.mjs';
import tableEdgeSamples from './samples-table-edges.mjs';
import tableConflictSamples from './samples-table-conflicts.mjs';
import formNativeSamples from './samples-form-native.mjs';
import inputLayoutSamples from './samples-input-layout.mjs';
import inputTypeSamples from './samples-input-types.mjs';
// Standalone HTML/CSS/JavaScript samples. Each tuple tests a distinct behavior.
const target='<div id="target" class="box"></div>';
const tiles='<div class="tiles"><i></i><i></i><i></i><i></i></div>';
const js=(id,css,body,script,extra={})=>[id,'javascript',css,`${body}<script>${script}</script>`,{requires:'javascript',...extra}];
const color="document.querySelector('#target').style.background='#5645d4'";
export default [
  ...canvasStateSamples,
  ...canvasNumericSamples,
  ...canvasTextArgumentsSamples,
  ...canvasTextCoercionSamples,
  ...canvasSpacingSamples,
  ...canvasKerningSamples,
  ...rasterSizeSamples,
  ...canvasOriginSamples,
  ...glyphPhaseSamples,
  ...pageSamples,
  ...selectSamples,
  ...selectStateSamples,
  ...tableEdgeSamples,
  ...tableConflictSamples,
  ...formNativeSamples,
  ...inputLayoutSamples,
  ...inputTypeSamples,
  ...canvasImageSamples,
  ...canvasPathSamples,
  ...canvasArcSamples,
  ...compositingSamples,
  ...canvasTextSamples,
  ...rangeSamples,
  ...fieldsetSamples,
  ...canvasSamples,
  ...eventSamples,
  ...eventOptionSamples,
  ...eventTargetSamples,
  ...eventFieldSamples,
  ...formSamples,
  ...tablePaintSamples,
  js('dom-create-append','', '<section id="target"></section>', "const node=document.createElement('div');node.className='box';node.style.background='#5645d4';document.querySelector('#target').appendChild(node)"),
  js('dom-fragment-replace','', '<section id="target"><div class="box"></div></section>', "const fragment=document.createDocumentFragment();for(const shade of ['#5645d4','#e5486a','#1f9d4c']){const item=document.createElement('div');item.className='tile';item.style.background=shade;fragment.appendChild(item)}document.querySelector('#target').replaceChildren(fragment)"),
  js('dom-template-clone','', '<template id="pattern"><div class="tile" style="background:#5645d4"></div></template><section id="target"></section>', "document.querySelector('#target').appendChild(document.querySelector('#pattern').content.cloneNode(true))"),
  js('dom-prepend-remove','', '<section id="target"><div id="old" class="tile"></div><div class="tile" style="background:#e5486a"></div></section>', "const first=document.createElement('div');first.className='tile';first.style.background='#5645d4';document.querySelector('#target').prepend(first);document.querySelector('#old').remove()"),
  js('dom-insert-adjacent-html','',target,"document.querySelector('#target').insertAdjacentHTML('beforebegin','<div class=\"tile\" style=\"background:#5645d4\"></div>');document.querySelector('#target').insertAdjacentHTML('afterend','<div class=\"tile\" style=\"background:#1f9d4c\"></div>')"),
  js('dom-text-content','', '<section id="target"></section>', "document.querySelector('#target').textContent='Generated HTML content'"),
  js('dom-inner-text-lines','', '<section id="target"></section>', "document.querySelector('#target').innerText='First line\\nSecond line\\nThird line'"),
  js('dom-class-list-toggle','.active{background:#5645d4}.wide{width:280px}',target,"const node=document.querySelector('#target');node.classList.add('active','wide');node.classList.toggle('active');node.classList.toggle('active')"),
  js('dom-dataset-selector','[data-state="ready"]{background:#5645d4;width:280px}',target,"document.querySelector('#target').dataset.state='ready'"),
  js('dom-attribute-removal','[hidden]{display:none}.box{background:#5645d4}', '<div id="target" class="box" hidden></div>',"document.querySelector('#target').removeAttribute('hidden')"),
  js('dom-query-closest','section.ready .box{background:#5645d4}', '<section><div class="box"><span id="leaf"></span></div></section>',"document.querySelector('#leaf').closest('section').classList.add('ready')"),
  js('dom-selectors-live-list','.tile{background:#5645d4}', '<section id="target"></section>',"const list=document.getElementsByClassName('tile');for(let i=0;i<3;i++){const item=document.createElement('div');item.className='tile';document.querySelector('#target').appendChild(item)}document.querySelector('#target').style.borderLeft=list.length===3?'12px solid #1f9d4c':'12px solid red'"),
  js('cssom-inline-properties','',target,"const node=document.querySelector('#target');node.style.setProperty('background','#5645d4');node.style.setProperty('width','280px');node.style.setProperty('border-radius','12px');node.style.removeProperty('height')"),
  js('cssom-custom-property','.box{background:var(--tone,#e5486a);width:var(--size,160px)}',target,"document.documentElement.style.setProperty('--tone','#5645d4');document.querySelector('#target').style.setProperty('--size','280px')"),
  js('cssom-css-text','',target,"document.querySelector('#target').style.cssText='width:280px;height:96px;background:#5645d4;border:8px solid #e5486a;border-radius:12px'"),
  js('cssom-insert-rule','',target,"document.styleSheets[0].insertRule('#target{background:#5645d4;width:280px}',document.styleSheets[0].cssRules.length)"),
  js('cssom-rule-style','',target,"const sheet=document.styleSheets[0];const index=sheet.insertRule('#target{background:red}',sheet.cssRules.length);sheet.cssRules[index].style.setProperty('background','#5645d4')"),
  js('cssom-appended-style','',target,"const style=document.createElement('style');style.textContent='#target{background:#5645d4;width:280px}';document.head.appendChild(style)"),
  js('cssom-replaced-style','',target,"document.querySelector('#dynamic').textContent='#target{background:#5645d4;width:280px}'",{head:'<style id="dynamic">#target{background:red}</style>'}),
  js('cssom-computed-size','.box{box-sizing:border-box;width:200px;padding:12px;border:4px solid #1f9d4c}',target,"const style=getComputedStyle(document.querySelector('#target'));if(style.width==='200px'&&style.paddingLeft==='12px'){"+color+"}"),
  js('cssom-match-media','',target,"if(matchMedia('(min-width:600px)').matches&&!matchMedia('(max-width:400px)').matches){"+color+"}"),
  js('cssom-supports','',target,"if(CSS.supports('display','grid')&&CSS.supports('color','rgb(10 20 30)')){"+color+"}"),
  js('event-click-bubble','', '<section id="parent">'+target+'</section>',"document.querySelector('#parent').addEventListener('click',()=>{"+color+"});document.querySelector('#target').click()"),
  js('event-listener-once','',target,"let count=0;const node=document.querySelector('#target');node.addEventListener('click',()=>{count++;node.style.background=count===1?'#5645d4':'red'},{once:true});node.click();node.click()"),
  js('event-custom-detail','',target,"const node=document.querySelector('#target');node.addEventListener('paint',event=>node.style.background=event.detail.color);node.dispatchEvent(new CustomEvent('paint',{detail:{color:'#5645d4'}}))"),
  js('event-stop-propagation','', '<section id="parent">'+target+'</section>',"const node=document.querySelector('#target');document.querySelector('#parent').addEventListener('click',()=>node.style.background='red');node.addEventListener('click',event=>{event.stopPropagation();node.style.background='#5645d4'});node.click()"),
  js('event-inline-handler','', '<div id="target" class="box" onclick="this.style.background=\'#5645d4\'"></div>',"document.querySelector('#target').click()"),
  js('event-prevent-default','',target,"const node=document.querySelector('#target');node.addEventListener('paint',event=>event.preventDefault());if(!node.dispatchEvent(new Event('paint',{cancelable:true}))){"+color+"}"),
  js('async-microtask-order','',target,"const order=[];Promise.resolve().then(()=>order.push('promise'));queueMicrotask(()=>{order.push('microtask');if(order.join(',')==='promise,microtask'){"+color+"}})"),
  js('async-await-dom','',target,"(async()=>{const value=await Promise.resolve('#5645d4');document.querySelector('#target').style.background=value})()"),
  js('async-timeout-cancel','',target,"const canceled=setTimeout(()=>document.querySelector('#target').style.background='red',0);clearTimeout(canceled);setTimeout(()=>{"+color+"},0)"),
  js('async-animation-frame','',target,"requestAnimationFrame(()=>{"+color+"})"),
  js('async-fetch-json','', '<section id="target"></section>',"fetch('../assets/sample-data.json').then(response=>response.json()).then(data=>{document.querySelector('#target').innerHTML=data.map(item=>'<div class=\"tile\" style=\"background:'+item.color+'\"></div>').join('');document.body.dataset.ready='true'})",{ready:'body[data-ready="true"]'}),
  js('module-import-map','',target,'',{head:'<script type="importmap">{"imports":{"palette":"../assets/sample-palette.js"}}</script><script type="module">import {purple} from "palette";document.querySelector("#target").style.background=purple;document.body.dataset.ready="true"</script>',ready:'body[data-ready="true"]'}),
  js('module-export-asi','',target,'',{head:'<script type="module">import {purple} from "../assets/sample-export.js";document.querySelector("#target").style.background=purple</script>'}),
  js('module-export-asi-comment','',target,'',{head:'<script type="module">import {purple} from "../assets/sample-export-comment.js";document.querySelector("#target").style.background=purple</script>'}),
  ['css-aspect-ratio-grid','layout','section{display:grid;grid-template-columns:repeat(3,1fr);gap:16px}.tile{width:auto;height:auto;aspect-ratio:4/3;background:#5645d4}', '<section><div class="tile"></div><div class="tile"></div><div class="tile"></div></section>'],
  ['css-flex-wrap-gap','layout','.tiles{display:flex;flex-wrap:wrap;gap:16px;width:400px}.tiles i{display:block;flex:0 0 120px;height:64px;background:#5645d4}',tiles],
  ['css-grid-auto-fit','layout','.tiles{display:grid;grid-template-columns:repeat(auto-fit,minmax(120px,1fr));gap:16px}.tiles i{display:block;height:80px;background:#5645d4}',tiles],
  ['css-minmax-intrinsic','layout','.tiles{display:grid;grid-template-columns:minmax(0,1fr) 160px;gap:16px}.tiles i{display:block;min-width:0;height:80px;background:#5645d4}',tiles],
  ['css-table-percent-columns','table','table{width:480px;border-spacing:0}td{height:64px;padding:0;background:#5645d4}td+td{background:#e5486a}td:last-child{background:#1f9d4c}', '<table><colgroup><col style="width:20%"><col style="width:30%"><col></colgroup><tr><td></td><td></td><td></td></tr></table>'],
  ['css-table-percent-cells','table','table{width:480px;border-spacing:0}td{height:64px;padding:0;background:#5645d4}td+td{background:#e5486a}td:last-child{background:#1f9d4c}', '<table><tr><td style="width:20%"></td><td style="width:30%"></td><td></td></tr></table>'],
  ['css-table-rowspan-height','table','table{width:400px;border-spacing:8px}td{padding:0;height:40px;background:#5645d4}td[rowspan]{height:160px;background:#e5486a}', '<table><tr><td rowspan="2"></td><td></td></tr><tr><td></td></tr></table>'],
  ['css-background-layer-order','paint','.box{width:320px;height:180px;background:linear-gradient(90deg,transparent 50%,#e5486a 50%) 0 0/64px 100%,linear-gradient(#5645d4,#1f9d4c)}',target],
  ['css-background-origin-clip','paint','.box{box-sizing:content-box;width:240px;height:120px;padding:20px;border:12px dashed #e5486a;background:linear-gradient(#5645d4,#1f9d4c);background-origin:content-box;background-clip:padding-box;background-repeat:no-repeat}',target],
  ['css-position-sticky-initial','layout','section{height:200px;overflow:auto;border:2px solid #e5e3df}.sticky{position:sticky;top:0;height:48px;background:#5645d4}.spacer{height:400px;background:linear-gradient(white,#d9f3e1)}', '<section><div class="sticky"></div><div class="spacer"></div></section>'],
  ['css-logical-rtl-layout','layout','section{direction:rtl;display:flex;gap:16px}.tile{width:120px;height:80px;margin-inline-start:12px;border-inline-start:8px solid #e5486a;background:#5645d4}', '<section><div class="tile"></div><div class="tile"></div></section>'],
  ['html-details-disclosure','html','details{border:1px solid #e5e3df;padding:16px;border-radius:12px}summary{font-weight:600}p{margin:12px 0 0}', '<details open><summary>Release details</summary><p>HTML, CSS and JavaScript render in one document.</p></details>'],
  ['html-progress-meter','html','progress,meter{display:block;width:320px;height:24px;margin:16px 0;accent-color:#5645d4}', '<progress max="100" value="65"></progress><meter min="0" max="100" value="65"></meter>'],
  ['html-fieldset-legend','html','fieldset{border:2px solid #5645d4;border-radius:12px;padding:20px;width:320px}legend{padding:0 8px;font-weight:600}', '<fieldset><legend>Project settings</legend><div class="tile" style="background:#d9f3e1"></div></fieldset>'],
];
