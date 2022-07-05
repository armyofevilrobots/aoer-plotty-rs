use std::error::Error;
use std::f64::consts::PI;
use std::fmt::{Debug, Display, Formatter};
use geo_types::{coord, Coordinate, Geometry, LineString, MultiLineString, MultiPolygon, Point, Polygon};
use svg::Document;
use svg::node::element::SVG;
use crate::prelude::{Arrangement, OutlineFillStroke, OutlineStroke, SvgCreationError, ToSvg};
use crate::geo_types::hatch::{Hatch, HatchPattern, LineHatch};
use anyhow::{Result as AResult, Error as AError};
use cubic_spline::{Points, SplineOpts};
use geo::map_coords::MapCoords;
use nalgebra::{Affine2, Matrix3, Point2 as NPoint2};
use svg::node::NodeClone;
use crate::geo_types::buffer::Buffer;
use crate::geo_types::clip::LineClip;
use crate::errors::ContextError;


#[derive(Clone)]
struct Operation {
    content: Geometry<f64>,
    transformation: Option<Affine2<f64>>,
    stroke_color: String,
    outline_stroke: Option<f64>,
    fill_color: String,
    line_join: String,
    line_cap: String,
    pen_width: f64,
    clip_previous: bool,
    hatch_angle: Option<f64>,
}



impl Operation {
    /// Helper function for converting polygons into sets of strings.
    fn poly2lines(poly: &Polygon<f64>, pen_width: f64, hatch_angle: Option<f64>) -> (MultiLineString<f64>, MultiLineString<f64>){
        let mut strokes = MultiLineString::new(vec![]);
        let mut fills = MultiLineString::new(vec![]);
        // Push the exterior
        strokes.0.push(poly.exterior().clone());
        for interior in poly.interiors() {
            strokes.0.push(interior.clone())
        }
        if hatch_angle != None {
            let hatches = poly
                .hatch(LineHatch {}, hatch_angle.unwrap(),
                       pen_width * 0.8, pen_width * 0.8)
                .unwrap_or(MultiLineString::new(vec![]));
            fills.0.append(&mut hatches.0.clone());
        }
        (strokes, fills)
    }

    /// Helper to transform geometry when we have an affine transform set.
    fn xform_coord((x,y): &(f64, f64), affine: &Affine2<f64>) -> (f64, f64){
        let out = affine * NPoint2::new(*x, *y);
        (out.x, out.y)
    }

    fn render_to_lines(&self) -> (MultiLineString<f64>, MultiLineString<f64>) {
        let (outlines, fills) = match &self.content {
            Geometry::MultiLineString(mls) =>
                (mls.clone(), MultiLineString::new(vec![])),
            Geometry::LineString(ls) =>
                (MultiLineString::new(vec![ls.clone()]),
                 MultiLineString::new(vec![])),
            Geometry::Polygon(poly) =>
                Self::poly2lines(poly, self.pen_width, self.hatch_angle),
            Geometry::MultiPolygon(polys) => {
                let mut strokes = MultiLineString::new(vec![]);
                let mut fills = MultiLineString::new(vec![]);
                for poly in polys{
                    let (new_strokes, new_fills) =
                        Self::poly2lines(poly, self.pen_width, self.hatch_angle);
                    strokes.0.append(&mut new_strokes.0.clone());
                    fills.0.append(&mut new_fills.0.clone());
                }
                (strokes, fills)
            }
            _ => (MultiLineString::new(vec![]), MultiLineString::new(vec![]))
        };
        let (outlines, fills) = match &self.transformation {
            Some(affine)=> {
                (outlines.map_coords(|xy| Self::xform_coord(xy, affine)),
                 fills.map_coords(|xy| Self::xform_coord(xy, affine)))
            },
            None => (outlines, fills)
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

#[derive(Clone)]
pub struct Context {
    operations: Vec<Operation>,
    transformation: Option<Affine2<f64>>,
    stroke_color: String,
    outline_stroke: Option<f64>,
    fill_color: String,
    line_join: String,
    line_cap: String,
    pen_width: f64,
    clip_previous: bool,
    hatch_angle: Option<f64>,
    stack: Vec<Context>,
}


impl Context {

    pub fn scale_matrix(sx: f64, sy: f64) -> Affine2<f64>{
        Affine2::from_matrix_unchecked(Matrix3::new(
            sx, 0.0, 0.0,
            0.0, sy, 0.0,
            0.0, 0.0, 1.0))
    }

    pub fn translate_matrix(tx: f64, ty: f64) -> Affine2<f64>{
        Affine2::from_matrix_unchecked(Matrix3::new(
            1.0, 0.0, tx,
            0.0, 1.0, ty,
            0.0, 0.0, 1.0))
    }

    /// Angle is in degrees because I am a terrible person.
    /// Also, compass degrees. For an SVG anyhow. I am a bastard.
    pub fn rotate_matrix(degrees: f64) -> Affine2<f64>{
        let angle = PI*(degrees/180.0);
        Affine2::from_matrix_unchecked(Matrix3::new(
            angle.cos(), -angle.sin(), 0.0,
            angle.sin(), angle.cos(), 0.0,
            0.0, 0.0, 1.0))
    }




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
            clip_previous: false,
            hatch_angle: None,
            stack: vec![],
        }
    }

    /// Pushes the current context onto the stack.
    pub fn push(&mut self) -> &mut Self{
        self.stack.push(Self{
            operations: vec![],
            transformation: match self.transformation.clone(){
                Some(transformation) => Some(transformation),
                None => None
            },
            stroke_color: self.stroke_color.clone(),
            outline_stroke: self.outline_stroke.clone(),
            fill_color: self.fill_color.clone(),
            line_join: self.line_join.clone(),
            line_cap: self.line_cap.clone(),
            pen_width: self.pen_width.clone(),
            clip_previous: self.clip_previous.clone(),
            hatch_angle: self.hatch_angle.clone(),
            stack: vec![],
        });
        self
    }

    /// Pops the previous context off the stack
    pub fn pop(&mut self) -> Result<&mut Self, ContextError>{
        let other = self.stack.pop().ok_or(ContextError::PoppedEmptyStack)?;
        self.transformation = match other.transformation.clone() {
            Some(transformation) => Some(transformation),
            None => None
        };
        self.stroke_color = other.stroke_color.clone();
        self.outline_stroke = other.outline_stroke.clone();
        self.fill_color = other.fill_color.clone();
        self.line_join = other.line_join.clone();
        self.line_cap = other.line_cap.clone();
        self.pen_width = other.pen_width.clone();
        self.hatch_angle = other.hatch_angle.clone();
        self.clip_previous = other.clip_previous.clone();
        Ok(self)
    }

    pub fn transform(&mut self, transformation: Option<&Affine2<f64>>) -> &mut Self{
        self.transformation = match transformation{
            Some(tx) => Some(tx.clone()),
            None => None
        };
        self

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
            clip_previous: self.clip_previous.clone(),
            hatch_angle: self.hatch_angle,
        });
    }

    /// Draws a simple line from x0,y0 to x1,y1
    pub fn line(&mut self, x0: f64, y0: f64, x1: f64, y1: f64) -> &mut Self {
        self.add_operation(
            Geometry::LineString(
                LineString::<f64>::new(
                    vec![
                        coord! {x: x0, y: y0},
                        coord! {x: x1, y: y1},
                    ])));
        self
    }

    /// Generates a spline from a set of points and renders as a
    /// multi line string. Doesn't do errors very well, just
    /// silently fails to draw.
    /// First and last point in points are NOT drawn, and set the 'tension'
    /// points which the line pulls from.
    pub fn spline(&mut self, points: &Vec<(f64, f64)>,
                  num_interpolated_segments: u32, tension: f64) -> &mut Self{
        let spline_opts = SplineOpts::new()
            .num_of_segments(num_interpolated_segments)
            .tension(tension);
        let points = match Points::try_from(points){
            Ok(pts) => pts,
            Err(e) => return self
        };
        let spline = cubic_spline::calc_spline(&points, &spline_opts);
        match spline {
            Ok(spts) => {
                self.add_operation(
                    Geometry::LineString(
                        LineString::<f64>::new(
                            spts
                                .get_ref()
                                .iter()
                                .map(|pt| coord!{x: pt.x, y: pt.y})
                                .collect()
                        )));
                self
            }
            Err(e) => self
        }
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
        let geo =
            Geometry::MultiPolygon(Geometry::Point(Point::new(x0, y0))
                                       .buffer(radius)
                                       .unwrap_or(MultiPolygon::new(vec![])));
        self.add_operation(geo);
        self
    }

    pub fn clip(&mut self, clip: bool) -> &mut Self{
        self.clip_previous = clip;
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

    pub fn matrix(&mut self, xform: Option<Affine2<f64>>) -> &mut Self{
        self.transformation = xform;
        self
    }

    pub fn to_svg(&self, arrangement: &Arrangement<f64>) -> Result<Document, ContextError> {
        struct OPLayer{
            stroke_lines: MultiLineString<f64>,
            fill_lines: MultiLineString<f64>,
            stroke: String,
            fill: String,
            stroke_width: f64,
            stroke_linejoin: String,
            stroke_linecap: String
        };
        let mut svg = arrangement
            .create_svg_document()
            .or(Err(ContextError::SvgGenerationError("Failed to create raw svg doc".into()).into()))?;
        let mut oplayers: Vec<OPLayer> = vec![];
        for op in &self.operations {
            let (stroke, fill) = op.render_to_lines();
            oplayers.push(OPLayer{
                stroke_lines: stroke,
                fill_lines: fill,
                stroke: op.stroke_color.clone(),
                fill: op.fill_color.clone(),
                stroke_width: op.pen_width,
                stroke_linejoin: op.line_join.clone(),
                stroke_linecap: op.line_cap.clone()
            });
        }
        assert_eq!(&self.operations.len(), &oplayers.len());

        // Iterate the layers, and clip their predecessors where appropriate.
        // NOTE: CLIPPING IS S_L_O_W AF.
        if self.operations.len()>1{
            for i in 0..(self.operations.len()-1) {
                for j in (i+1)..self.operations.len(){
                    if self.operations[j].clip_previous{
                        oplayers[i].stroke_lines = Geometry::MultiLineString(
                            oplayers[i].stroke_lines.clone())
                            .clipwith(&self.operations[j].content)
                            .unwrap_or(MultiLineString::<f64>::new(vec![]));
                        oplayers[i].fill_lines = Geometry::MultiLineString(
                            oplayers[i].fill_lines.clone())
                            .clipwith(&self.operations[j].content)
                            .unwrap_or(oplayers[i].fill_lines.clone());
                    }
                }
            }
        }

        let mut id = 0;
        for oplayer in oplayers{
            if !oplayer.stroke_lines.0.is_empty() {
                let slines = oplayer.stroke_lines.to_path(&arrangement);
                svg = svg.add(slines
                    .set("id", format!("outline-{}", id))
                    .set("fill", "none")
                    .set("stroke", oplayer.stroke.clone())
                    .set("stroke-width", oplayer.stroke_width)
                    .set("stroke-linejoin", oplayer.stroke_linejoin.clone())
                    .set("stroke-linecap", oplayer.stroke_linecap.clone())
                );
            }
            if !oplayer.fill_lines.0.is_empty() {
                let flines = oplayer.fill_lines.to_path(&arrangement);
                svg = svg.add(flines
                    .set("id", format!("fill-{}", id))
                    .set("fill", "none")
                    .set("stroke", oplayer.fill.clone())
                    .set("stroke-width", oplayer.stroke_width)
                    .set("stroke-linejoin", oplayer.stroke_linejoin.clone())
                    .set("stroke-linecap", oplayer.stroke_linecap.clone())
                );
                id = id + 1;
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
