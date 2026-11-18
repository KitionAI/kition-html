use kition_html_engine::{RenderOptions, Rendered, Resources, render};

fn frame(css: &str, body: &str, scale: f64) -> Result<Rendered, String> {
    let resources = Resources::from([(
        "/assets/Inter-Variable.ttf".into(),
        include_bytes!("../../../conformance/assets/Inter-Variable.ttf").to_vec(),
    )]);
    render(
        &format!(
            "<!doctype html><style>body{{margin:20px;background:white;font:16px/24px Inter}}section{{width:320px}}p{{margin:0}}i{{display:block;height:30px;background:#5645d4}}i:nth-child(2n){{background:#e5486a}}{css}</style><main id=stage>{body}</main>"
        ),
        resources,
        &RenderOptions {
            width: 420,
            height: 400,
            scale,
            ..Default::default()
        },
    )
}
fn blocks() -> &'static str {
    "<i id=a></i><i id=b></i><i id=c></i><i id=d></i><i id=e></i><i id=f></i>"
}
fn rect(image: &Rendered, id: &str) -> [f64; 4] {
    let Some(value) = image.geometry.iter().find(|v| v["id"] == id) else {
        panic!("missing element geometry: {id}");
    };
    ["x", "y", "width", "height"].map(|key| value[key].as_f64().unwrap_or(f64::NAN))
}
fn pixel(image: &Rendered, x: usize, y: usize) -> &[u8] {
    let i = (y * image.width as usize + x) * 4;
    &image.rgba[i..i + 4]
}
#[test]
fn balanced_columns_have_fragment_aware_bounds_and_pixel_edges() -> Result<(), String> {
    for scale in [1.0, 1.5, 2.0] {
        let image = frame(
            "section{columns:3;column-gap:20px}",
            &format!("<section id=cols>{}</section>", blocks()),
            scale,
        )?;
        assert_eq!(rect(&image, "cols"), [20.0, 20.0, 320.0, 60.0]);
        assert_eq!(rect(&image, "a"), [20.0, 20.0, 93.328125, 30.0]);
        assert_eq!(rect(&image, "c"), [133.328125, 20.0, 93.328125, 30.0]);
        assert_eq!(rect(&image, "e"), [246.671875, 20.0, 93.328125, 30.0]);
        for (x, expected) in [
            (112, [86, 69, 212, 255]),
            (114, [255; 4]),
            (134, [86, 69, 212, 255]),
            (228, [255; 4]),
            (248, [86, 69, 212, 255]),
        ] {
            assert_eq!(
                pixel(
                    &image,
                    (f64::from(x) * scale) as usize,
                    (30.0 * scale) as usize
                ),
                expected
            );
        }
        assert!(image.diagnostics.is_empty(), "{:?}", image.diagnostics);
    }
    Ok(())
}
#[test]
fn fixed_auto_fill_splits_empty_blocks_and_adds_overflow_columns() -> Result<(), String> {
    let image = frame(
        "section{columns:2;height:50px;column-fill:auto}",
        &format!("<section id=cols>{}</section>", blocks()),
        1.0,
    )?;
    assert_eq!(rect(&image, "cols"), [20.0, 20.0, 320.0, 50.0]);
    assert_eq!(rect(&image, "b"), [20.0, 20.0, 320.0, 50.0]);
    assert_eq!(rect(&image, "c"), [188.0, 30.0, 152.0, 30.0]);
    assert_eq!(rect(&image, "f"), [524.0, 20.0, 152.0, 30.0]);
    Ok(())
}
#[test]
fn auto_fill_without_a_height_keeps_one_column() -> Result<(), String> {
    let image = frame(
        "section{columns:3;column-fill:auto}",
        &format!("<section id=cols>{}</section>", blocks()),
        1.0,
    )?;
    assert_eq!(rect(&image, "cols"), [20.0, 20.0, 320.0, 180.0]);
    assert_eq!(rect(&image, "f"), [20.0, 170.0, 96.0, 30.0]);
    Ok(())
}
#[test]
fn column_width_count_padding_and_direction_share_the_same_layout() -> Result<(), String> {
    let image = frame(
        "section{columns:2 80px;column-gap:20px;padding:10px;border:2px solid black;direction:rtl}",
        &format!("<section id=cols>{}</section>", blocks()),
        1.0,
    )?;
    assert_eq!(rect(&image, "cols"), [20.0, 20.0, 344.0, 114.0]);
    assert_eq!(rect(&image, "a"), [202.0, 32.0, 150.0, 30.0]);
    assert_eq!(rect(&image, "d"), [32.0, 32.0, 150.0, 30.0]);
    Ok(())
}
#[test]
fn solid_and_double_rules_snap_before_device_scaling() -> Result<(), String> {
    for scale in [1.0, 2.0] {
        for rule in ["2px solid", "6px double"] {
            let image = frame(
                &format!("section{{columns:3;column-gap:24px;column-rule:{rule} #5645d4}}"),
                &format!("<section>{}</section>", blocks()),
                scale,
            )?;
            let x = if rule.contains("double") { 120 } else { 122 };
            assert_eq!(
                pixel(
                    &image,
                    (f64::from(x) * scale) as usize,
                    (30.0 * scale) as usize
                ),
                [86, 69, 212, 255]
            );
            assert_eq!(
                pixel(&image, (126.0 * scale) as usize, (30.0 * scale) as usize),
                [255; 4]
            );
        }
    }
    Ok(())
}
#[test]
fn default_widows_move_two_final_lines_together() -> Result<(), String> {
    let body = "<section id=cols><p id=text>one<br>two<br>three<br>four<br>five<br>six<br>seven</p></section>";
    let balanced = frame("section{columns:3}", body, 1.0)?;
    let explicit = frame("section{columns:3}p{widows:2;orphans:2}", body, 1.0)?;
    let relaxed = frame("section{columns:3}p{widows:1;orphans:1}", body, 1.0)?;
    assert_eq!(rect(&balanced, "cols"), [20.0, 20.0, 320.0, 72.0]);
    assert_eq!(rect(&balanced, "text"), [20.0, 20.0, 320.0, 72.0]);
    assert_eq!(balanced.rgba, explicit.rgba);
    assert_ne!(balanced.rgba, relaxed.rgba);
    Ok(())
}
#[test]
fn paragraph_offsets_are_not_counted_twice() -> Result<(), String> {
    let image = frame(
        "section{columns:2}p{margin-bottom:12px}",
        "<section id=cols><p>one<br>two</p><p id=second>three<br>four</p></section>",
        1.0,
    )?;
    assert_eq!(rect(&image, "cols"), [20.0, 20.0, 320.0, 60.0]);
    assert_eq!(rect(&image, "second"), [188.0, 20.0, 152.0, 48.0]);
    Ok(())
}
#[test]
fn nested_columns_map_inner_fragments_before_outer_fragments() -> Result<(), String> {
    let image = frame(
        "section,.inner{columns:2}",
        &format!(
            "<section id=cols><div class=inner>{}</div><div class=inner><i></i><i></i><i></i><i id=last></i><i></i><i></i></div></section>",
            blocks()
        ),
        1.0,
    )?;
    assert_eq!(rect(&image, "cols"), [20.0, 20.0, 320.0, 90.0]);
    assert_eq!(rect(&image, "last"), [272.0, 20.0, 68.0, 30.0]);
    Ok(())
}
#[test]
fn unsupported_spanners_and_positioning_have_explicit_diagnostics() -> Result<(), String> {
    let image = frame(
        "section{columns:2}h1{column-span:all}b{position:absolute}",
        "<section><h1>Heading</h1><b>Positioned</b><p>Content</p></section>",
        1.0,
    )?;
    assert!(
        image
            .diagnostics
            .iter()
            .any(|s| s.starts_with("html.unsupported-column-span:"))
    );
    // A positioned node hoisted outside the columns still receives its normal
    // containing-block placement; only fragment-painted descendants need a warning.
    Ok(())
}
#[test]
fn pathological_fragment_counts_are_bounded_and_reported() -> Result<(), String> {
    let image = frame(
        "section{columns:2;height:0.015625px;column-fill:auto}i{height:100px}",
        "<section><i></i></section>",
        1.0,
    )?;
    assert!(
        image
            .diagnostics
            .iter()
            .any(|s| s.starts_with("html.column-fragment-limit:"))
    );
    Ok(())
}
#[test]
fn height_limits_constrain_the_box_and_fragmentainers_separately() -> Result<(), String> {
    for (property, height) in [("min-height:100px", 100.0), ("max-height:40px", 40.0)] {
        let image = frame(
            &format!("section{{columns:3;{property}}}"),
            &format!("<section id=cols>{}</section>", blocks()),
            1.0,
        )?;
        assert_eq!(rect(&image, "cols"), [20.0, 20.0, 320.0, height]);
        if property.starts_with("max") {
            assert_eq!(rect(&image, "e"), [356.0, 20.0, 96.0, 30.0]);
        } else {
            assert_eq!(rect(&image, "e"), [244.0, 20.0, 96.0, 30.0]);
        }
    }
    Ok(())
}
#[test]
fn break_inside_avoid_keeps_a_nested_paragraph_together() -> Result<(), String> {
    let image = frame(
        "section{columns:3}article{break-inside:avoid}",
        "<section id=cols><article><p>one<br>two<br>three</p></article><article id=second><p>four<br>five<br>six<br>seven</p></article></section>",
        1.0,
    )?;
    assert_eq!(rect(&image, "cols"), [20.0, 20.0, 320.0, 96.0]);
    assert_eq!(rect(&image, "second"), [132.0, 20.0, 96.0, 96.0]);
    Ok(())
}
#[test]
fn sliced_borders_include_the_final_padding_in_balance() -> Result<(), String> {
    let image = frame(
        "section{columns:3}article{border:3px solid black;padding:8px;background:#eee}",
        &format!(
            "<section id=cols><article id=box>{}</article></section>",
            blocks()
        ),
        1.0,
    )?;
    assert_eq!(rect(&image, "cols"), [20.0, 20.0, 320.0, 67.34375]);
    assert_eq!(rect(&image, "box"), [20.0, 20.0, 320.0, 67.34375]);
    assert_eq!(pixel(&image, 22, 21), [0, 0, 0, 255]);
    assert_eq!(pixel(&image, 136, 21), [238, 238, 238, 255]);
    Ok(())
}
