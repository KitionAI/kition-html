use kition_html_engine::{RenderOptions, Rendered, Resources, render};
fn frame(css: &str, body: &str, scale: f64) -> Result<Rendered, String> {
    render(
        &format!(
            "<!doctype html><style>body{{margin:0}}.parent,.sub{{display:grid}}.cell{{min-height:20px;background:#5645d4}}{css}</style><main id=stage>{body}</main>"
        ),
        Resources::new(),
        &RenderOptions {
            width: 640,
            height: 480,
            scale,
            ..Default::default()
        },
    )
}
fn value(image: &Rendered, id: &str, key: &str) -> Result<f64, String> {
    image
        .geometry
        .iter()
        .find(|n| n["id"] == id)
        .and_then(|n| n[key].as_f64())
        .ok_or_else(|| format!("missing {id}.{key}"))
}
fn cells() -> &'static str {
    "<i class=cell id=a></i><i class=cell id=b></i><i class=cell id=c></i>"
}
#[test]
fn subgrid_columns_inherit_resolved_fractions_and_gutters_at_each_dpr() -> Result<(), String> {
    for scale in [1.0, 1.5, 2.0] {
        let image = frame(
            ".parent{width:420px;grid-template-columns:80px 1fr 120px;gap:12px}.sub{grid-column:span 3;grid-template-columns:subgrid}",
            &format!("<div class=parent><div class=sub>{}</div></div>", cells()),
            scale,
        )?;
        for (id, x, width) in [("a", 0.0, 80.0), ("b", 92.0, 196.0), ("c", 300.0, 120.0)] {
            assert_eq!(value(&image, id, "x")?, x);
            assert_eq!(value(&image, id, "width")?, width);
        }
        assert!(image.diagnostics.is_empty(), "{:?}", image.diagnostics);
    }
    Ok(())
}
#[test]
fn explicit_subgrid_gaps_share_the_parent_gutter_midpoints() -> Result<(), String> {
    let image = frame(
        ".parent{width:280px;grid-template-columns:60px 100px 80px;gap:20px}.sub{grid-column:span 3;grid-template-columns:subgrid;gap:0}",
        &format!("<div class=parent><div class=sub>{}</div></div>", cells()),
        1.0,
    )?;
    for (id, x, width) in [("a", 0.0, 70.0), ("b", 70.0, 120.0), ("c", 190.0, 90.0)] {
        assert_eq!(value(&image, id, "x")?, x);
        assert_eq!(value(&image, id, "width")?, width);
    }
    Ok(())
}
#[test]
fn subgrid_edge_tracks_account_for_padding_and_borders() -> Result<(), String> {
    let image = frame(
        ".parent{width:260px;grid-template-columns:60px 100px 80px;gap:10px}.sub{grid-column:span 3;grid-template-columns:subgrid;padding:10px;border:2px solid black}",
        &format!("<div class=parent><div class=sub>{}</div></div>", cells()),
        1.0,
    )?;
    for (id, x, width) in [("a", 12.0, 48.0), ("b", 70.0, 100.0), ("c", 180.0, 68.0)] {
        assert_eq!(value(&image, id, "x")?, x);
        assert_eq!(value(&image, id, "width")?, width);
    }
    Ok(())
}
#[test]
fn inherited_and_local_line_names_resolve_within_a_partial_span() -> Result<(), String> {
    for name in ["right", "local-middle"] {
        let image = frame(
            &format!(
                ".parent{{width:280px;grid-template-columns:[left]60px[center]120px[right]80px[end];gap:10px}}.sub{{grid-column:center/end;grid-template-columns:subgrid [local-start][local-middle][local-end]}}.cell{{grid-column:{name}/span 1}}"
            ),
            "<div class=parent><div class=sub><i class=cell id=a></i></div></div>",
            1.0,
        )?;
        assert_eq!(value(&image, "a", "x")?, 200.0);
        assert_eq!(value(&image, "a", "width")?, 80.0);
    }
    Ok(())
}
#[test]
fn both_subgrid_axes_inherit_tracks_and_names() -> Result<(), String> {
    let image = frame(
        ".parent{width:210px;grid-template-columns:80px 120px;grid-template-rows:40px 60px;gap:10px}.sub{grid-area:1/1/3/3;grid-template-columns:subgrid;grid-template-rows:subgrid}.cell{grid-area:2/2}",
        "<div class=parent><div class=sub><i class=cell id=a></i></div></div>",
        1.0,
    )?;
    for (key, expected) in [("x", 90.0), ("y", 50.0), ("width", 120.0), ("height", 60.0)] {
        assert_eq!(value(&image, "a", key)?, expected);
    }
    Ok(())
}
#[test]
fn nested_subgrids_settle_after_the_parent_tracks_change() -> Result<(), String> {
    let image = frame(
        ".parent{container:outer/inline-size;width:420px;grid-template-columns:80px 1fr 120px;gap:12px}.sub{grid-column:span 3;grid-template-columns:subgrid}.inner{display:grid;grid-column:2/4;grid-template-columns:subgrid}.cell{grid-column:2}",
        "<div class=parent><div class=sub><div class=inner><i class=cell id=a></i></div></div></div>",
        1.0,
    )?;
    assert_eq!(value(&image, "a", "x")?, 300.0);
    assert_eq!(value(&image, "a", "width")?, 120.0);
    assert!(image.diagnostics.is_empty(), "{:?}", image.diagnostics);
    Ok(())
}
#[test]
fn opposite_inline_directions_reverse_inherited_columns() -> Result<(), String> {
    for (parent, child, expected) in [
        ("ltr", "rtl", [180.0, 70.0, 0.0]),
        ("rtl", "ltr", [0.0, 90.0, 200.0]),
        ("rtl", "rtl", [200.0, 90.0, 0.0]),
    ] {
        let image = frame(
            &format!(
                ".parent{{direction:{parent};width:260px;grid-template-columns:60px 100px 80px;gap:10px}}.sub{{direction:{child};grid-column:span 3;grid-template-columns:subgrid}}"
            ),
            &format!("<div class=parent><div class=sub>{}</div></div>", cells()),
            1.0,
        )?;
        for (id, x) in ["a", "b", "c"].into_iter().zip(expected) {
            assert_eq!(value(&image, id, "x")?, x, "{parent}/{child}/{id}");
        }
    }
    Ok(())
}

#[test]
fn container_query_restyling_discards_obsolete_subgrid_tracks() -> Result<(), String> {
    let image = frame(
        ".parent{container:outer/inline-size;width:280px;grid-template-columns:60px 100px 80px;gap:20px}.sub{grid-column:span 3;grid-template-columns:subgrid}@container outer (width > 200px){.sub{grid-template-columns:50px 50px}}",
        &format!("<div class=parent><div class=sub>{}</div></div>", cells()),
        1.0,
    )?;
    assert_eq!(value(&image, "a", "width")?, 50.0);
    assert_eq!(value(&image, "b", "width")?, 50.0);
    assert_eq!(value(&image, "c", "x")?, 0.0);
    assert!(image.diagnostics.is_empty(), "{:?}", image.diagnostics);
    Ok(())
}

#[test]
fn a_parent_that_stops_being_a_grid_drops_its_old_track_data() -> Result<(), String> {
    let image = frame(
        ".scope{container:outer/inline-size;width:320px}.parent{width:280px;grid-template-columns:60px 100px 80px;gap:20px}.sub{grid-column:span 3;grid-template-columns:subgrid}@container outer (width > 200px){.parent{display:block}}",
        &format!(
            "<div class=scope><div class=parent><div class=sub>{}</div></div></div>",
            cells()
        ),
        1.0,
    )?;
    for id in ["a", "b", "c"] {
        assert_eq!(value(&image, id, "width")?, 280.0);
        assert_eq!(value(&image, id, "x")?, 0.0);
    }
    assert!(image.diagnostics.is_empty(), "{:?}", image.diagnostics);
    Ok(())
}
