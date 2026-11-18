import {readFileSync} from 'node:fs';
const states=JSON.parse(readFileSync(new URL('./canvas-numeric-cases.json',import.meta.url),'utf8'));
const samples=states.map(({id,script})=>[`canvas-numeric-${id}`,'javascript','canvas{display:none}.result{width:240px;height:96px;background:#e5486a}',`<canvas id="canvas"></canvas><div class="result"></div><script>document.fonts.load('32px Inter').then(()=>{const canvas=document.getElementById('canvas'),ctx=canvas.getContext('2d');ctx.font='32px Inter';if((()=>{${script}})())document.querySelector('.result').style.background='#5645d4';document.body.dataset.ready='true'})</script>`,{requires:'javascript',ready:'body[data-ready="true"]'}]);
const drawings={
 'fill-rect':"ctx.fillRect({valueOf(){canvas.width=canvas.width;return 24}},24,96,64)",
 'stroke-rect':"ctx.lineWidth=6;ctx.strokeRect({valueOf(){canvas.width=canvas.width;return 24.5}},24.5,96,64)",
 'clear-rect':"ctx.resetTransform();ctx.fillRect(24,24,96,64);try{ctx.clearRect(24n,24,96,64)}catch(e){if(!(e instanceof TypeError))throw e}",
 'translate':"ctx.translate({valueOf(){canvas.width=canvas.width;return 24}},16);ctx.fillRect(12,16,40,24)",
 'scale':"ctx.scale({valueOf(){canvas.width=canvas.width;return 2}},2);ctx.fillRect(12,16,40,24)",
 'rotate':"ctx.rotate({valueOf(){canvas.width=canvas.width;return Math.PI/2}});ctx.fillRect(24,-80,40,60)",
 'transform':"ctx.transform({valueOf(){canvas.width=canvas.width;return 2}},0,0,1,24,16);ctx.fillRect(12,16,40,24)",
};
for(const [id,script]of Object.entries(drawings))samples.push([`canvas-numeric-${id}-visible`,'javascript','canvas{display:block}',`<canvas id="canvas" width="592" height="240"></canvas><script>const canvas=document.getElementById('canvas'),ctx=canvas.getContext('2d');ctx.fillStyle='#5645d4';ctx.strokeStyle='#5645d4';ctx.translate(40,30);ctx.scale(2,2);${script}</script>`,{requires:'javascript'}]);
export default samples;
