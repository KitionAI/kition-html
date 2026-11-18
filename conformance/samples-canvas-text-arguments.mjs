import {readFileSync} from 'node:fs';
const states=JSON.parse(readFileSync(new URL('./canvas-text-arguments-cases.json',import.meta.url),'utf8'));
const samples=states.map(({id,script})=>[`canvas-text-arguments-${id}`,'javascript','canvas{display:none}.result{width:240px;height:96px;background:#e5486a}',`<canvas id="canvas"></canvas><div class="result"></div><script>document.fonts.load('32px Inter').then(()=>{const canvas=document.getElementById('canvas'),ctx=canvas.getContext('2d');ctx.font='32px Inter';if((()=>{${script}})())document.querySelector('.result').style.background='#5645d4';document.body.dataset.ready='true'})</script>`,{requires:'javascript',ready:'body[data-ready="true"]'}]);
for(const op of ['fillText','strokeText'])for(const argument of ['text','x','max']){
 const value=argument==='text'?"{toString(){canvas.width=canvas.width;return 'Canvas reset'}}":"'Canvas reset'";
 const x=argument==='x'?"{valueOf(){canvas.width=canvas.width;return 24}}":'24';
 const max=argument==='max'?"{valueOf(){canvas.width=canvas.width;return 180}}":'180';
 samples.push([`canvas-text-arguments-${op.toLowerCase()}-${argument}-visible`,'javascript','canvas{display:block}',`<canvas id="canvas" width="592" height="240"></canvas><script>document.fonts.load('32px Inter').then(()=>{const canvas=document.getElementById('canvas'),ctx=canvas.getContext('2d');ctx.font='32px Inter';ctx.fillStyle='#5645d4';ctx.strokeStyle='#5645d4';ctx.scale(2,2);ctx.${op}(${value},${x},80,${max});document.body.dataset.ready='true'})</script>`,{requires:'javascript',ready:'body[data-ready="true"]'}]);
}
export default samples;
