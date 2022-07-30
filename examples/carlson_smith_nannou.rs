use std::borrow::{Borrow, BorrowMut};
use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::{Arc, Mutex};
use std::thread::{JoinHandle, spawn, Thread};
use geo_types::{MultiLineString, MultiPolygon};
use nalgebra::DimAdd;
use aoer_plotty_rs::geo_types::nannou::NannouDrawer;
use nannou::prelude::*;
use nannou::color;
use nannou::lyon::lyon_tessellation::LineJoin;
use nannou::lyon::tessellation::LineCap;
use nannou_egui::{self, egui, Egui, Input};
use aoer_plotty_rs::context::Context as AOERCTX;
use aoer_plotty_rs::context::typography::TextAlignment::Center;
use aoer_plotty_rs::context::typography::Typography;
use aoer_plotty_rs::elements::CarlsonSmithTruchet;
use aoer_plotty_rs::prelude::LineHatch;


/// All the stuff we want in the egui component
#[derive(Clone, PartialEq)]
struct Settings {
    rotation: f32,
    color: Srgb<u8>,
    draft: bool
}

impl Settings {
    fn changed(&self, other: &Self) -> bool{
        self.draft != other.draft
    }
}

/// Hashmap which points a color (as a string) to a pair of <outlines, fills>
type Layers = HashMap<String, (MultiLineString<f64>, MultiLineString<f64>)>;

/// The Model contains just the loop count (number of frames) and the tlines (turtle lines)
/// MultiLineString that contains the gosper curve.
struct Model {
    loops: u32,
    layers: Layers,
    dirty: bool,
    egui: Egui,
    pub settings: Settings,
    generate: fn(&Settings) -> AOERCTX,
    on_change: fn(&fn(&Settings)->AOERCTX, &Settings) -> Layers,
}


/// Generates the actual content
fn generate(settings: &Settings) -> AOERCTX {
    let mut ctx = AOERCTX::new();
    let mut count = 0;

    ctx.pen(1.0)
        .pattern(LineHatch::gen());
    let mut typo = Typography::new();
    typo.size(4.5)
        .align(Center);


    for (name, geo) in CarlsonSmithTruchet::full_set(false) {
        println!("Plotting {}", name);
        let yofs: f64 = -256.0f64 + 128.0f64 * <f64 as From<i32>>::from((count / 6) as i32);
        let xofs: f64 = -512.0f64 + 96.0f64 * <f64 as From<i32>>::from((count % 6) as i32);
        println!("xy is {},{}", xofs, yofs);
        let tx =
            AOERCTX::translate_matrix(xofs, yofs) *
                AOERCTX::scale_matrix(64.0, 64.0);
        count += 1;
        ctx.transform(Some(&tx))
            .geometry(&geo);
        let tx =
            AOERCTX::translate_matrix(xofs, yofs) * AOERCTX::scale_matrix(1.0, -1.0);

        ctx.transform(Some(&tx))
            .typography(&name, 0.0,60.0, &typo);


    }
    count = 0;
    for (name, geo) in CarlsonSmithTruchet::full_set(true) {
        println!("Plotting inverse scale/2 {}", name);
        let yofs: f64 = -256.0f64 - 16.0f64 + 128.0f64 * <f64 as From<i32>>::from((count / 6) as i32);
        let xofs: f64 = -512.0f64  + (96.0f64 * 6.0f64) + 96.0f64 * <f64 as From<i32>>::from((count % 6) as i32);
        let tx =
            AOERCTX::translate_matrix(xofs, yofs) *
                AOERCTX::scale_matrix(32.0, 32.0);
        count += 1;
        ctx.transform(Some(&tx))
            .geometry(&geo);
        ctx.transform(Some(&tx))
            .geometry(&geo);
        let tx =
            AOERCTX::translate_matrix(xofs, yofs) * AOERCTX::scale_matrix(1.0, -1.0);

        ctx.transform(Some(&tx))
            .typography(&name, 0.0,40.0, &typo);

    }
    ctx

}

fn on_change(generate: &fn(&Settings)->AOERCTX, settings: &Settings) -> Layers {
    let ctx = (generate)(&settings.clone());
    println!("Flattening...");
    let layers = if settings.draft {
        ctx.flatten().to_layers()
    }else{
        ctx.to_layers()
    };
    println!("Done flattening");

    let mut out = Layers::new();
    for layer in &layers {
        let (strokes, fills) = layer.to_lines();
        if settings.draft{
            if out.contains_key(&layer.stroke()){
                let (mut orig_stroke, _orig_fill) = out.get_mut(&layer.stroke()).unwrap().clone();
                orig_stroke.0.append(&mut strokes.0.clone());
                out.insert(layer.stroke().clone(), (orig_stroke.clone(), MultiLineString::new(vec![])));
            }else{
                out.insert(layer.stroke().clone(), (strokes.clone(), MultiLineString::new(vec![])));
            }
        }else{
            if out.contains_key(&layer.stroke()) {
                let (mut orig_stroke, mut orig_fill) = out.get_mut(&layer.stroke()).unwrap().clone();
                orig_stroke.0.append(&mut strokes.0.clone());
                orig_fill.0.append(&mut fills.0.clone());
                out.insert(layer.stroke(), (orig_stroke.clone(), orig_fill.clone()));
            }else{
                out.insert(layer.stroke(), (strokes.clone(), fills.clone()));

            }
        }
    }
    out
}

/// Creates a new turtle, then a new gosper LSystem. Walks the Gosper path after expanding
/// the LSystem, and then spits out a multiline string which we use to populate the Model.
/// Also centers the resulting MultiLineString on the 0,0 point in the middle of the screen.
fn model(app: &App) -> Model {
    // Generate the egui stuffs
    // Create window
    let window_id = app
        .new_window()
        .view(view)
        .raw_event(raw_window_event)
        .build()
        .unwrap();
    let window = app.window(window_id).unwrap();

    let egui = Egui::from_window(&window);

    // We're done. Save it in the model.
    Model {
        loops: 0,
        layers: HashMap::new(),
        egui: egui,
        dirty: true,
        settings: Settings {
            rotation: 0.0,
            color: WHITE,
            draft: true
        },
        on_change,
        generate
    }
}

fn update(_app: &App, model: &mut Model, update: Update) {
    // Update the var used to spin the gosper
    model.loops += 1;

    let orig_settings = model.settings.clone();
    let settings = &mut model.settings;
    let egui = &mut model.egui;

    egui.set_elapsed_time(update.since_start);
    let ctx = egui.begin_frame();

    egui::Window::new("Carlson Smith Truchets").show(&ctx, |ui| {
        // Rotation slider
        ui.label("Rotation:");
        ui.add(egui::Slider::new(&mut settings.rotation, 0.0..=360.0));

        ui.label("Draft Mode:");
        ui.add(egui::Checkbox::new(&mut settings.draft, "draft"));

        // Random color button
        let clicked = ui.button("Random color").clicked();

        if clicked {
            settings.color = rgb(random(), random(), random());
        }

        if model.dirty {
            ui.label("Building");
            ui.add(egui::ProgressBar::new(<f32 as From<f32>>::from((model.loops % 15) as f32)/15.0));
            model.layers = (model.on_change)(&model.generate, settings);
            model.dirty = false;
        }






    });
    if model.settings.changed(&orig_settings){
        model.dirty = true
    }

}

fn raw_window_event(_app: &App, model: &mut Model, event: &nannou::winit::event::WindowEvent) {
    // Let egui handle things like keyboard and mouse input.
        model.egui.handle_raw_event(event);
}

fn view(app: &App, model: &Model, frame: Frame) {
    // Broilerplate Nannou
    // And slowly spin it
    let draw = app.draw().rotate(model.settings.rotation*PI/180.0);//(_model.loops as f32) * PI/180.0);
    frame.clear(PURPLE);

    for (color, (outlines, fills)) in &model.layers { //model.layers.clone().get_mut().expect("FOO") {
        for outline in outlines{
            draw.polyline()
                .stroke_weight(1.0)
                .caps(LineCap::Round)
                .join(LineJoin::Round)
                .polyline_from_linestring(outline)
                .color(model.settings.color);
        }
        if !model.settings.draft{
            for fill in fills{
                draw.polyline()
                    .stroke_weight(1.0)
                    .caps(LineCap::Round)
                    .join(LineJoin::Round)
                    .polyline_from_linestring(fill)
                    .color(model.settings.color);
            }
        }
    }


    // Done. Put it on the screen
    draw.to_frame(app, &frame).unwrap();
    model.egui.draw_to_frame(&frame).unwrap();
}

fn main() {
    // Basic Nannou setup.
    nannou::app(model)
        .update(update)
        .simple_window(view)
        .size(1280, 1024)
        .run();
}

