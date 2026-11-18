use kition_html_engine::{RenderOptions, Rendered, Resources, render};
fn frame(css: &str, scale: f64) -> Result<Rendered, String> {
    let mut resources = Resources::new();
    resources.insert(
        "/assets/Inter.ttf".into(),
        include_bytes!("../../../conformance/assets/Inter-Variable.ttf").to_vec(),
    );
    render(
        &format!(
            "<!doctype html><style>@font-face{{font-family:Inter;src:url(/assets/Inter.ttf)}}body{{margin:0;background:white}}p{{margin:20px;font:24px/48px Inter;color:transparent;text-decoration:underline wavy red 2px;text-underline-offset:6px;{css}}}</style><main id=stage><p>MMMMMMMM</p></main>"
        ),
        resources,
        &RenderOptions {
            width: 260,
            height: 120,
            scale,
            ..Default::default()
        },
    )
}
#[test]
fn cubic_waves_repeat_at_the_same_css_period_at_each_device_scale() -> Result<(), String> {
    for scale in [1.0, 2.0] {
        let image = frame("", scale)?;
        let period = (11.0 * scale) as usize;
        let mut nonwhite = 0;
        for y in 0..image.height as usize {
            for x in (40.0 * scale) as usize..(80.0 * scale) as usize {
                let a = (y * image.width as usize + x) * 4;
                let b = a + period * 4;
                assert!(
                    image.rgba[a..a + 4]
                        .iter()
                        .zip(&image.rgba[b..b + 4])
                        .all(|(&a, &b)| a.abs_diff(b) <= 1)
                );
                if image.rgba[a + 1] < 255 {
                    nonwhite += 1;
                }
            }
        }
        assert!(nonwhite > 80);
    }
    Ok(())
}
#[test]
fn wave_endpoints_clip_to_the_decoration_extent() -> Result<(), String> {
    for scale in [1.0, 2.0] {
        let image = frame("", scale)?;
        for y in 0..image.height as usize {
            for x in 0..(20.0 * scale) as usize {
                let i = (y * image.width as usize + x) * 4;
                assert_eq!(&image.rgba[i..i + 4], &[255, 255, 255, 255]);
            }
        }
    }
    Ok(())
}
#[test]
fn overline_and_strike_waves_keep_their_distinct_vertical_positions() -> Result<(), String> {
    let bounds = |image: &Rendered| -> Option<(usize, usize)> {
        let mut ys = Vec::new();
        for (y, row) in image
            .rgba
            .chunks_exact(image.width as usize * 4)
            .enumerate()
        {
            if row.as_chunks::<4>().0.iter().any(|p| p[1] < 255) {
                ys.push(y);
            }
        }
        Some((*ys.first()?, *ys.last()?))
    };
    let under = frame("", 1.0)?;
    let over = frame("text-decoration-line:overline", 1.0)?;
    let strike = frame("text-decoration-line:line-through", 1.0)?;
    let under = bounds(&under).ok_or("missing underline")?;
    let over = bounds(&over).ok_or("missing overline")?;
    let strike = bounds(&strike).ok_or("missing strike")?;
    assert!(
        over.1 < strike.0 && strike.1 < under.0,
        "{over:?} {strike:?} {under:?}"
    );
    Ok(())
}
