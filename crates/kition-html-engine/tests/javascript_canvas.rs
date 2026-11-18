use blitz_dom::DocumentConfig;
use blitz_vibey_script::ScriptDocument;
use kition_html_engine::{RenderOptions, Resources, Session, render};

fn check(script: &str) {
    let mut doc = ScriptDocument::from_html(
        "<!doctype html><canvas id=canvas width=32 height=32></canvas>",
        DocumentConfig::default(),
    )
    .without_timer_thread();
    doc.eval(&format!("function check(value,message){{if(!value)throw Error(message)}}const canvas=document.getElementById('canvas'),ctx=canvas.getContext('2d');{script}"));
    let errors = doc.take_js_errors();
    assert!(errors.is_empty(), "{errors:?}");
}

#[test]
fn drawing_and_readback_share_a_persistent_engine_bitmap() {
    check(
        r#"
        check(ctx===canvas.getContext('2d')&&ctx.canvas===canvas,'context identity');
        check(canvas.getContext('webgl')===null,'unsupported context');
        ctx.fillStyle='#5645d4';ctx.fillRect(0,0,32,32);
        check(ctx.getImageData(0,0,1,1).data.join(',')==='86,69,212,255','read filled pixel');
        ctx.clearRect(4,4,8,8);
        check(ctx.getImageData(5,5,1,1).data.join(',')==='0,0,0,0','transparent clearing');
        check(ctx.getImageData(0,0,1,1).data[3]===255,'surrounding pixels retained');
        check(ctx.getImageData(-1,-1,1,1).data[3]===0,'outside bitmap');
        ctx.fillStyle='not-a-color';check(ctx.fillStyle==='#5645d4','invalid paint ignored');
    "#,
    );
}

#[test]
fn resize_resets_context_state_and_bitmap_even_when_size_is_unchanged() {
    check(
        r#"
        ctx.fillStyle='red';ctx.fillRect(0,0,32,32);ctx.save();ctx.translate(8,8);ctx.globalAlpha=.5;
        canvas.width=32;check(ctx.fillStyle==='#000000'&&ctx.globalAlpha===1,'state reset');
        check(ctx.getTransform().e===0&&ctx.getImageData(0,0,1,1).data[3]===0,'bitmap and transform reset');
        ctx.restore();check(ctx.fillStyle==='#000000','save stack reset');
        ctx.fillStyle='blue';ctx.fillRect(0,0,2,2);
        const clone=canvas.cloneNode();check(clone.getContext('2d').getImageData(0,0,1,1).data[3]===0,'clone has empty bitmap');
    "#,
    );
}

#[test]
fn pixel_writes_ignore_transforms_alpha_and_clip() {
    check(
        r#"
        const image=new ImageData(new Uint8ClampedArray([86,69,212,255,229,72,106,255]),2,1);
        ctx.scale(2,2);ctx.globalAlpha=0;ctx.beginPath();ctx.rect(20,20,2,2);ctx.clip();
        ctx.putImageData(image,4,4);
        check(ctx.getImageData(4,4,2,1).data.join(',')===image.data.join(','),'direct pixel copy');
        ctx.putImageData(image,0,0,1,0,1,1);
        check(ctx.getImageData(0,0,1,1).data[3]===0&&ctx.getImageData(1,0,1,1).data[0]===229,'dirty rectangle');
        check(ctx.createImageData(image).data.every(v=>v===0),'blank image data');
    "#,
    );
}

#[test]
fn paths_keep_the_transform_at_creation_and_restore_preserves_current_path() {
    check(
        r#"
        ctx.beginPath();ctx.rect(0,0,4,4);ctx.save();ctx.translate(8,0);ctx.rect(0,0,4,4);ctx.restore();
        ctx.fillStyle='#5645d4';ctx.fill();
        for(const x of [0,8])check(ctx.getImageData(x,0,1,1).data.join(',')==='86,69,212,255','transformed path');
        check(ctx.getImageData(5,0,1,1).data[3]===0,'path gap');
        ctx.lineWidth=-1;check(ctx.lineWidth===1,'invalid width');ctx.globalAlpha=2;check(ctx.globalAlpha===1,'invalid alpha');
        ctx.setLineDash([2,3,4]);check(ctx.getLineDash().join(',')==='2,3,4,2,3,4','odd dash pattern');
    "#,
    );
}

#[test]
fn alpha_and_compositing_keep_straight_pixel_readback() {
    check(
        r#"
        ctx.fillStyle='rgba(86,69,212,0.5)';ctx.fillRect(0,0,32,32);
        const p=ctx.getImageData(0,0,1,1).data;check(Math.abs(p[0]-86)<=1&&Math.abs(p[1]-69)<=1&&Math.abs(p[2]-212)<=1&&Math.abs(p[3]-128)<=1,p.join(','));
        ctx.globalCompositeOperation='destination-out';ctx.globalAlpha=1;ctx.fillStyle='black';ctx.fillRect(0,0,8,8);
        check(ctx.getImageData(0,0,1,1).data[3]===0,'destination out');
        check(ctx.getImageData(16,16,1,1).data[3]>0,'outside composite');
    "#,
    );
}

#[test]
fn canvas_rendering_persists_across_frames_without_changing_source() -> Result<(), String> {
    let html = "<!doctype html><style>body{margin:0}canvas{display:block}</style><canvas id=canvas width=32 height=32></canvas><script>const ctx=document.getElementById('canvas').getContext('2d');ctx.fillStyle='#5645d4';ctx.fillRect(0,0,32,32);document.body.onclick=()=>{ctx.fillStyle='#e5486a';ctx.fillRect(0,0,16,16)}</script>";
    let options = RenderOptions {
        width: 64,
        height: 64,
        ..Default::default()
    };
    let image = render(html, Resources::new(), &options)?;
    assert!(image.diagnostics.is_empty(), "{:?}", image.diagnostics);
    assert_eq!(&image.rgba[..4], &[86, 69, 212, 255]);
    let mut session = Session::new(html, Resources::new(), &options)?;
    session.frame(1.0)?;
    session.click(4.0, 4.0)?;
    assert_eq!(&session.frame(1.0)?.rgba[..4], &[229, 72, 106, 255]);
    assert_eq!(&session.frame(2.0)?.rgba[..4], &[229, 72, 106, 255]);
    Ok(())
}

#[test]
fn canvas_rejects_unbounded_work_without_allocating_a_bitmap() {
    check(
        r#"
        let rejected=false;try{ctx.arc(0,0,1e300,0,Math.PI*2);ctx.fill()}catch(error){rejected=error instanceof RangeError}
        check(rejected,'coordinate bound');ctx.beginPath();
        canvas.width=100000;rejected=false;try{ctx.fillRect(0,0,1,1)}catch(error){rejected=error instanceof RangeError}
        check(rejected,'bitmap bound');canvas.width=32;ctx.fillRect(0,0,1,1);
        check(ctx.getImageData(0,0,1,1).data[3]===255,'context remains usable');
    "#,
    );
}

#[test]
fn image_drawing_snapshots_sources_and_normalizes_crop_rectangles() {
    check(
        r#"
        const source=document.createElement('canvas');source.width=4;source.height=2;
        const paint=source.getContext('2d');paint.fillStyle='red';paint.fillRect(0,0,2,2);paint.fillStyle='blue';paint.fillRect(2,0,2,2);
        ctx.imageSmoothingEnabled=false;ctx.drawImage(source,0,0,16,8);
        check(ctx.getImageData(2,2,1,1).data.join(',')==='255,0,0,255','left source');
        check(ctx.getImageData(12,2,1,1).data.join(',')==='0,0,255,255','right source');
        paint.clearRect(0,0,4,2);check(ctx.getImageData(2,2,1,1).data[3]===255,'snapshot retained');
        ctx.drawImage(canvas,0,0,16,8,8,8,16,8);
        check(ctx.getImageData(10,10,1,1).data.join(',')==='255,0,0,255','self-copy snapshot');
        ctx.drawImage(canvas,16,8,-16,-8,16,24,-16,-8);
        check(ctx.getImageData(2,18,1,1).data.join(',')==='255,0,0,255','negative rectangles do not flip');
        ctx.clearRect(0,24,32,8);ctx.drawImage(canvas,-8,0,32,8,0,24,32,8);
        check(ctx.getImageData(2,26,1,1).data[3]===0&&ctx.getImageData(10,26,1,1).data[0]===255,'out-of-bounds source clips destination');
    "#,
    );
}

#[test]
fn image_drawing_validates_sources_and_empty_rectangles_do_not_composite() {
    check(
        r#"
        let rejected=false;try{ctx.drawImage({},0,0)}catch(error){rejected=error instanceof TypeError}check(rejected,'invalid source');
        const source=document.createElement('canvas');source.width=0;
        rejected=false;try{ctx.drawImage(source,0,0)}catch(error){rejected=error.name==='InvalidStateError'}check(rejected,'empty source bitmap');
        source.width=4;source.height=4;
        ctx.fillStyle='red';ctx.fillRect(0,0,32,32);ctx.globalCompositeOperation='copy';
        ctx.drawImage(source,0,0,0,4,0,0,4,4);
        check(ctx.getImageData(0,0,1,1).data.join(',')==='255,0,0,255','zero source width is a no-op');
        ctx.drawImage(source,0,0,0,4);
        check(ctx.getImageData(0,0,1,1).data.join(',')==='255,0,0,255','zero destination width is a no-op');
    "#,
    );
}

#[test]
fn supplied_svg_images_draw_through_the_engine_canvas() -> Result<(), String> {
    let mut resources = Resources::new();
    resources.insert("/assets/source.svg".into(),br#"<svg xmlns="http://www.w3.org/2000/svg" width="4" height="4"><rect width="4" height="4" fill="red"/></svg>"#.to_vec());
    let result = render(
        "<!doctype html><style>body{margin:0}img{display:none}</style><img id=source src=/assets/source.svg><canvas id=canvas width=32 height=32></canvas><script>document.getElementById('canvas').getContext('2d').drawImage(document.getElementById('source'),0,0,32,32)</script>",
        resources,
        &RenderOptions {
            width: 64,
            height: 64,
            ..Default::default()
        },
    )?;
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(&result.rgba[..4], &[255, 0, 0, 255]);
    Ok(())
}

#[test]
fn bilinear_image_sampling_matches_fixed_point_reference_weights() {
    check(
        r#"
        const source=document.createElement('canvas');source.width=2;source.height=1;
        const paint=source.getContext('2d');paint.fillStyle='#5645d4';paint.fillRect(0,0,1,1);paint.fillStyle='#e5486a';paint.fillRect(1,0,1,1);
        ctx.drawImage(source,0,0,6,3);
        check(ctx.getImageData(2,1,1,1).data.join(',')==='130,69,178,255','first filtered edge');
        check(ctx.getImageData(3,1,1,1).data.join(',')==='175,70,145,255','second filtered edge');
    "#,
    );
}

#[test]
fn retained_paths_copy_geometry_and_apply_current_drawing_transforms() {
    check(
        r#"
        const path=new Path2D();path.rect(0,0,8,8);const copy=new Path2D(path);path.rect(16,0,8,8);
        ctx.translate(4,4);ctx.fillStyle='red';ctx.fill(copy);
        check(ctx.getImageData(5,5,1,1).data[0]===255&&ctx.getImageData(21,5,1,1).data[3]===0,'copy is independent');
        const joined=new Path2D();joined.addPath(copy,new DOMMatrix().translate(12,12));ctx.fillStyle='blue';ctx.fill(joined);
        check(ctx.getImageData(17,17,1,1).data[2]===255,'addPath transform');
        const svg=new Path2D('M0 0h8v8h-8z');ctx.resetTransform();ctx.fillStyle='lime';ctx.fill(svg);
        check(ctx.getImageData(1,1,1,1).data[1]===255,'SVG path');
    "#,
    );
}

#[test]
fn retained_clipping_snapshots_paths_and_evenodd_has_holes() {
    check(
        r#"
        const path=new Path2D();path.rect(0,0,24,24);path.rect(8,8,8,8);
        ctx.clip(path,'evenodd');path.rect(0,0,32,32);ctx.fillRect(0,0,32,32);
        check(ctx.getImageData(2,2,1,1).data[3]===255,'clip interior');
        check(ctx.getImageData(12,12,1,1).data[3]===0,'clip hole');
        check(ctx.getImageData(28,28,1,1).data[3]===0,'path mutation does not alter clip');
    "#,
    );
}

#[test]
fn path_hit_testing_uses_canvas_coordinates_and_stroke_geometry() {
    check(
        r#"
        ctx.translate(8,8);ctx.beginPath();ctx.rect(0,0,16,16);
        check(ctx.isPointInPath(12,12)&&!ctx.isPointInPath(2,2),'current path transform');
        const path=new Path2D();path.rect(0,0,16,16);
        check(ctx.isPointInPath(path,12,12)&&!ctx.isPointInPath(path,2,2),'retained path transform');
        ctx.lineWidth=4;check(ctx.isPointInStroke(path,8,16)&&!ctx.isPointInStroke(path,16,16),'stroke geometry');
        check(ctx.isPointInPath(path,24,24),'boundary belongs to path');
        ctx.resetTransform();ctx.beginPath();ctx.moveTo(0,0);ctx.lineTo(16,0);ctx.lineTo(16,16);
        check(ctx.isPointInPath(12,4)&&!ctx.isPointInPath(4,12),'open subpath closes for hit testing');
        check(!ctx.isPointInPath(NaN,0),'nonfinite point');
    "#,
    );
}

#[test]
fn rounded_rectangles_validate_radii_and_preserve_winding() {
    check(
        r#"
        let rejected=false;try{ctx.roundRect(0,0,16,16,-1)}catch(error){rejected=error instanceof RangeError}check(rejected,'negative radius');
        rejected=false;try{ctx.roundRect(0,0,16,16,[])}catch(error){rejected=error instanceof RangeError}check(rejected,'empty radius sequence');
        ctx.rect(0,0,32,32);ctx.roundRect(24,8,-16,16,4);ctx.fill();
        check(ctx.getImageData(16,16,1,1).data[3]===0,'reverse winding makes a hole');
        check(ctx.getImageData(2,2,1,1).data[3]===255,'outer shape retained');
        const path=new Path2D();path.roundRect(0,0,32,32,{x:8,y:4});
        check(ctx.isPointInPath(path,16,16)&&!ctx.isPointInPath(path,0,0),'elliptical radii');
    "#,
    );
}

#[test]
fn half_opacity_paint_preserves_reference_color_on_a_colored_backdrop() -> Result<(), String> {
    let html = "<!doctype html><style>body{margin:0}canvas{display:block}</style><canvas id=canvas width=32 height=32></canvas><script>const ctx=document.getElementById('canvas').getContext('2d');ctx.fillStyle='#5645d4';ctx.fillRect(0,0,32,32);ctx.fillStyle='#e5486a';ctx.globalAlpha=.5;ctx.fillRect(0,0,32,32)</script>";
    let frame = render(
        html,
        Resources::new(),
        &RenderOptions {
            width: 64,
            height: 64,
            ..Default::default()
        },
    )?;
    assert!(frame.diagnostics.is_empty(), "{:?}", frame.diagnostics);
    let offset = (16 * 64 + 16) * 4;
    assert_eq!(&frame.rgba[offset..offset + 4], &[158, 70, 159, 255]);
    Ok(())
}
