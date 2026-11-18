const sample=(id,code)=>['event-options-'+id,'javascript','',`<div id="target" class="box"></div><script>const target=document.getElementById('target');function ready(condition){if(condition)target.style.background='#5645d4'}${code}</script>`,{requires:'javascript'}];
export default [
 sample('passive',"target.addEventListener('paint',e=>e.preventDefault(),{passive:true});const event=new Event('paint',{cancelable:true});ready(target.dispatchEvent(event)&&!event.defaultPrevented);"),
 sample('active-after-passive',"target.addEventListener('paint',e=>e.preventDefault(),{passive:true});target.addEventListener('paint',e=>e.preventDefault());const event=new Event('paint',{cancelable:true});ready(!target.dispatchEvent(event)&&event.defaultPrevented);"),
 sample('window-passive',"window.addEventListener('paint',e=>e.preventDefault(),{passive:true});const event=new Event('paint',{cancelable:true});ready(window.dispatchEvent(event)&&!event.defaultPrevented);"),
 sample('passive-checkbox',"const input=document.createElement('input');input.type='checkbox';input.addEventListener('click',e=>e.preventDefault(),{passive:true});input.click();ready(input.checked);"),
 sample('remove-readd',"let count=0;const later=()=>count++;target.addEventListener('paint',()=>{target.removeEventListener('paint',later);target.addEventListener('paint',later)});target.addEventListener('paint',later);target.dispatchEvent(new Event('paint'));ready(count===0);"),
 sample('window-remove-readd',"let count=0;const later=()=>count++;window.addEventListener('paint',()=>{window.removeEventListener('paint',later);window.addEventListener('paint',later)});window.addEventListener('paint',later);window.dispatchEvent(new Event('paint'));ready(count===0);"),
 sample('load-remove-readd',"let count=0;const later=()=>count++;window.addEventListener('load',()=>{window.removeEventListener('load',later);window.addEventListener('load',later);queueMicrotask(()=>ready(count===0))});window.addEventListener('load',later);"),
 sample('load-object',"window.addEventListener('load',{handleEvent(event){ready(event.currentTarget===window&&event.eventPhase===2)}});"),
 sample('load-stop',"window.addEventListener('load',event=>{event.stopImmediatePropagation();ready(true)});window.addEventListener('load',()=>ready(false));window.onload=()=>target.style.background='red';"),
 sample('duplicate-options',"const listener=e=>e.preventDefault();target.addEventListener('paint',listener,{passive:true});target.addEventListener('paint',listener,{passive:false});ready(target.dispatchEvent(new Event('paint',{cancelable:true})));"),
];
