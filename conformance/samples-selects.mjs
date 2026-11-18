const options='<option value="en">English</option><option value="fr">French</option><option value="pt">Portuguese</option>';
const sample=(id,attributes='',css='',items=options,script='')=>['select-'+id,script?'javascript':'forms',`select{font:14px Inter;color:#1a1a1a}${css}`,`<select id="choice" ${attributes}>${items}</select>${script?`<script>${script}</script>`:''}`,{requires:script?'javascript':'static-html-css'}];
export default [
 sample('default'),
 sample('selected','','','<option>English</option><option selected>French</option>'),
 sample('label','','','<option label="Display label">Stored text</option>'),
 sample('empty','','',''),
 sample('padded','','select{width:240px;padding:10px;border:1px solid #c8c4be;border-radius:8px}'),
 sample('large-font','','select{font-size:24px}'),
 sample('line-height','','select{line-height:3}'),
 sample('appearance-none','','select{appearance:none;padding:8px;border:1px solid #c8c4be}'),
 sample('disabled','disabled'),
 sample('rtl','dir="rtl"','select{width:240px;padding:8px}'),
 sample('height','','select{width:240px;height:64px}'),
 sample('narrow','','select{width:48px}'),
 sample('option-group','','','<optgroup label="Languages"><option>English</option><option selected>French</option></optgroup>'),
 sample('disabled-first','','','<option disabled>Unavailable</option><option>Available</option>'),
 sample('whitespace','','','<option>  English\n  language  </option>'),
 sample('value','','',options,"document.getElementById('choice').value='fr'"),
 sample('index','','',options,"document.getElementById('choice').selectedIndex=2"),
 sample('option-selected','','',options,"document.getElementById('choice').children[1].selected=true"),
 sample('no-selection','','',options,"document.getElementById('choice').selectedIndex=-1"),
 sample('changed-text','','',options,"document.getElementById('choice').children[0].textContent='Updated option'"),
];
