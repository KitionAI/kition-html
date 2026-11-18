// Independent placement and advance samples; no expected raster is synthesized.
const fonts = [['inter','Inter'], ['lora','Lora'], ['mono','JetBrains Mono']];
const samples = [];
for (const [id, family] of fonts) for (const size of [12,16,24]) {
  const css = `#stage{padding:0}span{position:absolute;font:${size}px/48px "${family}";white-space:pre}`;
  const phases = Array.from({length:48}, (_,i) => {
    const column = i % 16, row = Math.floor(i / 16);
    return `<span style="left:${16+column*38+column/16}px;top:${24+row*96}px;color:${['#000000','#1a1a1a','#5645d4'][row]}">Hg</span>`;
  }).join('');
  samples.push([`glyph-phase-${id}-${size}`, 'typography', css, phases]);
  const lines = ['HHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHH', 'Hamburg AVATAR 0123456789 ffi fl', '{"value":"alpha","selected":[true,false]}'];
  samples.push([`glyph-advance-${id}-${size}`, 'typography', css,
    lines.map((line,i)=>`<span style="left:${24+i*.1875}px;top:${24+i*96}px">${line}</span>`).join('')]);
}
for (const [name, transform] of [
  ['translate','translate(.1875px,.3125px)'],
  ['scale','scale(1.25)'],
  ['reflect','scaleX(-1)'],
  ['quarter-turn','rotate(90deg)'],
  ['rotate','rotate(17deg)'],
  ['skew','skewX(12deg)'],
]) {
  samples.push([`glyph-transform-${name}`, 'typography',
    `#stage{padding:0}span{position:absolute;left:320px;top:96px;font:16px/32px Inter;transform-origin:0 0;transform:${transform};white-space:pre}`,
    '<span>Hamburg AVATAR 012345</span>']);
}
export default samples;
