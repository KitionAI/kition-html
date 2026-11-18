use kition_html_engine::{RenderOptions, Resources, render};

#[test]
fn angle_gradients_use_the_tile_center_and_preserve_transparent_layers() -> Result<(), String> {
    for direction in ["90deg", "to right"] {
        let html = format!(
            "<!doctype html><style>body{{margin:0}}div{{width:320px;height:180px;background:linear-gradient({direction},transparent 50%,#e5486a 50%) 0 0/64px 100%,#5645d4}}</style><div></div>"
        );
        let frame = render(
            &html,
            Resources::new(),
            &RenderOptions {
                width: 400,
                height: 240,
                ..Default::default()
            },
        )?;
        for x in 0..320 {
            let offset = (40 * 400 + x) * 4;
            let color = if x % 64 < 32 {
                [86, 69, 212, 255]
            } else {
                [229, 72, 106, 255]
            };
            assert_eq!(&frame.rgba[offset..offset + 4], &color, "{direction} x={x}");
        }
    }
    Ok(())
}

#[test]
fn empty_inline_blocks_keep_auto_width_and_zero_intrinsic_height() -> Result<(), String> {
    let frame = render(
        "<!doctype html><style>body{margin:0}section{width:200px}#target{background:red}</style><main id=stage><section><div id=target><span></span></div></section></main>",
        Resources::new(),
        &RenderOptions {
            width: 320,
            height: 240,
            ..Default::default()
        },
    )?;
    let target = frame
        .geometry
        .iter()
        .find(|node| node["id"] == "target")
        .ok_or("Missing target")?;
    assert_eq!(target["width"], 200.0);
    assert_eq!(target["height"], 0.0);
    Ok(())
}
