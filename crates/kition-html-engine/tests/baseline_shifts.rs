use kition_html_engine::{RenderOptions, Resources, render};

fn frame(parent_size: u32, scale: f64, content: &str) -> Result<Vec<u8>, String> {
    let mut resources = Resources::new();
    resources.insert(
        "/assets/Inter.ttf".into(),
        include_bytes!("../../../conformance/assets/Inter-Variable.ttf").to_vec(),
    );
    let result = render(
        &format!(
            "<!doctype html><style>@font-face{{font-family:Inter;src:url(/assets/Inter.ttf)}}body{{margin:0;background:white;font: {parent_size}px/2 Inter}}p{{margin:20px}}span{{font-size:.75em}}i{{display:inline-block;width:10px;height:10px;background:red}}</style><main id=stage><p>A{content}B</p></main>"
        ),
        resources,
        &RenderOptions {
            width: 300,
            height: 200,
            scale,
            ..Default::default()
        },
    )?;
    Ok(result.rgba)
}

#[test]
fn sub_and_super_include_one_css_pixel_beyond_the_parent_font_fraction() -> Result<(), String> {
    for size in [20, 30, 45] {
        for scale in [1.0, 1.5, 2.0] {
            for (keyword, offset) in [
                ("sub", -(size as f64 / 5.0 + 1.0)),
                ("super", size as f64 / 3.0 + 1.0),
            ] {
                let keyword = frame(
                    size,
                    scale,
                    &format!("<span style='vertical-align:{keyword}'>X</span>"),
                )?;
                let explicit = frame(
                    size,
                    scale,
                    &format!("<span style='vertical-align:{offset}px'>X</span>"),
                )?;
                assert_eq!(
                    keyword, explicit,
                    "font size {size}, scale {scale}, offset {offset}"
                );
            }
        }
    }
    Ok(())
}

#[test]
fn atomic_inline_boxes_use_the_same_parent_relative_shift() -> Result<(), String> {
    for scale in [1.0, 1.5, 2.0] {
        for (keyword, offset) in [("sub", -7), ("super", 11)] {
            let a = frame(
                30,
                scale,
                &format!("<i style='vertical-align:{keyword}'></i>"),
            )?;
            let b = frame(
                30,
                scale,
                &format!("<i style='vertical-align:{offset}px'></i>"),
            )?;
            let changed = a
                .as_chunks::<4>()
                .0
                .iter()
                .zip(b.as_chunks::<4>().0.iter())
                .filter(|(a, b)| a != b)
                .count();
            assert!(
                a == b,
                "scale {scale}, keyword {keyword}, changed pixels {changed}"
            );
        }
    }
    Ok(())
}
