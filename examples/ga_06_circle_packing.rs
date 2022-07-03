use std::path::Path;
use geo_types::{coord, MultiLineString, Rect, point, Geometry, Point};
use aoer_plotty_rs::geo_types::buffer::Buffer;
use aoer_plotty_rs::prelude::{Arrangement, ToSvg};
use rand::prelude::*;

/// This is a rusty take on the excellent: https://generativeartistry.com/tutorials/circle-packing/

/// This is a fast circle intersection algo Trait and struct. No reason to do full polygon
/// intersections like with geos. That's just wasteful
struct Circle {
    point: Point<f64>,
    radius: f64,
}

impl Circle {
    fn collides(&self, other: &Circle) -> bool {
        if ((self.point.x() - other.point.x()).powi(2) +
            (self.point.y() - other.point.y()).powi(2))
            .sqrt() <= self.radius + other.radius {
            return true;
        }
        false
    }
}


fn main() {
    let size = 224;
    let pen_width = 0.5;
    let total_circles = 1000;
    let circle_attempts = 1000;
    let max_radius = 100.0f64;
    let min_radius = 2.0f64;
    // We're using a static random generator here so that our SVG files
    // don't get regenerated every time we run the examples.
    let mut rng = SmallRng::seed_from_u64(12345);

    // Define our viewbox/canvas (in mm)
    let viewbox = Rect::new(
        coord! {
            x:0f64,
            y:0f64},
        coord! {
            x: f64::from(size),
            y: f64::from(size)});


    // let mut circles = MultiPolygon::<f64>::new(vec![]);
    let mut circles: Vec<Circle> = vec![];
    for _ic in 0..total_circles {
        for _attempt in 0..circle_attempts {
            let circle = Circle {
                point: point! {
                    x: rng.gen::<f64>()*f64::from(size),
                    y: rng.gen::<f64>()*f64::from(size),
                    },
                radius: rng.gen::<f64>() * (max_radius - min_radius) + min_radius,
            };
            if circle.point.x() + circle.radius > f64::from(size)
                || circle.point.x() - circle.radius < 0.0f64
                || circle.point.y() + circle.radius > f64::from(size)
                || circle.point.y() - circle.radius < 0.0f64 { continue; } // Outside bounds
            let mut collided = false;
            for other in &circles {
                if other.collides(&circle) {
                    collided = true;
                    break;
                }
            }
            if !collided {
                circles.push(circle);
                break;
            }
        }
    }

    let all_lines = MultiLineString::<f64>::new(
        circles
            .iter()
            .map(|c| Geometry::<f64>::from(c.point)
                .buffer(c.radius)
                .unwrap()
                .0
                .pop()
                .unwrap()
                .exterior()
                .clone())
            .collect());

    // The arrangement chooses the way we "arrange" the SVG on the page.
    // In this case, fit it, center it, and then DON'T flip the coordinate
    // system upside down (SVG has top left as 0,0, whereas mathematically
    // 0,0 is the center, and on a CNC machine, 0,0 is bottom left... usually).
    let arrangement = Arrangement::FitCenterMargin(10.0, viewbox, false);

    // Use a shortcut to create an SVG scaffold from our arrangement.
    let svg = arrangement.create_svg_document().unwrap()
        .add(all_lines.to_path(&arrangement)
            .set("fill", "none")
            .set("stroke", "red")
            .set("stroke-width", pen_width)
            .set("stroke-linejoin", "round")
            .set("stroke-linecap", "round"));

    // Write it to the images folder, so we can use it as an example!
    // Write it out to /images/$THIS_EXAMPLE_FILE.svg
    let fname = Path::new(file!()).file_stem().unwrap().to_str().unwrap();
    svg::save(format!("images/{}.svg", fname), &svg).unwrap();
}
