import {readFileSync} from 'node:fs';
const states=JSON.parse(readFileSync(new URL('./canvas-state-cases.json',import.meta.url),'utf8'));
const samples=states.map(({id,script})=>[`canvas-state-${id}`,'javascript','canvas{display:none}.result{width:240px;height:96px;background:#e5486a}',`<canvas id="canvas"></canvas><div class="result"></div><script>document.fonts.load('32px Inter').then(()=>{const canvas=document.getElementById('canvas'),ctx=canvas.getContext('2d');ctx.font='32px Inter';if((()=>{${script}})())document.querySelector('.result').style.background='#5645d4';document.body.dataset.ready='true'})</script>`,{requires:'javascript',ready:'body[data-ready="true"]'}]);
const drawings={
 'alpha-bigint':"ctx.globalAlpha=0.5;try{ctx.globalAlpha=1n}catch(e){}ctx.fillRect(16,16,96,64)",
 'width-bigint':"ctx.lineWidth=8;try{ctx.lineWidth=1n}catch(e){}ctx.strokeRect(24,24,96,64)",
 'dash-sequence':"try{ctx.setLineDash({0:2,1:12,length:2})}catch(e){}ctx.lineWidth=8;ctx.beginPath();ctx.moveTo(16,32);ctx.lineTo(160,32);ctx.stroke()",
 'dash-order':"let n=2;ctx.setLineDash({[Symbol.iterator](){let i=0;return {next(){if(i++===2)return {done:true};return {done:false,value:{valueOf(){n+=2;return n}}}}}}});ctx.lineWidth=8;ctx.beginPath();ctx.moveTo(16,32);ctx.lineTo(160,32);ctx.stroke()",
};
for(const [id,script] of Object.entries(drawings))samples.push([`canvas-state-${id}-visible`,'javascript','canvas{display:block}',`<canvas id="canvas" width="592" height="240"></canvas><script>const canvas=document.getElementById('canvas'),ctx=canvas.getContext('2d');ctx.fillStyle='#5645d4';ctx.strokeStyle='#5645d4';${script}</script>`,{requires:'javascript'}]);
export default samples;
