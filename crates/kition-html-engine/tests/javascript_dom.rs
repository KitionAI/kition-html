use blitz_dom::{Document, DocumentConfig};
use blitz_traits::shell::{ColorScheme, Viewport};
use blitz_vibey_script::ScriptDocument;

fn document(body: &str) -> ScriptDocument {
    ScriptDocument::from_html(
        &format!("<!doctype html>{body}"),
        DocumentConfig {
            viewport: Some(Viewport::new(640, 480, 1.0, ColorScheme::Light)),
            ..Default::default()
        },
    )
    .without_timer_thread()
}

fn script_test(body: &str, script: &str) {
    let mut doc = document(body);
    doc.eval(&format!(
        r#"function check(value, message) {{ if (!value) throw new Error(message); }} {script}"#
    ));
    let errors = doc.take_js_errors();
    assert!(errors.is_empty(), "{errors:?}\n{script}");
}

#[test]
fn passive_listeners_cannot_cancel_and_do_not_change_duplicate_options() {
    script_test(
        "<div id=target></div>",
        r#"
        for(const target of [document.getElementById('target'),window]) {
            const listener=event=>event.preventDefault();
            target.addEventListener('passive',listener,{passive:true});
            target.addEventListener('passive',listener,{passive:false});
            const event=new Event('passive',{cancelable:true});
            check(target.dispatchEvent(event)&&!event.defaultPrevented,'passive duplicate');
            target.addEventListener('passive',e=>e.preventDefault());
            check(!target.dispatchEvent(event)&&event.defaultPrevented,'active listener after passive');
        }
        const input=document.createElement('input');input.type='checkbox';
        input.addEventListener('click',event=>event.preventDefault(),{passive:true});
        input.click();check(input.checked,'passive listener leaves activation enabled');
    "#,
    );
}

#[test]
fn removing_and_readding_a_listener_does_not_revive_the_dispatch_snapshot() {
    script_test(
        "<div id=target></div>",
        r#"
        for(const target of [document.getElementById('target'),window]) {
            let count=0;
            const later=()=>count++;
            const first=()=>{target.removeEventListener('paint',later);target.addEventListener('paint',later)};
            target.addEventListener('paint',first);target.addEventListener('paint',later);
            target.dispatchEvent(new Event('paint'));check(count===0,'removed snapshot entry');
            target.removeEventListener('paint',first);target.dispatchEvent(new Event('paint'));
            check(count===1,'new registration runs next time');
        }
    "#,
    );
}

#[test]
fn native_window_load_uses_shared_dispatch_and_object_callbacks() {
    let mut doc = document(
        "<script>let count=0;const later=()=>{throw Error('removed listener ran')};window.addEventListener('load',{handleEvent(event){if(event.currentTarget!==window||event.eventPhase!==2)throw Error('load event fields');count++;window.removeEventListener('load',later);window.addEventListener('load',later)}});window.addEventListener('load',later);window.onload=()=>{if(count!==1)throw Error('load listener order')}</script>",
    );
    doc.execute_scripts();
    assert!(doc.take_js_errors().is_empty());
}

#[test]
fn empty_inline_descendants_preserve_block_dimensions_after_restyle() -> Result<(), String> {
    for child in [
        "<span id=leaf></span>",
        "<span id=leaf> </span>",
        "<span id=leaf><em></em></span>",
    ] {
        let mut doc = document(&format!(
            "<style>.box{{width:160px;height:96px;background:red}}section.ready .box{{background:#5645d4}}</style><main><section><div class=box>{child}</div></section></main>"
        ));
        for script in [
            "",
            "document.getElementById('leaf').closest('section').classList.add('ready')",
            "document.querySelector('.box').style.width='200px'",
        ] {
            doc.eval(script);
            doc.inner_mut().resolve(0.0);
            let dom = doc.inner();
            let id = dom
                .query_selector(".box")
                .map_err(|_| "Invalid selector")?
                .ok_or("Missing box")?;
            let rect = dom.get_client_bounding_rect(id).ok_or("Missing bounds")?;
            assert_eq!(
                rect.width,
                if script.contains("200px") {
                    200.0
                } else {
                    160.0
                }
            );
            assert_eq!(rect.height, 96.0);
        }
        assert!(doc.take_js_errors().is_empty());
    }
    Ok(())
}

#[test]
fn script_events_capture_target_bubble_and_cleanup() {
    script_test(
        "<section id=parent><div id=target></div></section>",
        r#"
        const parent=document.getElementById('parent'), target=document.getElementById('target');
        const seen=[];
        for (const [node,name] of [[window,'window'],[document,'document'],[parent,'parent'],[target,'target']]) {
            for (const capture of [true,false]) node.addEventListener('paint',event=>{
                check(event.target===target,'target');check(event.currentTarget===node,'current target');
                seen.push(name+':'+capture+':'+event.eventPhase);
            },capture);
        }
        const detail={color:'#5645d4'};
        const event=new CustomEvent('paint',{bubbles:true,cancelable:true,detail});
        check(event instanceof Event && event instanceof CustomEvent,'constructors');
        check(event.detail===detail && !event.isTrusted && event.target===null,'initial state');
        target.dispatchEvent(event);
        check(seen.join(',')==='window:true:1,document:true:1,parent:true:1,target:true:2,target:false:2,parent:false:3,document:false:3,window:false:3',seen.join(','));
        check(event.currentTarget===null&&event.eventPhase===0&&event.target===target,'cleanup');
    "#,
    );
}

#[test]
fn event_cancel_stop_once_removal_and_reentrancy() {
    script_test(
        "<section id=parent><div id=target></div></section>",
        r#"
        const parent=document.getElementById('parent'),target=document.getElementById('target');
        let canceled=0;
        target.addEventListener('paint',event=>{event.preventDefault();canceled++},{once:true});
        let event=new Event('paint');
        check(target.dispatchEvent(event)&&!event.defaultPrevented,'not cancelable');
        check(target.dispatchEvent(event)&&canceled===1,'once removed');
        target.addEventListener('paint',event=>event.preventDefault());
        event=new Event('paint',{cancelable:true});
        check(!target.dispatchEvent(event)&&event.defaultPrevented,'canceled');
        let order=[];
        target.addEventListener('click',event=>{order.push(1);event.stopPropagation();});
        target.addEventListener('click',()=>order.push(2));
        parent.addEventListener('click',()=>order.push(3));target.click();
        check(order.join(',')==='1,2','stop propagation');
        order=[];
        const removed=()=>order.push(9);
        target.addEventListener('remove',()=>target.removeEventListener('remove',removed));
        target.addEventListener('remove',removed);target.dispatchEvent(new Event('remove'));
        check(order.length===0,'removed listener');
        let once=0;
        target.addEventListener('nested',()=>{once++;target.dispatchEvent(new Event('nested'));},{once:true});
        target.dispatchEvent(new Event('nested'));check(once===1,'once reentrant');
        target.addEventListener('same',event=>{
            let rejected=false;try{target.dispatchEvent(event);}catch(error){rejected=error.name==='InvalidStateError';}
            check(rejected,'redispatch guard');
        });target.dispatchEvent(new Event('same'));
        const object={count:0,handleEvent(){this.count++;}};
        target.addEventListener('object',object);target.dispatchEvent(new Event('object'));
        check(object.count===1,'listener object');
    "#,
    );
}

#[test]
fn click_inline_handlers_disabled_elements_and_detached_nodes() {
    script_test(
        "<button id=target onclick=\"this.dataset.clicked=event.type;return false\"></button><button id=disabled disabled></button>",
        r#"
        const target=document.getElementById('target');
        const event=new MouseEvent('click',{cancelable:true});
        check(!target.dispatchEvent(event)&&target.dataset.clicked==='click','inline handler');
        let disabled=0;document.getElementById('disabled').addEventListener('click',()=>disabled++);
        document.getElementById('disabled').click();check(disabled===0,'disabled click');
        let count=0;target.onclick=()=>{count++;target.click();};target.click();check(count===1,'click recursion');
        let leaked=false;window.addEventListener('detached',()=>leaked=true);
        document.createElement('div').dispatchEvent(new Event('detached',{bubbles:true}));
        check(!leaked,'detached propagation');
    "#,
    );
}

#[test]
fn adjacent_html_and_outer_html_preserve_context_and_order() {
    script_test(
        "<div id=parent><span id=target>middle</span></div><table><tbody id=rows></tbody></table>",
        r#"
        const target=document.getElementById('target');
        target.insertAdjacentHTML('beforebegin','<b>first</b>');
        target.insertAdjacentHTML('AFTERBEGIN','<i>start</i>');
        target.insertAdjacentHTML('beforeend','<i>end</i>');
        target.insertAdjacentHTML('afterend','<b>last</b>');
        check(document.getElementById('parent').textContent==='firststartmiddleendlast','adjacent order');
        document.getElementById('rows').insertAdjacentHTML('beforeend','<tr><td>cell</td></tr>');
        check(document.querySelector('#rows > tr > td').textContent==='cell','table context');
        target.outerHTML='<em id=replacement>new</em>';
        check(document.getElementById('parent').textContent==='firstnewlast','outer replacement');
        let invalid=false;try{target.insertAdjacentHTML('sideways','');}catch(e){invalid=e.name==='SyntaxError';}
        check(invalid,'invalid position');
        invalid=false;try{document.documentElement.insertAdjacentHTML('beforebegin','x');}catch(e){invalid=e.name==='NoModificationAllowedError';}
        check(invalid,'document parent');
    "#,
    );
}

#[test]
fn media_queries_use_stylesheet_parser_and_viewport() {
    script_test(
        "<div></div>",
        r#"
        for(const [query,expected] of [
            ['(min-width:600px)',true],['(max-width:400px)',false],['(width >= 600px)',true],
            ['screen and (orientation:landscape)',true],['print',false],['(prefers-color-scheme:light)',true],
            ['not screen',false],['(unsupported-feature:yes)',false],['nonsense(',false],['',true],
            ['print, (height:480px)',true]
        ]) check(matchMedia(query).matches===expected,query);
        const list=matchMedia('(min-width: 1px)');check(list instanceof MediaQueryList&&typeof list.media==='string','interface');
    "#,
    );
}

#[test]
fn live_collections_and_static_node_lists_keep_browser_membership_rules() {
    script_test(
        "<section id=target><i id=first class=tile></i></section>",
        r#"
        const target=document.getElementById('target');
        const live=document.getElementsByClassName('tile');
        const tags=target.getElementsByTagName('i');
        const nodes=target.childNodes, children=target.children, snapshot=target.querySelectorAll('i');
        check(live instanceof HTMLCollection && snapshot instanceof NodeList,'interfaces');
        check(nodes===target.childNodes && children===target.children,'same object');
        target.insertAdjacentHTML('beforeend','<i id=second class=tile></i>');
        check(live.length===2&&tags.length===2&&snapshot.length===1,'live versus static');
        check(nodes.length===2&&children.length===2,'live children');
        check(live.namedItem('second')===live[1]&&live.second===live.item(1),'named items');
        check(live.item(9)===null&&live[9]===undefined,'missing index');
        check([...live].length===2&&Object.keys(live).join(',')==='0,1','iteration');
        live[0].classList.remove('tile');check(live.length===1&&live[0].id==='second','class update');
        target.removeChild(target.firstChild);check(tags.length===1&&snapshot[0].id==='first','removal and snapshot');
    "#,
    );
}
