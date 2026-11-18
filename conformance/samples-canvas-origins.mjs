// Canvas bitmap positions are independent of the page's device scale.
const fonts=[['inter','Inter'],['lora','Lora'],['mono','JetBrains Mono']];
const sample=(id,family,size,code)=>[
 `canvas-origin-${id}`, 'javascript', 'canvas{display:block}',
 `<canvas id="canvas" width="592" height="320"></canvas><script>document.fonts.load('${size}px "${family}"').then(()=>{const ctx=document.getElementById('canvas').getContext('2d');ctx.font='${size}px "${family}"';ctx.fillStyle='#1a1a1a';${code}document.body.dataset.ready='true'})</script>`,
 {requires:'javascript',ready:'body[data-ready="true"]'},
];
const samples=[];
for(const [id,family] of fonts) for(const size of [12,16,24]) for(const axis of ['x','y']){
 samples.push(sample(`${id}-${size}-${axis}`,family,size,
  `for(let row=0;row<3;row++){ctx.fillStyle=['#000000','#1a1a1a','#5645d4'][row];for(let i=0;i<16;i++)ctx.fillText('Hg',16+i*35+${axis==='x'?'i/16':'0'},60+row*88+${axis==='y'?'i/16':'0'});}`));
}
for(const [id,code] of [
 ['translate',"ctx.translate(.1875,.3125);ctx.fillText('Hamburg AVATAR 012345',24,72);"],
 ['scale',"ctx.scale(1.25,1.5);ctx.fillText('Hamburg AVATAR 012345',24.125,72.25);"],
 ['skew',"ctx.transform(1,0,.25,1,0,0);ctx.fillText('Hamburg AVATAR 012345',24.125,72.25);"],
 ['rotate',"ctx.translate(80,96);ctx.rotate(Math.PI/12);ctx.fillText('Hamburg AVATAR 012345',.1875,.3125);"],
 ['reflect',"ctx.translate(520,0);ctx.scale(-1,1);ctx.fillText('Hamburg AVATAR 012345',24.1875,72.3125);"],
 ['max-width',"ctx.fillText('Hamburg AVATAR 012345',24.1875,72.3125,140);"],
 ['stroke-x',"ctx.lineWidth=1;for(let i=0;i<16;i++)ctx.strokeText('H',16+i*35+i/16,72);"],
 ['stroke-y',"ctx.lineWidth=1;for(let i=0;i<16;i++)ctx.strokeText('H',16+i*35,72+i/16);"],
]) samples.push(sample(id,'Inter',16,code));
export default samples;
