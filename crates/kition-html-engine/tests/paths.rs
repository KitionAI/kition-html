use kition_html_engine::{RenderOptions, Resources, render};

#[test]
fn small_circle_coverage_tracks_its_geometric_area_at_multiple_dprs() -> Result<(), String> {
    for scale in [1.0, 1.5, 2.0] {
        let image = render(
            "<!doctype html><style>body{margin:0}div{position:absolute;left:20px;top:20px;width:6px;height:6px;background:black;border-radius:50%}</style><div></div>",
            Resources::new(),
            &RenderOptions {
                width: 48,
                height: 48,
                scale,
                ..Default::default()
            },
        )?;
        let area: f64 = image
            .rgba
            .as_chunks::<4>()
            .0
            .iter()
            .map(|p| (255.0 - f64::from(p[0])) / 255.0)
            .sum::<f64>()
            / (scale * scale);
        assert!(
            (area - std::f64::consts::PI * 9.0).abs() < 0.5,
            "scale {scale}: {area}"
        );
    }
    Ok(())
}

#[test]
fn refined_stroke_preserves_round_caps_and_transformed_area() -> Result<(), String> {
    for scale in [1.0, 2.0] {
        let mut resources = Resources::new();
        resources.insert("/assets/line.svg".into(),br#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="40"><path d="M25 20H75" stroke="black" stroke-width="10" stroke-linecap="round"/></svg>"#.to_vec());
        let image = render(
            "<!doctype html><style>body{margin:0}img{display:block}</style><img src=/assets/line.svg>",
            resources,
            &RenderOptions {
                width: 100,
                height: 40,
                scale,
                ..Default::default()
            },
        )?;
        let area: f64 = image
            .rgba
            .as_chunks::<4>()
            .0
            .iter()
            .map(|p| (255.0 - f64::from(p[0])) / 255.0)
            .sum::<f64>()
            / (scale * scale);
        let exact = 500.0 + std::f64::consts::PI * 25.0;
        assert!(
            (area - exact).abs() < 2.0,
            "scale {scale}: {area} != {exact}"
        );
        let at = |x: usize, y: usize| {
            image.rgba[((y as f64 * scale) as usize * image.width as usize
                + (x as f64 * scale) as usize)
                * 4]
        };
        assert_eq!(at(19, 20), 255);
        assert_eq!(at(21, 20), 0);
        assert_eq!(at(78, 20), 0);
        assert_eq!(at(81, 20), 255);
    }
    Ok(())
}
