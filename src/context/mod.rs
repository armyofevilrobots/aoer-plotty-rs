use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use geo_types::{coord, Coordinate, Geometry, LineString, MultiLineString, MultiPolygon, Point, Polygon};
use svg::Document;
use svg::node::element::SVG;
use crate::prelude::{Arrangement, OutlineFillStroke, OutlineStroke, SvgCreationError, ToSvg};
use crate::geo_types::hatch::{Hatch, HatchPattern, LineHatch};
use anyhow::{Result as AResult, Error as AError};
use crate::geo_types::buffer::Buffer;


#[derive(Debug)]
struct ContextError {
    msg: String,
}


impl Display for ContextError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Error for ContextError {}

struct Operation {
    content: Geometry<f64>,
    transformation: Option<nalgebra::Affine2<f64>>,
    stroke_color: String,
    outline_stroke: Option<f64>,
    fill_color: String,
    line_join: String,
    line_cap: String,
    pen_width: f64,
    hatch_angle: Option<f64>,
}

impl Operation {
    fn render_to_lines(&self) -> (MultiLineString<f64>, MultiLineString<f64>) {
        let (outlines, fills) = match &self.content {
            Geometry::MultiLineString(mls) =>
                (mls.clone(), MultiLineString::new(vec![])),
            Geometry::LineString(ls) =>
                (MultiLineString::new(vec![ls.clone()]),
                 MultiLineString::new(vec![])),
            Geometry::Polygon(poly) => {
                let mut strokes = MultiLineString::new(vec![]);
                let mut fills = MultiLineString::new(vec![]);
                // Push the exterior
                strokes.0.push(poly.exterior().clone());
                for interior in poly.interiors() {
                    strokes.0.push(interior.clone())
                }
                if self.hatch_angle != None {
                    let hatches = poly
                        .hatch(LineHatch {}, self.hatch_angle.unwrap(),
                               self.pen_width * 0.8, self.pen_width * 0.8)
                        .unwrap_or(MultiLineString::new(vec![]));
                    fills.0.append(&mut hatches.0.clone());
                }
                (strokes, fills)
            }
            _ => (MultiLineString::new(vec![]), MultiLineString::new(vec![]))
        };
        let outlines = match self.outline_stroke{
            Some(stroke) => outlines
                .outline_fill_stroke_with_hatch(stroke,
                                                self.pen_width,
                                                Box::new(LineHatch{}),
                                                self.hatch_angle.unwrap_or(45.0))
                .unwrap_or(outlines),
            None => outlines
        };
        (outlines, fills)
    }
}

pub struct Context {
    operations: Vec<Operation>,
    transformation: Option<nalgebra::Affine2<f64>>,
    stroke_color: String,
    outline_stroke: Option<f64>,
    fill_color: String,
    line_join: String,
    line_cap: String,
    pen_width: f64,
    hatch_angle: Option<f64>,
}


impl Context {
    pub fn new() -> Context {
        Context {
            operations: vec![],
            transformation: None,
            stroke_color: "black".to_string(),
            outline_stroke: None,
            fill_color: "black".to_string(),
            line_join: "round".to_string(),
            line_cap: "round".to_string(),
            pen_width: 0.5,
            hatch_angle: None,
        }
    }

    fn add_operation(&mut self, geometry: Geometry<f64>) {
        self.operations.push(Operation {
            content: geometry,
            transformation: self.transformation.clone(),
            stroke_color: self.stroke_color.clone(),
            outline_stroke: self.outline_stroke.clone(),
            fill_color: self.fill_color.clone(),
            line_join: self.line_join.clone(),
            line_cap: self.line_cap.clone(),
            pen_width: self.pen_width.clone(),
            hatch_angle: self.hatch_angle,
        });
    }

    pub fn rect(&mut self, x0: f64, y0: f64, x1: f64, y1: f64) -> &mut Self{
        self.add_operation(
            Geometry::Polygon(
                Polygon::<f64>::new(
                    LineString::new(vec![
                        coord! {x: x0, y: y0},
                        coord! {x: x1, y: y0},
                        coord! {x: x1, y: y1},
                        coord! {x: x0, y: y1},
                        coord! {x: x0, y: y0},
                    ]),
                    vec![])));
        self
    }

    /// Draws a polygon
    pub fn poly(&mut self, exterior: Vec<(f64, f64)>, interiors: Vec<Vec<(f64, f64)>>) -> &mut Self{
        self.add_operation(
            Geometry::Polygon(
                Polygon::<f64>::new(
                    LineString::new(
                        exterior
                            .iter()
                            .map(|(x,y)| coord!{x:*x, y:*y})
                            .collect()
                    ),
                    interiors
                        .iter()
                        .map(|interior|{
                            LineString::<f64>::new(interior
                                .iter()
                                .map(|(x,y)| coord!{x:*x, y:*y})
                                .collect::<Vec<Coordinate<f64>>>())
                        })
                        .collect()
                )));
        self
    }

    pub fn circle(&mut self, x0: f64, y0: f64, radius: f64) -> &mut Self {
        self.add_operation(
            Geometry::MultiPolygon(Geometry::Point(Point::new(x0, y0))
                .buffer(radius)
                .unwrap_or(MultiPolygon::new(vec![]))
            ));
        self
    }

    pub fn stroke(&mut self, color: &str) -> &mut Self{
        self.stroke_color = color.to_string();
        self
    }

    pub fn fill(&mut self, color: &str) -> &mut Self{
        self.fill_color = color.to_string();
        self
    }

    pub fn hatch(&mut self, angle: Option<f64>) -> &mut Self{
        self.hatch_angle = angle;
        self
    }

    pub fn pen(&mut self, width: f64) -> &mut Self{
        self.pen_width = width;
        self
    }

    pub fn outline(&mut self, stroke: Option<f64>) -> &mut Self{
        self.outline_stroke = stroke;
        self
    }

    pub fn matrix(&mut self, xform: Option<nalgebra::Affine2<f64>>) -> &mut Self{
        self.transformation = xform;
        self
    }

    pub fn to_svg(&self, arrangement: &Arrangement<f64>) -> AResult<Document> {
        let mut svg = arrangement
            .create_svg_document()
            .or(Err(ContextError { msg: "Failed to create raw svg doc".into() }))?;
        // self.operations.iter().for_each(move |op| {
        for op in &self.operations{
            let (stroke, fill) = op.render_to_lines();
            if !stroke.0.is_empty() {
                svg = svg.add(stroke.to_path(&arrangement)
                    .set("fill", "none")
                    .set("stroke", op.stroke_color.clone())
                    .set("stroke-width", op.pen_width)
                    .set("stroke-linejoin", op.line_join.clone())
                    .set("stroke-linecap", op.line_cap.clone())
                );
            }
            if !fill.0.is_empty() {
                svg = svg.add(fill.to_path(&arrangement)
                    .set("fill", "none")
                    .set("stroke", op.fill_color.clone())
                    .set("stroke-width", op.pen_width)
                    .set("stroke-linejoin", op.line_join.clone())
                    .set("stroke-linecap", op.line_cap.clone())
                );
            }
        };
        Ok(svg)
    }
}


#[cfg(test)]
mod test {
    use geo_types::Rect;
    use super::*;

    #[test]
    fn test_context_new() {
        let context = Context::new();
        assert_eq!(context.transformation, None);
    }

    #[test]
    fn test_minimal_rect(){
        let mut context = Context::new();
        context.stroke("red")
            .pen(0.8)
            .fill("blue")
            .hatch(Some(45.0))
            .rect(10.0, 10.0, 50.0, 50.0);
        let arrangement = Arrangement::FitCenterMargin(
            10.0,
            Rect::new(coord!{x: 0.0, y: 0.0}, coord!{x:100.0, y:100.0}),
            false);
        let svg = context.to_svg(&arrangement);
        println!("Context: {:?}", svg);
    }

    #[test]
    fn test_minimal_poly(){
        let mut context = Context::new();
        context.stroke("red");
        context.pen(0.8);
        context.fill("blue");
        context.hatch(Some(45.0));
        context.poly(vec![(10.0, 10.0), (50.0, 10.0), (25.0, 25.0)],
                     vec![]);
        let arrangement = Arrangement::FitCenterMargin(
            10.0,
            Rect::new(coord!{x: 0.0, y: 0.0}, coord!{x:100.0, y:100.0}),
            false);
        let svg = context.to_svg(&arrangement);
    }
}
