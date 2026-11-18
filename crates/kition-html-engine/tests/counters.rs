use kition_html_engine::{RenderOptions, Resources, render};

fn equivalent(css: &str, body: &str, expected_content: &str) -> Result<(), String> {
    let mut resources = Resources::new();
    resources.insert(
        "/assets/Inter.ttf".into(),
        include_bytes!("../../../conformance/assets/Inter-Variable.ttf").to_vec(),
    );
    let source = |extra: &str| {
        format!(
            "<!doctype html><style>body{{margin:0;font:18px/1.5 Inter}}p{{margin:0}}{css}{extra}</style>{body}"
        )
    };
    let options = RenderOptions {
        width: 320,
        height: 240,
        ..Default::default()
    };
    let actual = render(&source(""), resources.clone(), &options)?;
    let expected = render(&source(expected_content), resources, &options)?;
    assert_eq!(actual.rgba, expected.rgba);
    Ok(())
}

#[test]
fn counters_increment_and_skip_display_none() -> Result<(), String> {
    equivalent(
        "section{counter-reset:item}p{counter-increment:item}p::before{content:counter(item,decimal-leading-zero) '. '} .hidden{display:none}",
        "<section><p class='a'>First</p><p class='hidden'>Hidden</p><p class='b'>Second</p></section>",
        ".a::before{content:'01. '}.b::before{content:'02. '}",
    )
}

#[test]
fn nested_counters_keep_ancestor_values_and_restore_outer_scope() -> Result<(), String> {
    equivalent(
        "section{counter-reset:item}.step{counter-increment:item}.step::before{content:counters(item,'.') ' '}",
        "<section><div class='step a'>Outer<section><div class='step b'>Inner</div><div class='step c'>Inner</div></section></div><div class='step d'>Outer</div></section>",
        ".a::before{content:'1 '}.b::before{content:'1.1 '}.c::before{content:'1.2 '}.d::before{content:'2 '}",
    )
}

#[test]
fn pseudo_elements_increment_in_before_children_after_order() -> Result<(), String> {
    equivalent(
        "body{counter-reset:item}p::before,p::after{counter-increment:item;content:counter(item)}",
        "<p class='a'>A</p><p class='b'>B</p>",
        ".a::before{content:'1'}.a::after{content:'2'}.b::before{content:'3'}.b::after{content:'4'}",
    )
}

#[test]
fn counter_styles_cover_roman_alphabetic_and_signed_padding() -> Result<(), String> {
    equivalent(
        "body{counter-reset:roman 14 alpha 27 negative -1}p::before{content:counter(roman,upper-roman) ' ' counter(alpha,lower-alpha) ' ' counter(negative,decimal-leading-zero)}",
        "<p>Label</p>",
        "p::before{content:'XIV aa -01'}",
    )
}

#[test]
fn sibling_reset_replaces_previous_sibling_counter() -> Result<(), String> {
    equivalent(
        ".a,.b{counter-reset:item 3}.a::before,.b::before,.c::before{content:counters(item,'.')}.c{counter-increment:item 2}",
        "<p class='a'>A</p><p class='b'>B</p><p class='c'>C</p>",
        ".a::before{content:'3'}.b::before{content:'3'}.c::before{content:'5'}",
    )
}

#[test]
fn ordered_lists_support_zero_negative_reversed_and_explicit_values() -> Result<(), String> {
    for (attributes, body, expected) in [
        (
            "start='0'",
            "<li>A</li><li>B</li>",
            "<li>0. A</li><li>1. B</li>",
        ),
        (
            "start='-2'",
            "<li>A</li><li>B</li>",
            "<li>-2. A</li><li>-1. B</li>",
        ),
        (
            "reversed",
            "<li>A</li><li>B</li>",
            "<li>2. A</li><li>1. B</li>",
        ),
        (
            "reversed start='8'",
            "<li>A</li><li value='4'>B</li><li>C</li>",
            "<li>8. A</li><li>4. B</li><li>3. C</li>",
        ),
        (
            "start='3' style='list-style-type:upper-roman'",
            "<li>A</li><li>B</li>",
            "<li>III. A</li><li>IV. B</li>",
        ),
    ] {
        let mut resources = Resources::new();
        resources.insert(
            "/assets/Inter.ttf".into(),
            include_bytes!("../../../conformance/assets/Inter-Variable.ttf").to_vec(),
        );
        let source = |markup: &str| {
            format!(
                "<!doctype html><style>body{{font:18px/1.5 Inter}}ol{{list-style-position:inside}}</style>{markup}"
            )
        };
        let actual = render(
            &source(&format!("<ol {attributes}>{body}</ol>")),
            resources.clone(),
            &RenderOptions::default(),
        )?;
        let expected = render(
            &source(&format!(
                "<ol style='list-style:none;font-variant-numeric:tabular-nums'>{expected}</ol>"
            )),
            resources,
            &RenderOptions::default(),
        )?;
        assert_eq!(actual.rgba, expected.rgba, "{attributes}");
    }
    Ok(())
}

#[test]
fn outside_list_markers_preserve_the_suffix_space() -> Result<(), String> {
    let mut resources = Resources::new();
    resources.insert(
        "/assets/Mono.ttf".into(),
        include_bytes!("../../../conformance/assets/JetBrainsMono-Variable.ttf").to_vec(),
    );
    let source = |body: &str| {
        format!(
            "<!doctype html><style>body{{margin:0;font:20px/1.5 'JetBrains Mono'}}ol{{margin:0;padding-left:100px;list-style-type:upper-roman}}</style>{body}"
        )
    };
    let actual = render(
        &source("<ol start='3'><li>Label</li></ol>"),
        resources.clone(),
        &RenderOptions::default(),
    )?;
    let expected = render(
        &source(
            "<div style='position:relative;margin-left:100px'>Label<span style='position:absolute;right:100%;top:0;white-space:pre'>III. </span></div>",
        ),
        resources,
        &RenderOptions::default(),
    )?;
    assert_eq!(actual.rgba, expected.rgba);
    Ok(())
}
