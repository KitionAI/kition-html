const visual=(id,type,value='Input text',extra='')=>['input-type-'+id,'forms','input.text{font:14px Inter;width:240px;padding:10px;border:1px solid #c8c4be;color:#1a1a1a}.check{accent-color:#5645d4}',`<input class="${/checkbox|radio/i.test(type)?'check':'text'}" ${type==='missing'?'':`type="${type}"`} value="${value}" ${extra}>`];
const behavior=(id,code)=>['input-type-'+id,'javascript','#target{width:280px;height:96px;background:#e5486a}input{display:none}',`<div id="target"></div><input id="input"><script>const input=document.getElementById('input');if((()=>{${code}})())document.getElementById('target').style.background='#5645d4'</script>`,{requires:'javascript'}];
export default [
 visual('missing','missing'), visual('empty',''), visual('invalid','unknown-kind'),
 visual('uppercase-text','TEXT'), visual('number','number','42'), visual('uppercase-number','NUMBER','42'),
 visual('uppercase-email','EMAIL','mail@example.invalid'), visual('uppercase-tel','TEL','555 0100'),
 visual('uppercase-checkbox','CHECKBOX','','checked'), visual('uppercase-radio','RADIO','','checked'),
 visual('uppercase-hidden','HIDDEN'), visual('uppercase-button','BUTTON','Continue'),
 behavior('getter-missing',"return input.type==='text'&&!input.hasAttribute('type')"),
 behavior('getter-empty',"input.type='';return input.type==='text'&&input.getAttribute('type')===''") ,
 behavior('getter-invalid',"input.type='unknown-kind';return input.type==='text'&&input.getAttribute('type')==='unknown-kind'"),
 behavior('getter-case',"input.type='EMAIL';return input.type==='email'&&input.getAttribute('type')==='EMAIL'"),
 behavior('getter-live',"input.type='number';const first=input.type;input.removeAttribute('type');return first==='number'&&input.type==='text'"),
 behavior('checkbox-value-reflection',"input.type='CHECKBOX';input.value='saved';return input.getAttribute('value')==='saved'&&input.value==='saved'"),
];
