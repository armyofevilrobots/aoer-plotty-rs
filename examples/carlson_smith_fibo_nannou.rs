use aoer_plotty_rs::context::Context;
// use aoer_plotty_rs::elements::CarlsonSmithTruchet;
use geo_types::coord; //, Geometry, MultiLineString};

#[cfg(feature = "nannou")]
use nannou::{
    App, Frame,
    prelude::{PURPLE, Update},
};
#[cfg(feature = "nannou")]
use nannou_egui::{self, Egui, egui};
use num_traits::Pow;
use std::f64::consts::PI;

/// Hashmap which points a color (as a string) to a pair of <outlines, fills>
//type Layers = HashMap<String, (MultiLineString<f64>, MultiLineString<f64>)>;

struct Settings {
    pub fibdepth: usize,
    pub scale: f64,
}

#[cfg(feature = "nannou")]
struct Model {
    egui: Egui,
    // tiles: HashMap<String, Rc<Geometry<f64>>>,
    // layers: Layers,
    pub settings: Settings,
}

#[cfg(feature = "nannou")]
impl Model {
    /// Generates my content
    fn generate(&mut self) {
        let /*mut*/ _ctx = Context::new();
        let mut fibval = 1;
        let mut lastfib = 0;
        let mut angle = 0.0f64;

        for _i in 0..self.settings.fibdepth {
            let scale = self.settings.scale * (<f64 as From<i32>>::from(fibval));
            // let translation = Context::translate_matrix(scale, scale) * Context::rotate_matrix(angle);
            let _origin = coord! {
                x: scale*(angle*(PI/180.0)).cos(),
                y: scale*(angle*(PI/180.0)).sin()
            };
            // let base_scale = 2.0f64 * (f64::from(fibval)/2.0).floor();
            let base_scale = 2.0f64.pow(<f64 as From<i32>>::from(fibval).log2().floor());
            println!(
                "FIB {} would have a base_scale of {} with a rootval of {} and logval of {}",
                fibval,
                base_scale,
                <f64 as From<i32>>::from(fibval).log2().floor(),
                f64::from(fibval).log2()
            );

            let tmp = fibval;
            fibval = fibval + lastfib;
            lastfib = tmp;
            angle += 90.0;
        }
    }
}

#[cfg(feature = "nannou")]
fn model(app: &App) -> Model {
    // Create window
    let window_id = app
        // .window_ids().first().unwrap().clone();
        .new_window()
        .view(view)
        .raw_event(raw_window_event)
        .build()
        .unwrap();
    let window = app.window(window_id).unwrap();
    let egui = Egui::from_window(&window);
    // We're done. Save it in the model.
    let mut model = Model {
        egui: egui,
        // tiles: CarlsonSmithTruchet::full_set(),
        // layers: Default::default(),
        settings: Settings {
            fibdepth: 8,
            scale: 8.0,
        },
    };
    model.generate();
    model
}

#[cfg(feature = "nannou")]
fn update(_app: &App, model: &mut Model, update: Update) {
    // Update the var used to spin the gosper
    let egui = &mut model.egui;
    egui.set_elapsed_time(update.since_start);
    let ctx = egui.begin_frame();
    egui::Window::new("Carlson Smith Fibo").show(&ctx, |_ui| {});
}

#[cfg(feature = "nannou")]
fn raw_window_event(_app: &App, model: &mut Model, event: &nannou::winit::event::WindowEvent) {
    // Let egui handle things like keyboard and mouse input.
    model.egui.handle_raw_event(event);
}

#[cfg(feature = "nannou")]
fn view(app: &App, model: &Model, frame: Frame) {
    // Broilerplate Nannou
    let draw = app.draw();
    frame.clear(PURPLE);

    // Done. Put it on the screen
    draw.to_frame(app, &frame).unwrap();

    // Don't forget the GUI
    model.egui.draw_to_frame(&frame).unwrap();
}

fn main() {
    // Basic Nannou setup.
    #[cfg(feature = "nannou")]
    nannou::app(model).update(update).run();
}
