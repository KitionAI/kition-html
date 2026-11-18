use kition_html_engine::{RenderOptions, Rendered, Session, default_resources, render};

fn source(css: &str, body: &str) -> String {
    format!(
        "<!doctype html><style>body{{margin:0;font:16px/24px Inter}}select{{font:14px Inter}}{css}</style><main id=stage>{body}</main>"
    )
}

fn options() -> RenderOptions {
    RenderOptions {
        width: 320,
        height: 120,
        ..Default::default()
    }
}

fn geometry(image: &Rendered, id: &str) -> [f64; 4] {
    let node = image.geometry.iter().find(|node| node["id"] == id);
    ["x", "y", "width", "height"]
        .map(|key| node.and_then(|node| node[key].as_f64()).unwrap_or(f64::NAN))
}

fn frame(css: &str, body: &str) -> Result<Rendered, String> {
    render(&source(css, body), default_resources(), &options())
}

#[test]
fn native_checkable_baselines_ignore_bottom_margins() -> Result<(), String> {
    for kind in ["checkbox", "radio"] {
        for (css, y) in [
            ("", 5.),
            ("body{line-height:40px}", 13.),
            ("body{line-height:12px}", 3.),
            ("input{margin:8px 12px 6px 4px}", 8.),
            ("input{margin-bottom:-4px}", 5.),
            ("input{margin-bottom:6px}", 5.),
        ] {
            let image = frame(css, &format!("<input id=control type={kind} checked>"))?;
            let rect = geometry(&image, "control");
            assert_eq!(&rect[1..], &[y, 13., 13.], "{kind}: {css}");
        }
    }
    Ok(())
}

#[test]
fn appearance_none_removes_intrinsic_size_but_keeps_author_dimensions() -> Result<(), String> {
    for kind in ["checkbox", "radio"] {
        let body = format!("<input id=control type={kind} checked>");
        let small = frame("input{appearance:none;border:2px solid #5645d4}", &body)?;
        assert_eq!(&geometry(&small, "control")[2..], &[4., 4.]);
        let large = frame(
            "input{appearance:none;width:24px;height:20px;border:2px solid #5645d4}",
            &body,
        )?;
        assert_eq!(&geometry(&large, "control")[2..], &[24., 20.]);
    }
    Ok(())
}

#[test]
fn optgroups_reserve_shaped_spaces_without_measuring_the_group_label() -> Result<(), String> {
    for (size, plain_width, grouped_width) in [(10, 57., 68.), (14, 70., 86.), (24, 101., 126.)] {
        let css = format!("select{{font-size:{size}px}}");
        let choices = "<option>English</option><option>French</option>";
        let plain = frame(&css, &format!("<select id=control>{choices}</select>"))?;
        assert_eq!(geometry(&plain, "control")[2], plain_width);
        for label in ["A", "An arbitrarily long group heading"] {
            let grouped = frame(
                &css,
                &format!(
                    "<select id=control><optgroup label='{label}'>{choices}</optgroup></select>"
                ),
            )?;
            assert_eq!(
                geometry(&grouped, "control")[2],
                grouped_width,
                "{size}: {label}"
            );
        }
    }
    let none = frame(
        "select{appearance:none}",
        "<select id=control><optgroup label=A><option>English</option></optgroup></select>",
    )?;
    assert_eq!(geometry(&none, "control")[2], 66.);
    Ok(())
}

#[test]
fn grouped_menu_width_uses_normal_weight_while_plain_options_inherit() -> Result<(), String> {
    for (choices, width) in [
        ("<option>English</option>", 73.),
        ("<optgroup label=A><option>English</option></optgroup>", 86.),
    ] {
        let image = frame(
            "select{font-weight:700}",
            &format!("<select id=control>{choices}</select>"),
        )?;
        assert_eq!(geometry(&image, "control")[2], width);
    }
    for css in ["option{font-weight:700}", "option{font-size:24px}"] {
        let image = frame(
            css,
            "<select id=control><optgroup label=A><option>English</option></optgroup></select>",
        )?;
        assert_eq!(geometry(&image, "control")[2], 86.);
    }
    Ok(())
}

#[test]
fn regrouping_unchanged_option_labels_rebuilds_intrinsic_width() -> Result<(), String> {
    let html = source(
        "",
        "<select id=control><option>English</option><option>French</option></select>",
    ) + "<script>const choice=document.getElementById('control'),group=document.createElement('optgroup');setTimeout(()=>{while(choice.firstChild)group.appendChild(choice.firstChild);choice.appendChild(group)},100);setTimeout(()=>{while(group.firstChild)choice.insertBefore(group.firstChild,group);group.remove()},200)</script>";
    let mut session = Session::new(&html, default_resources(), &options())?;
    assert_eq!(geometry(&session.frame(1.)?, "control")[2], 70.);
    session.advance(100)?;
    assert_eq!(geometry(&session.frame(1.)?, "control")[2], 86.);
    session.advance(100)?;
    assert_eq!(geometry(&session.frame(1.)?, "control")[2], 70.);
    Ok(())
}

#[test]
fn narrow_native_labels_can_extend_beneath_the_arrow() -> Result<(), String> {
    let css = "select{display:block;width:48px}";
    let filled = frame(css, "<select><option>MMMM</option></select>")?;
    let blank = frame(css, "<select><option></option></select>")?;
    let changed = (4..14)
        .flat_map(|y| (33..46).map(move |x| (x, y)))
        .filter(|&(x, y)| {
            let offset = (y * filled.width as usize + x) * 4;
            filled.rgba[offset..offset + 4] != blank.rgba[offset..offset + 4]
        })
        .count();
    assert!(
        changed > 10,
        "label must not stop at the reserved arrow width"
    );
    Ok(())
}

#[test]
fn checkable_and_grouped_select_layout_survive_scale_changes() -> Result<(), String> {
    let html = source(
        "",
        "<input type=checkbox checked><select><optgroup label=A><option>English</option></optgroup></select>",
    );
    let mut session = Session::new(&html, default_resources(), &options())?;
    for scale in [1., 1.5, 2., 1.] {
        let actual = session.frame(scale)?;
        let expected = render(
            &html,
            default_resources(),
            &RenderOptions { scale, ..options() },
        )?;
        assert_eq!(actual.rgba, expected.rgba, "scale {scale}");
    }
    Ok(())
}
