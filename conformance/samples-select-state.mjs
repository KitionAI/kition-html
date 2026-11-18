import cases from './select-state-cases.json' with {type:'json'};
// State traces make intermediate DOM transitions observable without popup UI.
export default cases.map(({id,body,script})=>[
  `select-state-${id}`, 'javascript',
  'select{display:none}pre{margin:0;font:12px/20px "JetBrains Mono";white-space:pre-wrap;color:#1a1a1a}',
  `${body}<pre id="result"></pre><script>${script};document.getElementById('result').textContent=trace.map(step=>JSON.stringify(step)).join('\\n');</script>`,
  {requires:'javascript'},
]);
