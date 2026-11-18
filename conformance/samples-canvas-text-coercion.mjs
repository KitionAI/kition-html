import {readFileSync} from 'node:fs';
const states=JSON.parse(readFileSync(new URL('./canvas-text-coercion-cases.json',import.meta.url),'utf8'));
const samples=states.map(({id,script})=>[`canvas-text-coercion-${id}`,'javascript','canvas{display:none}.result{width:240px;height:96px;background:#e5486a}',`<canvas id="canvas"></canvas><div class="result"></div><script>document.fonts.load('32px Inter').then(()=>{const canvas=document.getElementById('canvas'),ctx=canvas.getContext('2d');ctx.font='32px Inter';if((()=>{${script}})())document.querySelector('.result').style.background='#5645d4';document.body.dataset.ready='true'})</script>`,{requires:'javascript',ready:'body[data-ready="true"]'}]);
export default samples;
