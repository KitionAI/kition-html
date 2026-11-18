use kition_html_engine::{RenderOptions, Resources, render};

#[test]
fn flex_text_minimum_widths_keep_subpixel_precision_at_each_dpr() -> Result<(), String> {
    for scale in [1.0, 1.5, 2.0] {
        let mut resources = Resources::new();
        resources.insert(
            "/assets/Inter.ttf".into(),
            include_bytes!("../../../conformance/assets/Inter-Variable.ttf").to_vec(),
        );
        let image = render(
            "<!doctype html><style>body{margin:0;font:16px/24px Inter}section{display:flex;gap:16px;width:360px}i{display:block;padding:12px;border:2px solid white}</style><main id=stage><section><i>A</i><i>B</i><i>C</i><i>D</i><i>E</i><i>F</i></section></main>",
            resources,
            &RenderOptions {
                width: 400,
                height: 80,
                scale,
                ..Default::default()
            },
        )?;
        let items: Vec<_> = image.geometry.iter().filter(|g| g["tag"] == "I").collect();
        assert_eq!(items.len(), 6);
        // Chrome measurements with the same fixed Inter face, 16px optical size,
        // padding and border. Allow one layout unit for shaping-rounding differences.
        let reference = [38.984375, 38.453125, 39.6875, 39.5, 37.59375, 37.390625];
        let mut end = 0.0;
        for (item, expected) in items.iter().zip(reference) {
            let width = item["width"].as_f64().ok_or("missing width")?;
            let x = item["x"].as_f64().ok_or("missing x")?;
            assert!(
                (width - expected).abs() <= 1.0 / 64.0,
                "DPR {scale}: {width} vs {expected}"
            );
            end = x + width;
        }
        assert!(
            (end - 311.609375).abs() <= 3.0 / 64.0,
            "DPR {scale}: end {end}"
        );
    }
    Ok(())
}
