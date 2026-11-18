// The host page only transports data. All parsing, layout, and rasterization run here.
import init, {HtmlDocument} from '/engine/kition_html_kit.js';
await init();
self.postMessage({ready:true});
let resourceBundle=[];
self.onmessage=({data:{html,assets,options}})=>{
  let doc;
  try{
    if(assets){resourceBundle=assets;self.postMessage({configured:true});return;}
    doc=HtmlDocument.open('fixture.html',new TextEncoder().encode(html));
    doc.setViewport(options.width,options.height,options.dark);
    for(const [name,bytes] of resourceBundle)doc.setResource(`/assets/${name}`,new Uint8Array(bytes));
    if (options.scale !== 1) {
      const width=Math.round(options.width*options.scale);
      const height=Math.round(options.height*options.scale);
      const rgba=doc.render(0,0,options.scale,width,height);
      self.postMessage({rgba:rgba.buffer,width,height},[rgba.buffer]);
    } else {
      const png=doc.renderPng();
      self.postMessage({png:png.buffer},[png.buffer]);
    }
  }catch(error){self.postMessage({error:String(error)});}
  finally{doc?.free();}
};
