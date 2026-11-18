import { mkdir, writeFile } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import path from 'node:path';
import layout from '../conformance/layout.mjs';
import paint from '../conformance/paint.mjs';
import text from '../conformance/text.mjs';
import platform from '../conformance/platform.mjs';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const cases = [...layout, ...paint, ...text, ...platform];
if (cases.length < 100 || new Set(cases.map(([id]) => id)).size !== cases.length) {
  throw new Error('At least 100 distinct feature cases are required');
}
const base = `
@font-face{font-family:Inter;src:url(../assets/Inter-Variable.ttf);font-weight:100 900;font-style:normal}
@font-face{font-family:Lora;src:url(../assets/Lora-Variable.ttf);font-weight:400 700;font-style:normal}
@font-face{font-family:"JetBrains Mono";src:url(../assets/JetBrainsMono-Variable.ttf);font-weight:100 800;font-style:normal}
html,body{margin:0;padding:0;background:white;color:#1a1a1a;font:16px/1.5 Inter}
#stage{box-sizing:border-box;width:640px;min-height:480px;padding:24px}
.a,.b,.c{width:160px;height:64px}.a{background:#5645d4}.b{background:#e5486a}.c{background:#1f9d4c}
`;
await mkdir(path.join(root, 'conformance/cases'), { recursive:true });
const manifest = [];
for (const [id, category, css, body, options = {}] of cases) {
  const source = `<!doctype html>\n<html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>${id}</title><style>${base}\n${css}</style>${options.head ?? ''}</head><body><main id="stage">${body}</main></body></html>\n`;
  await writeFile(path.join(root, 'conformance/cases', `${id}.html`), source);
  manifest.push({id,category,file:`cases/${id}.html`,width:640,height:480,scale:1,dark:options.dark ?? false,requires:options.requires ?? 'static-html-css'});
}
await writeFile(path.join(root, 'conformance/manifest.json'), `${JSON.stringify(manifest,null,2)}\n`);
console.log(`Generated ${manifest.length} distinct feature fixtures in ${new Set(manifest.map(c=>c.category)).size} categories`);
