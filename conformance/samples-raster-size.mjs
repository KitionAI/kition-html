// Font-size and transform combinations retain independent Chrome references.
const fonts=[['inter','Inter'],['lora','Lora'],['mono','JetBrains Mono']];
const samples=[];
for(const [id,family] of fonts)for(const size of [16,32])for(const scale of [.75,1.25,2]){
 const suffix=`${id}-${size}-${String(scale).replace('.','-')}`;
 samples.push([`raster-size-dom-${suffix}`,'typography',
  `#stage{padding:0}span{position:absolute;left:32px;top:48px;font:${size}px/48px "${family}";white-space:pre;transform-origin:0 0;transform:scale(${scale})}span+span{top:208px;color:#5645d4}`,
  '<span>Hamburg 0123</span><span>Hamburg 0123</span>']);
 samples.push([`raster-size-canvas-${suffix}`,'javascript','canvas{display:block}',
  `<canvas id="canvas" width="592" height="360"></canvas><script>document.fonts.load('${size}px "${family}"').then(()=>{const ctx=document.getElementById('canvas').getContext('2d');ctx.font='${size}px "${family}"';ctx.scale(${scale},${scale});ctx.fillStyle='#1a1a1a';ctx.fillText('Hamburg 0123',24/${scale},96/${scale});ctx.fillStyle='#5645d4';ctx.fillText('Hamburg 0123',24/${scale},256/${scale});document.body.dataset.ready='true'})</script>`,
  {requires:'javascript',ready:'body[data-ready="true"]'}]);
}
export default samples;
