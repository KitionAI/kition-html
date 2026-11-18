const rows='<tr><td></td><td></td><td></td></tr>'.repeat(3);
const sample=(id,css,body=rows)=>['table-edges-'+id,'table',`table{border-collapse:collapse;width:360px}td{height:40px;padding:0;background:#e9e6fb}${css}`,`<table>${body}</table>`];
export default [
 sample('bottom','td{border-bottom:2px solid #5645d4}'),
 sample('top','td{border-top:2px solid #5645d4}'),
 sample('left','td{border-left:2px solid #5645d4}'),
 sample('right','td{border-right:2px solid #5645d4}'),
 sample('horizontal','td{border-block:2px solid #5645d4}'),
 sample('vertical','td{border-inline:2px solid #5645d4}'),
 sample('colors','td{border:2px solid;border-color:#5645d4 #e5486a #1f9d4c #1a1a1a}'),
 sample('unequal','td{border:solid;border-width:2px 4px 6px 8px;border-color:#5645d4 #e5486a #1f9d4c #1a1a1a}'),
 sample('transparent-top','td{border:2px solid #5645d4;border-top-color:transparent}'),
 sample('bottom-caption','td{border-bottom:2px solid #5645d4}caption{height:24px;background:#1f9d4c}','<caption></caption>'+rows),
 sample('single-bottom','td{border-bottom:2px solid #5645d4}','<tr><td></td></tr>'),
 sample('bottom-one-pixel','td{border-bottom:1px solid #5645d4}'),
];
