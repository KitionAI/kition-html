const rows = '<tr><td></td><td></td><td></td></tr>'.repeat(3);
const sample = (id, css, body = rows, script = '') => [
  `table-conflict-${id}`, 'table',
  `table{border-collapse:collapse;width:360px}td{height:40px;padding:0;border:2px solid #5645d4;background:#e9e6fb}${css}`,
  `<table>${body}</table>${script ? `<script>${script}</script>` : ''}`,
  script ? {requires:'javascript'} : {},
];
export default [
  sample('footer-none', 'td{border:0;border-bottom:2px solid #5645d4}tfoot td{border:0}', `<tbody>${rows}</tbody><tfoot><tr><td></td><td></td><td></td></tr></tfoot>`),
  sample('row-colors', 'tr:nth-child(2) td{border-color:#e5486a}tr:last-child td{border-color:#1f9d4c}'),
  sample('column-colors', 'td:nth-child(2){border-color:#e5486a}td:last-child{border-color:#1f9d4c}'),
  sample('current-color', 'td{border-color:currentColor;color:#5645d4}td:nth-child(2){color:#e5486a}td:last-child{color:#1f9d4c}'),
  sample('hidden-horizontal', 'tr:nth-child(2) td{border-top-style:hidden}'),
  sample('hidden-vertical', 'td:nth-child(2){border-left-style:hidden}'),
  sample('none-neighbor', 'td:nth-child(2){border:0}'),
  sample('transparent-winner', 'td:first-child{border-right-color:transparent}'),
  sample('wider-neighbor', 'td:nth-child(2){border-left:6px solid #e5486a}'),
  sample('first-cell-none', 'td:first-child{border:0}'),
  sample('colspan', '', '<tr><td colspan="2"></td><td></td></tr><tr><td></td><td></td><td></td></tr>'),
  sample('rowspan', '', '<tr><td rowspan="2"></td><td></td><td></td></tr><tr><td></td><td></td></tr><tr><td></td><td></td><td></td></tr>'),
  sample('rowspan-column-color', 'td{border-color:currentColor;color:#5645d4}td.special{color:#e5486a}', '<tr><td rowspan="2"></td><td></td><td></td></tr><tr><td class="special"></td><td></td></tr><tr><td></td><td></td><td></td></tr>'),
  sample('rtl-tie', 'table{direction:rtl}td:nth-child(2){border-color:#e5486a}td:last-child{border-color:#1f9d4c}'),
  sample('script-color', '', rows, "document.querySelectorAll('tr')[1].querySelectorAll('td').forEach(cell=>cell.style.borderColor='#e5486a')"),
  sample('script-span', '', '<tr><td></td><td></td><td></td></tr><tr><td></td><td></td><td></td></tr>', "const cells=document.querySelectorAll('tr')[0].querySelectorAll('td');cells[0].setAttribute('colspan','2');cells[1].remove()"),
  sample('fixed-length', 'table{table-layout:fixed}tr:first-child td:first-child{width:180px;padding:8px}', rows),
  sample('fixed-percentage', 'table{table-layout:fixed}tr:first-child td:first-child{width:50%;padding:8px}', rows),
  sample('fixed-border-box', 'table{table-layout:fixed}tr:first-child td:first-child{box-sizing:border-box;width:180px;padding:8px}', rows),
  sample('caption-border', 'caption{height:20px;border:4px solid #1f9d4c}', '<caption></caption>'+rows),
];
