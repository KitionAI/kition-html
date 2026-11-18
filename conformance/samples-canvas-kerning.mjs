import {readFileSync} from 'node:fs';
const states=JSON.parse(readFileSync(new URL('./canvas-kerning-cases.json',import.meta.url),'utf8'));
const samples=states.map(({id,script})=>[`canvas-kerning-${id}`,'javascript','canvas{display:none}.result{width:240px;height:96px;background:#e5486a}',`<canvas id="canvas"></canvas><div class="result"></div><script>document.fonts.load('32px Inter').then(()=>{const canvas=document.getElementById('canvas'),ctx=canvas.getContext('2d');ctx.font='32px Inter';if((()=>{${script}})())document.querySelector('.result').style.background='#5645d4';document.body.dataset.ready='true'})</script>`,{requires:'javascript',ready:'body[data-ready="true"]'}]);
for(const [id,family] of [['inter','Inter'],['lora','Lora'],['mono','JetBrains Mono']])for(const kind of ['fill','stroke','max-width','scaled']){
 const op=kind==='stroke'?'strokeText':'fillText';
 samples.push([`canvas-kerning-${id}-${kind}`,'javascript','canvas{display:block}',`<canvas id="canvas" width="592" height="360"></canvas><script>document.fonts.load('32px "${family}"').then(()=>{const ctx=document.getElementById('canvas').getContext('2d');ctx.font='32px "${family}"';ctx.fillStyle='#5645d4';ctx.strokeStyle='#5645d4';${kind==='scaled'?'ctx.translate(8.25,8.25);ctx.scale(1.25,1.25);':''}ctx.fontKerning='none';ctx.${op}('AVATAR To Wa',24,80${kind==='max-width'?',180':''});ctx.fontKerning='normal';ctx.${op}('AVATAR To Wa',24,160${kind==='max-width'?',180':''});document.body.dataset.ready='true'})</script>`,{requires:'javascript',ready:'body[data-ready="true"]'}]);
}
export default samples;
