use geo_types::MultiLineString;
use aoer_plotty_rs::geo_types::nannou::NannouDrawer;
use nannou::prelude::*;
use nannou::color;
use nannou::lyon::lyon_tessellation::LineJoin;
use nannou::lyon::tessellation::LineCap;
use aoer_plotty_rs::context::Context as AOERCTX;
use aoer_plotty_rs::elements::CarlsonSmithTruchet;
use aoer_plotty_rs::prelude::LineHatch;

/// The Model contains just the loop count (number of frames) and the tlines (turtle lines)
/// MultiLineString that contains the gosper curve.
struct Model {
    loops: u32,
    tlines: MultiLineString<f64>,
}

/// Creates a new turtle, then a new gosper LSystem. Walks the Gosper path after expanding
/// the LSystem, and then spits out a multiline string which we use to populate the Model.
/// Also centers the resulting MultiLineString on the 0,0 point in the middle of the screen.
fn model(_app: &App) -> Model {
    let mut ctx = AOERCTX::new();
    let mut count = 0;

    ctx.pen(1.0)
        .pattern(LineHatch::gen());

    for (name, geo) in CarlsonSmithTruchet::full_set(false) {
        println!("Plotting {}", name);
        let yofs: f64 = -128.0f64 + 64.0f64 * <f64 as From<i32>>::from((count / 6) as i32);
        let xofs: f64 = -128.0f64 + 64.0f64 * <f64 as From<i32>>::from((count % 6) as i32);
        let tx =
            AOERCTX::translate_matrix(xofs, yofs) *
                AOERCTX::scale_matrix(64.0, 64.0);
        count += 1;
        ctx.transform(Some(&tx))
            .geometry(&geo);
    }
    count = 0;
    for (name, geo) in CarlsonSmithTruchet::full_set(true) {
        println!("Plotting inverse scale/2 {}", name);
        let yofs: f64 = -128.0f64 - 16.0f64 + 32.0f64 * <f64 as From<i32>>::from((count / 6) as i32);
        let xofs: f64 = -128.0f64 - 16.0f64 + (64.0f64 * 6.0f64) + 32.0f64 * <f64 as From<i32>>::from((count % 6) as i32);
        let tx =
            AOERCTX::translate_matrix(xofs, yofs) *
                AOERCTX::scale_matrix(32.0, 32.0);
        count += 1;
        ctx.transform(Some(&tx))
            .geometry(&geo);
    }


    println!("Flattening...");
    let layers = ctx.flatten().to_layers();
    println!("Done flattening");

    let mut outlines = vec![];
    let mut fills = vec![];
    let mut all_lines: MultiLineString<f64> = MultiLineString::new(vec![]);

    for layer in &layers {
        let (o, f) = layer.to_lines();
        outlines.extend(o);
        fills.extend(f);
    }

    all_lines.0.extend(outlines);
    all_lines.0.extend(fills);

    // We're done. Save it in the model.
    Model {
        loops: 0,
        tlines: all_lines,
    }
}

fn update(_app: &App, _model: &mut Model, _update: Update) {
    // Update the var used to spin the gosper
    _model.loops += 1;
}

fn view(_app: &App, _model: &Model, frame: Frame) {
    // Broilerplate Nannou
    // And slowly spin it
    let draw = _app.draw().rotate((_model.loops as f32) * PI/180.0);
    frame.clear(PURPLE);

    // Draw the turtle lines into the draw context
    _model.tlines.iter().for_each(|tline| {
        draw.polyline()
            .stroke_weight(1.0)
            .caps(LineCap::Round)
            .join(LineJoin::Round)
            .polyline_from_linestring(tline)
            .color(color::NAVY);
    });


    // Done. Put it on the screen
    draw.to_frame(_app, &frame).unwrap();
}

fn main() {
    // Basic Nannou setup.
    nannou::app(model)
        .update(update)
        .simple_window(view)
        .run();
}

