use blitz_dom::DocumentConfig;
use blitz_vibey_script::ScriptDocument;
use kition_html_engine::{RenderOptions, Resources, Session, render};

fn check(body: &str, script: &str) {
    let mut document =
        ScriptDocument::from_html(&format!("<!doctype html>{body}"), DocumentConfig::default())
            .without_timer_thread();
    document.eval(&format!(
        "function check(value,message){{if(!value)throw Error(message)}}{script}"
    ));
    let errors = document.take_js_errors();
    assert!(errors.is_empty(), "{errors:?}");
}

#[test]
fn checked_properties_do_not_rewrite_default_attributes() {
    check(
        "<input id=control type=checkbox checked>",
        r#"
        const control=document.getElementById('control');
        control.checked=false;
        check(!control.checked&&control.hasAttribute('checked')&&control.defaultChecked,'property vs default');
        control.setAttribute('checked','');check(!control.checked,'dirty checkedness');
        control.defaultChecked=false;check(!control.hasAttribute('checked'),'default attribute');
        control.checked=true;check(control.checked&&!control.hasAttribute('checked'),'source retained');
        const clone=control.cloneNode();check(clone.checked&&!clone.hasAttribute('checked'),'clone state');
        const fresh=document.createElement('input');fresh.setAttribute('type','checkbox');fresh.setAttribute('checked','false');
        check(fresh.checked,'boolean attribute presence');
    "#,
    );
}

#[test]
fn checkbox_preactivation_cancel_and_event_types_match_html() {
    check(
        "<input id=control type=checkbox>",
        r#"
        const control=document.getElementById('control'),order=[];
        for(const type of ['click','input','change'])control.addEventListener(type,event=>order.push(type+':'+control.checked));
        control.click();check(order.join(',')==='click:true,input:true,change:true',order.join(','));
        order.length=0;control.addEventListener('click',event=>event.preventDefault(),{once:true});
        control.click();check(control.checked&&order.join(',')==='click:false','cancel restoration');
        control.checked=false;control.dispatchEvent(new Event('click'));check(!control.checked,'generic Event has no activation');
        control.dispatchEvent(new MouseEvent('click'));check(control.checked,'MouseEvent activation');
        check(!control.hasAttribute('checked'),'source retained');
    "#,
    );
}

#[test]
fn radios_are_scoped_by_owner_and_emit_changes_only_when_needed() {
    check(
        "<form><input id=a type=radio name=choice checked><input id=b type=radio name=choice></form><form><input id=c type=radio name=choice checked></form>",
        r#"
        const a=document.getElementById('a'),b=document.getElementById('b'),c=document.getElementById('c');
        b.checked=true;check(!a.checked&&b.checked&&c.checked,'form groups');
        let changes=0;a.addEventListener('change',()=>changes++);a.click();a.click();
        check(a.checked&&!b.checked&&c.checked&&changes===1,'radio activation');
        b.onclick=event=>event.preventDefault();b.click();check(a.checked&&!b.checked,'canceled radio');
        const detached=document.createElement('input');detached.type='radio';detached.setAttribute('name','choice');
        detached.checked=true;check(a.checked&&detached.checked,'detached tree');
    "#,
    );
}

#[test]
fn input_and_textarea_live_values_preserve_source_defaults() {
    check(
        "<input id=input value=original><textarea id=area>original</textarea>",
        r#"
        for(const id of ['input','area']) {
            const control=document.getElementById(id);check(control.value==='original','initial value '+id);
            control.value='edited';check(control.value==='edited'&&control.defaultValue==='original','dirty value '+id);
            control.defaultValue='default';check(control.value==='edited'&&control.defaultValue==='default','default value '+id);
        }
        const input=document.getElementById('input');input.setAttribute('value','attribute');
        check(input.value==='edited','attribute does not overwrite live edit');
    "#,
    );
}

#[test]
fn select_and_option_state_follow_values_and_indices() {
    check(
        "<select id=control><option value=a>Alpha</option><option value=b selected>Beta</option><option> Gamma </option></select>",
        r#"
        const control=document.getElementById('control'),options=control.getElementsByTagName('option');
        check(control.value==='b'&&control.selectedIndex===1&&options[1].selected,'default selection');
        control.value='a';check(control.selectedIndex===0&&options[0].selected&&!options[1].selected,'set value');
        check(options[1].hasAttribute('selected'),'selection default retained');
        control.selectedIndex=2;check(control.value==='Gamma'&&options[2].value==='Gamma','option text value');
        control.value='missing';check(control.selectedIndex===-1&&control.value==='','missing value');
        options[1].selected=true;check(control.value==='b'&&!options[0].selected,'option selection');
    "#,
    );
}

#[test]
fn summary_activation_toggles_details_and_can_be_canceled() {
    check(
        "<details id=details><summary id=control>Details</summary><p>Content</p></details>",
        r#"
        const details=document.getElementById('details'),control=document.getElementById('control');
        control.click();check(details.open&&details.hasAttribute('open'),'summary activation');
        control.onclick=event=>event.preventDefault();control.click();check(details.open,'cancel details');
        details.open=false;check(!details.hasAttribute('open'),'open reflection');
    "#,
    );
}

#[test]
fn checked_state_restyles_css_and_pointer_clicks_do_not_double_toggle() -> Result<(), String> {
    let html = "<!doctype html><style>body{margin:0}input{appearance:none;display:block;width:32px;height:32px;padding:0;margin:0;border:0;background:red}input:checked{background:#5645d4}</style><input id=control type=checkbox><script>document.getElementById('control').checked=true</script>";
    let options = RenderOptions {
        width: 64,
        height: 64,
        ..Default::default()
    };
    let image = render(html, Resources::new(), &options)?;
    assert_eq!(&image.rgba[..4], &[86, 69, 212, 255]);
    assert!(image.diagnostics.is_empty(), "{:?}", image.diagnostics);
    let mut session = Session::new(html, Resources::new(), &options)?;
    session.frame(1.0)?;
    session.click(4.0, 4.0)?;
    assert_eq!(&session.frame(1.0)?.rgba[..4], &[255, 0, 0, 255]);
    session.click(4.0, 4.0)?;
    assert_eq!(&session.frame(1.0)?.rgba[..4], &[86, 69, 212, 255]);
    Ok(())
}

#[test]
fn reset_is_cancelable_and_restores_defaults_without_input_events() {
    check(
        "<form id=form><input id=input value=original><input id=box type=checkbox checked><textarea id=area>original</textarea><select id=select><option value=a>A</option><option value=b selected>B</option></select><button type=reset id=button>Reset</button></form><input id=external form=form value=outside>",
        r#"
        const form=document.getElementById('form'),input=document.getElementById('input'),box=document.getElementById('box'),area=document.getElementById('area'),select=document.getElementById('select'),external=document.getElementById('external');
        input.value='edited';box.checked=false;area.value='edited';select.value='a';external.value='edited';
        let inputEvents=0,resets=0,during='';
        form.addEventListener('input',()=>inputEvents++);
        form.addEventListener('reset',()=>{resets++;during=input.value;form.reset()});
        const cancel=e=>e.preventDefault();form.addEventListener('reset',cancel);
        form.reset();check(input.value==='edited'&&resets===1,'cancel reset and recursion');
        form.removeEventListener('reset',cancel);document.getElementById('button').click();
        check(resets===2&&during==='edited'&&input.value==='original','reset order');
        check(box.checked&&area.value==='original'&&select.value==='b'&&external.value==='outside','all controls');
        check(inputEvents===0,'reset does not fire input');
        input.defaultValue='next';box.defaultChecked=false;check(input.value==='next'&&!box.checked,'dirty flags cleared');
    "#,
    );
}

#[test]
fn disabled_fieldsets_except_the_first_legend_block_clicks() {
    check(
        "<fieldset id=fieldset disabled><legend><input id=first type=checkbox></legend><input id=second type=checkbox><legend><input id=third type=checkbox></legend></fieldset>",
        r#"
        const first=document.getElementById('first'),second=document.getElementById('second'),third=document.getElementById('third'),fieldset=document.getElementById('fieldset');
        let clicks=0;second.onclick=()=>clicks++;
        first.click();second.click();third.click();
        check(first.checked&&!second.checked&&!third.checked&&clicks===0,'disabled activation');
        check(!first.matches(':disabled')&&second.matches(':disabled')&&third.matches(':disabled'),'disabled selector');
        check(!second.disabled&&fieldset.disabled,'disabled property reflects only own attribute');
        fieldset.disabled=false;second.click();check(second.checked&&clicks===1,'enable ancestor');
    "#,
    );
}

#[test]
fn label_activation_retargets_click_and_honors_cancelation() {
    check(
        "<label id=label for=control><span id=text>Check</span></label><input id=control type=checkbox><label><button id=button type=button>Button</button><input id=other type=checkbox></label>",
        r#"
        const control=document.getElementById('control'),label=document.getElementById('label');
        const order=[];control.onclick=e=>order.push(e.target===control&&e instanceof MouseEvent);control.onchange=()=>order.push('change');
        document.getElementById('text').click();check(control.checked&&order.join(',')==='true,change','forwarded click');
        label.onclick=e=>e.preventDefault();label.click();check(control.checked,'canceled label');
        document.getElementById('button').click();check(!document.getElementById('other').checked,'interactive descendant');
    "#,
    );
}

#[test]
fn indeterminate_preactivation_restores_state_when_canceled() {
    check(
        "<input id=control type=checkbox>",
        r#"
        const control=document.getElementById('control');control.indeterminate=true;
        check(control.matches(':indeterminate'),'indeterminate selector');
        control.onclick=e=>{check(!control.indeterminate&&control.checked,'preactivation');e.preventDefault()};
        control.click();check(control.indeterminate&&!control.checked,'restore state');
        control.onclick=null;control.click();check(!control.indeterminate&&control.checked,'activate');
    "#,
    );
}

#[test]
fn form_owner_tracks_dom_moves_and_explicit_invalid_references() {
    check(
        "<form id=first><input id=control><input id=invalid form=missing value=original></form><form id=second></form>",
        r#"
        const first=document.getElementById('first'),second=document.getElementById('second'),control=document.getElementById('control'),invalid=document.getElementById('invalid');
        check(control.form===first&&invalid.form===null,'initial owners');
        invalid.value='edited';first.reset();check(invalid.value==='edited','unowned control');
        second.appendChild(control);check(control.form===second,'moved owner');
        control.setAttribute('form','first');check(control.form===first,'explicit owner');
        control.remove();check(control.form===null,'detached explicit owner');
    "#,
    );
}

#[test]
fn changing_ancestor_disabled_and_indeterminate_state_repaints_existing_controls()
-> Result<(), String> {
    let html = "<!doctype html><style>body{margin:0;min-height:64px}fieldset{display:contents}input{appearance:none;display:block;width:32px;height:32px;padding:0;margin:0;border:0;background:red}input:disabled,input:indeterminate{background:#5645d4}</style><fieldset id=fieldset><input id=control type=checkbox></fieldset><script>let count=0;document.body.onclick=()=>{if(++count===1)document.getElementById('fieldset').disabled=true;else{document.getElementById('fieldset').disabled=false;document.getElementById('control').indeterminate=count===3}}</script>";
    let options = RenderOptions {
        width: 64,
        height: 64,
        ..Default::default()
    };
    let mut session = Session::new(html, Resources::new(), &options)?;
    for expected in [
        [255, 0, 0, 255],
        [86, 69, 212, 255],
        [255, 0, 0, 255],
        [86, 69, 212, 255],
    ] {
        assert_eq!(&session.frame(1.0)?.rgba[..4], &expected);
        session.click(48.0, 48.0)?;
    }
    Ok(())
}
