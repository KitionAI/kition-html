const base='fieldset{margin:0;border:2px solid #5645d4;padding:20px;width:320px}legend{width:120px;height:24px;padding:0;background:#e5486a}.content{width:80px;height:50px;background:#1f9d4c}';
const body='<fieldset><legend></legend><div class="content"></div><div class="content"></div></fieldset>';
const sample=(id,css='',html=body)=>['fieldset-'+id,'html',base+css,html];
export default [
 sample('legend-block'),
 sample('specified-height','fieldset{height:100px}'),
 sample('border-box','fieldset{height:100px;box-sizing:border-box}'),
 sample('tall-legend','legend{height:60px}'),
 sample('legend-margins','legend{margin:8px 12px}'),
 sample('legend-centered','legend{margin-inline:auto}'),
 sample('thick-border','fieldset{border-width:40px}'),
 sample('zero-padding','fieldset{padding:0}'),
 sample('flex-content','fieldset{display:flex;gap:8px}'),
 sample('grid-content','fieldset{display:grid;grid-template-columns:1fr 1fr}'),
 sample('hidden-legend','legend{display:none}'),
 sample('positioned-legend','fieldset{position:relative}legend{position:absolute;left:100px;top:80px}'),
 sample('rtl-legend','fieldset{direction:rtl}'),
 sample('rounded-border','fieldset{border-radius:12px}'),
 sample('background','fieldset{background:#f0edff}'),
 sample('empty-content','', '<fieldset><legend></legend></fieldset>'),
 sample('no-legend','', '<fieldset><div class="content"></div></fieldset>'),
 sample('two-legends','', '<fieldset><legend></legend><legend style="background:#d9f3e1"></legend><div class="content"></div></fieldset>'),
 sample('hidden-first-legend','', '<fieldset><legend style="display:none"></legend><legend></legend><div class="content"></div></fieldset>'),
 sample('percentage-legend','legend{width:50%}'),
 sample('legend-border','legend{border:4px solid #ddaa30;padding:8px}'),
 sample('min-height','fieldset{min-height:200px}'),
 sample('max-height','fieldset{max-height:60px}'),
 sample('inline-fieldset','fieldset{display:inline-block;width:auto}'),
];
