import {mkdir,writeFile} from 'node:fs/promises';
import {PNG} from 'pngjs';
import samples from '../conformance/samples.mjs';
const root=new URL('../conformance/',import.meta.url);
const base=`
@font-face{font-family:Inter;src:url(../assets/Inter-Variable.ttf);font-weight:100 900}
@font-face{font-family:Lora;src:url(../assets/Lora-Variable.ttf);font-weight:400 700}
@font-face{font-family:"JetBrains Mono";src:url(../assets/JetBrainsMono-Variable.ttf);font-weight:100 800}
html,body{margin:0;padding:0;background:white;color:#1a1a1a;font:16px/1.5 Inter}
#stage{box-sizing:border-box;width:100%;min-height:480px;padding:24px}
.box{width:160px;height:96px;background:#e5486a}.tile{display:inline-block;width:120px;height:80px;margin:0 12px 12px 0;vertical-align:top}
`;
if(new Set(samples.map(([id])=>id)).size!==samples.length)throw Error('Duplicate sample ids');
await mkdir(new URL('samples/',root),{recursive:true});
const manifest=[];
for(const [id,category,css,body,options={}] of samples){
  if(!/^[a-z0-9-]+$/.test(id))throw Error('Sample ids must be portable');
  const html=`<!doctype html>\n<html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>${id}</title><style>${base}\n${css}</style>${options.head??''}</head><body><main id="stage">${body}</main></body></html>\n`;
  await writeFile(new URL(`samples/${id}.html`,root),html);
  manifest.push({id,category,file:`samples/${id}.html`,width:640,height:480,scale:1,dark:false,requires:options.requires??'static-html-css',...(options.ready?{ready:options.ready}:{})});
}
await writeFile(new URL('samples/manifest.json',root),JSON.stringify(manifest,null,2)+'\n');
await writeFile(new URL('assets/sample-data.json',root),JSON.stringify([{color:'#5645d4'},{color:'#e5486a'},{color:'#1f9d4c'}])+'\n');
await writeFile(new URL('assets/sample-palette.js',root),"export const purple = '#5645d4';\n");
await writeFile(new URL('assets/sample-export.js',root),"const purple = '#5645d4'; export {purple}");
await writeFile(new URL('assets/sample-export-comment.js',root),"const purple = '#5645d4'; export {purple} // final export\n");
const image = new PNG({width:80,height:40});
for(let y=0;y<40;y++)for(let x=0;x<80;x++)image.data.set(x<40?[86,69,212,255]:[229,72,106,255],(y*80+x)*4);
await writeFile(new URL('assets/sample-canvas.png',root),PNG.sync.write(image));
await writeFile(new URL('assets/sample-canvas.svg',root),'<svg xmlns="http://www.w3.org/2000/svg" width="80" height="40"><path fill="#5645d4" d="M0 0h40v40H0z"/><path fill="#e5486a" d="M40 0h40v40H40z"/></svg>\n');
console.log(`Generated ${manifest.length} supplementary samples`);
