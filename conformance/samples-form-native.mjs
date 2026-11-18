const check = (id, css = '', body = '<input type="checkbox" checked>') => [
  `form-native-${id}`, 'forms',
  `input{accent-color:#5645d4}section{font:16px/24px Inter}${css}`,
  `<section>${body}</section>`,
];
const select = (id, css, options, script = '') => [
  `form-native-select-${id}`, script ? 'javascript' : 'forms',
  `select{font:14px Inter;color:#1a1a1a}${css}`,
  `<select id="choice">${options}</select>${script ? `<script>${script}</script>` : ''}`,
  script ? {requires:'javascript'} : {},
];
const choices = '<option>English</option><option>French</option>';
const group = `<optgroup label="Languages">${choices}</optgroup>`;
export default [
  check('checkbox-baseline'),
  check('checkbox-text', '', '<label><input type="checkbox" checked> Receive updates</label>'),
  check('checkbox-tall-line', 'section{line-height:40px}'),
  check('checkbox-short-line', 'section{line-height:12px}'),
  check('checkbox-margins', 'input{margin:8px 12px 6px 4px}'),
  check('checkbox-negative-margin', 'input{margin-bottom:-4px}'),
  check('checkbox-middle', 'input{vertical-align:middle}'),
  check('checkbox-custom-size', 'input{width:24px;height:24px}'),
  check('checkbox-appearance-none', 'input{appearance:none;background:#5645d4;border:2px solid #1f9d4c}'),
  check('radio-baseline', '', '<input type="radio" checked>'),
  check('radio-margins', 'input{margin-bottom:6px}', '<input type="radio" checked>'),
  check('radio-text', '', '<label><input type="radio" checked> Weekly summary</label>'),
  select('group-short-label', '', `<optgroup label="A">${choices}</optgroup>`),
  select('group-long-label', '', `<optgroup label="A long category label">${choices}</optgroup>`),
  select('group-wide-option', '', '<optgroup label="A"><option>A much longer option</option><option>Short</option></optgroup>'),
  select('mixed-groups', '', `<option>Ungrouped option</option>${group}`),
  select('group-appearance-none', 'select{appearance:none}', group),
  select('narrow-label', 'select{width:48px}', choices),
  select('narrow-rtl', 'select{width:48px;direction:rtl}', choices),
  select('live-group', '', choices, "const select=document.getElementById('choice'),group=document.createElement('optgroup');group.label='Languages';while(select.firstChild)group.appendChild(select.firstChild);select.appendChild(group)"),
  select('group-small-font', 'select{font-size:10px}', group),
  select('group-large-font', 'select{font-size:24px}', group),
  select('group-monospace', 'select{font-family:"JetBrains Mono"}', group),
  select('group-bold', 'select{font-weight:700}', group),
];
