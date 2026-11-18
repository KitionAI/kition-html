import {readFileSync} from 'node:fs';
const states=JSON.parse(readFileSync(new URL('./canvas-spacing-cases.json',import.meta.url),'utf8'));
const samples=states.map(({id,script})=>[`canvas-spacing-${id}`,'javascript','canvas{display:none}.result{width:240px;height:96px;background:#e5486a}',`<canvas id="canvas"></canvas><div class="result"></div><script>document.fonts.load('32px Inter').then(()=>{const canvas=document.getElementById('canvas'),ctx=canvas.getContext('2d');ctx.font='32px Inter';if((()=>{${script}})())document.querySelector('.result').style.background='#5645d4';document.body.dataset.ready='true'})</script>`,{requires:'javascript',ready:'body[data-ready="true"]'}]);
for(const [id,family] of [['inter','Inter'],['lora','Lora'],['mono','JetBrains Mono']])for(const kind of ['letter','word','negative','stroke','max-width','scaled']){
 const op=kind==='stroke'?'strokeText':'fillText';
 const letter=kind==='word'?'0px':kind==='negative'?'-1px':'2px';
 const word=kind==='letter'?'0px':kind==='negative'?'-2px':'4px';
 samples.push([`canvas-spacing-${id}-${kind}`,'javascript','canvas{display:block}',`<canvas id="canvas" width="592" height="360"></canvas><script>document.fonts.load('32px "${family}"').then(()=>{const ctx=document.getElementById('canvas').getContext('2d');ctx.font='32px "${family}"';ctx.fillStyle='#5645d4';ctx.strokeStyle='#5645d4';${kind==='scaled'?'ctx.translate(8.25,8.25);ctx.scale(1.25,1.25);':''}ctx.letterSpacing='${letter}';ctx.wordSpacing='${word}';${kind==='max-width'?"ctx.textAlign='center';":''}ctx.${op}('AVATAR To office',${kind==='max-width'?280:24},80${kind==='max-width'?',180':''});ctx.${op}(' A  B C ',${kind==='max-width'?280:24},160${kind==='max-width'?',120':''});document.body.dataset.ready='true'})</script>`,{requires:'javascript',ready:'body[data-ready="true"]'}]);
}
export default samples;
