use std::f64::consts::PI;
use std::path::Path;
use geo::centroid::Centroid;
use geo::rotate::RotatePoint;
use geo::translate::Translate;
use geo_types::{coord, Coordinate, LineString, MultiLineString, Polygon, MultiPolygon, Rect, point};
use nalgebra::{Affine2, Matrix3};
use nannou::prelude::PI_F64;
use rand::{random, Rng};
use wkt::types::Coord;
use aoer_plotty_rs::prelude::{Arrangement, Hatch, LineHatch, ToSvg};

fn draw(line_positions: Vec<f64>, size: f64, xc: f64, yc: f64, rotation: f64) -> MultiLineString<f64> {
    let mut lines: Vec<LineString<f64>> = vec![];
    for position in line_positions {
        let p1 = coord! {
            x: position*size,
            y: 0.0
        };
        let p2 = coord! {
            x: position*size,
            y: size
        };
        let line = LineString::new(vec![p1, p2])
            .rotate_around_point(
                rotation,
                point! {
                x: f64::from(size)/2.0,
                y: f64::from(size)/2.0})
            .translate(
                xc - f64::from(size) / 2.0,
                yc - f64::from(size) / 2.0);
        lines.push(line);
    } // i in 0..count
    MultiLineString::new(lines)
}

/// This is a rusty take on the excellent: https://generativeartistry.com/tutorials/un-deux-trois/
fn main() {
    let size = 224;
    let steps = 14;  // Matching the tute.
    let step = size / steps;
    let pen_width = 0.3;
    let stroke_mm = f64::from(step)/6.0;


    // Define our viewbox/canvas (in mm)
    let viewbox = Rect::new(
        coord! {
            x:0f64,
            y:0f64},
        coord! {
            x: f64::from(size),
            y: f64::from(size)});


    // let all_lines = draw(3, f64::from(step), 0.0, 0.0, 0.0);
    let mut lines_list: Vec<MultiLineString<f64>> = vec![];
    let mut line_positions: Vec<f64> = vec![];
    for yc in 0..steps{
        for xc in 0..steps{
            if yc < steps/3 {
                line_positions = vec![0.5]
            }else if yc < ((2*steps)/3) {
                line_positions = vec![0.2, 0.8]
            }else{
                line_positions = vec![0.1, 0.5, 0.9]
            }
            lines_list.push(draw(line_positions,
                                 f64::from(step),
                                 f64::from(xc*step),
                                 f64::from(yc*step),
                                 rand::random::<f64>()*180.0));
        }
    }
    let mut all_lines: MultiLineString<f64> = MultiLineString::new(vec![]);
    for line in lines_list{
        all_lines.0.append(&mut line.0.clone());
    }

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
            .set("stroke-width", stroke_mm)
            .set("stroke-linejoin", "round")
            .set("stroke-linecap", "round"));

    // Write it to the images folder, so we can use it as an example!
    // Write it out to /images/$THIS_EXAMPLE_FILE.svg
    let fname = Path::new(file!()).file_stem().unwrap().to_str().unwrap();
    svg::save(format!("images/{}.svg", fname), &svg).unwrap();
}
