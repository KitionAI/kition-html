use kition_html_engine::{RenderOptions, Session, default_resources, render};

#[test]
fn changing_scale_rebuilds_transforms_overflow_and_device_rounded_borders() -> Result<(), String> {
    let options = RenderOptions {
        width: 400,
        height: 240,
        ..Default::default()
    };
    for css in [
        "#box{transform:translate(60px,40px)}",
        "#box{transform:rotate(20deg)}",
        "#box{transform:skew(15deg,8deg)}",
        "section{width:120px;height:80px;overflow:clip}#box{transform:translate(40px,40px)}",
        "#box{border:9px double #5645d4;background:white}",
        "section{border:1px solid #ccc;padding:16px}",
    ] {
        let html = format!(
            "<!doctype html><style>body{{margin:24px;font:16px/24px Inter}}#box{{width:160px;height:64px;background:#5645d4}}{css}</style><main id=stage><section><div id=box></div></section></main>"
        );
        let mut session = Session::new(&html, default_resources(), &options)?;
        for scale in [1., 1.5, 2., 1., 2., 1.5] {
            let actual = session.frame(scale)?;
            let expected = render(
                &html,
                default_resources(),
                &RenderOptions {
                    scale,
                    ..options.clone()
                },
            )?;
            assert_eq!(
                actual.geometry, expected.geometry,
                "geometry: {css} at {scale}"
            );
            assert!(actual.rgba == expected.rgba, "pixels: {css} at {scale}");
        }
    }
    Ok(())
}

#[test]
fn scale_changes_preserve_live_script_state_and_do_not_repeat_initialization() -> Result<(), String>
{
    let html = "<!doctype html><style>body{margin:0}#box{width:100px;height:80px;transform:translate(20px,20px);background:red}</style><div id=box></div><script>let clicks=0;document.body.onclick=()=>{document.getElementById('box').style.background=++clicks===1?'#5645d4':'#1f9d4c'}</script>";
    let mut session = Session::new(html, default_resources(), &RenderOptions::default())?;
    session.click(30., 30.)?;
    for scale in [2., 1.5, 1.] {
        let image = session.frame(scale)?;
        let (x, y) = ((30. * scale) as usize, (30. * scale) as usize);
        let index = (y * image.width as usize + x) * 4;
        assert_eq!(&image.rgba[index..index + 4], &[86, 69, 212, 255]);
    }
    session.click(30., 30.)?;
    let image = session.frame(1.)?;
    let index = (30 * image.width as usize + 30) * 4;
    assert_eq!(&image.rgba[index..index + 4], &[31, 157, 76, 255]);
    Ok(())
}
