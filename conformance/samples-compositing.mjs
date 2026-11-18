const colors=['#5645d4','#e5486a','#1f9d4c','#ffffff','#000000'];
const alphas=[.1,.25,.5,.75,.9];
const rgb=color=>[1,3,5].map(i=>parseInt(color.slice(i,i+2),16)).join(',');
const tiles=(mode)=>colors.flatMap((color,row)=>alphas.map(alpha=>{
  const background=colors[(row+1)%colors.length];
  const style=mode==='color'?`background:rgba(${rgb(color)},${alpha})`:`background:${color};opacity:${alpha}`;
  return `<div class="sample" style="background:${background}"><i style="${style}">${mode==='overlap'?'<b></b>':''}</i></div>`;
})).join('');
const css='.grid{display:grid;grid-template-columns:repeat(5,48px);gap:8px}.sample{width:48px;height:48px}.sample i{display:block;width:48px;height:48px}.sample b{display:block;position:relative;left:12px;top:12px;width:24px;height:24px;background:#fff}';
const canvas=(id,draw)=>['composite-'+id,'javascript','canvas{display:block}',`<canvas id="canvas" width="280" height="280"></canvas><script>const ctx=document.getElementById('canvas').getContext('2d'),colors=${JSON.stringify(colors)},alphas=${JSON.stringify(alphas)};for(let row=0;row<5;row++)for(let col=0;col<5;col++){const x=col*56,y=row*56,alpha=alphas[col],color=colors[row];ctx.globalAlpha=1;ctx.fillStyle=colors[(row+1)%5];ctx.fillRect(x,y,48,48);${draw}}</script>`,{requires:'javascript'}];
export default [
 ['composite-css-color','paint',css,`<div class="grid">${tiles('color')}</div>`],
 ['composite-css-layer','paint',css,`<div class="grid">${tiles('layer')}</div>`],
 ['composite-css-overlap','paint',css,`<div class="grid">${tiles('overlap')}</div>`],
 ['composite-css-rounded','paint',css+'.sample i{border-radius:12px}',`<div class="grid">${tiles('color')}</div>`],
 canvas('canvas-alpha','ctx.globalAlpha=alpha;ctx.fillStyle=color;ctx.fillRect(x,y,48,48);'),
 canvas('canvas-rounded','ctx.globalAlpha=alpha;ctx.fillStyle=color;ctx.beginPath();ctx.roundRect(x,y,48,48,12);ctx.fill();'),
 canvas('canvas-color',"const rgb=[1,3,5].map(i=>parseInt(color.slice(i,i+2),16));ctx.fillStyle='rgba('+rgb.join(',')+','+alpha+')';ctx.fillRect(x,y,48,48);"),
 canvas('canvas-image',"const source=document.createElement('canvas');source.width=1;source.height=1;const brush=source.getContext('2d');brush.fillStyle=color;brush.fillRect(0,0,1,1);ctx.globalAlpha=alpha;ctx.drawImage(source,x,y,48,48);"),
];
