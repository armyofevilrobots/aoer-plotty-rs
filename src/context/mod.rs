//! Provides the [`crate::context::Context`] struct which gives us a canvas-style drawing
//! context which provides plotter-ready SVG files. See `Context` for more details, and examples.
use std::error::Error;
use embed_doc_image::embed_doc_image;


use std::f64::consts::PI;
use std::ops::Deref;
use std::rc::Rc;
use geo_types::{coord, Coordinate, Geometry, GeometryCollection, LineString, MultiLineString, Polygon, Rect};
use svg::Document;
use crate::prelude::{Arrangement, HatchPattern, NoHatch, OutlineFillStroke, ToSvg};
use crate::geo_types::hatch::{Hatch, LineHatch};
use cubic_spline::{Points, SplineOpts};
use geo::map_coords::MapCoords;
use geos::{Geom, GeometryTypes};
use nalgebra::{Affine2, Matrix3, Point2 as NPoint2};
use nannou::prelude::PI_F64;
use crate::geo_types::clip::{LineClip, try_to_geos_geometry};
use crate::errors::ContextError;
use crate::geo_types::shapes;

/// Operations are private items used to store the operation stack
/// consisting of a combination of Geometry and Context state.
#[derive(Clone)]
struct Operation {
    content: Geometry<f64>,
    rendered: (MultiLineString<f64>, MultiLineString<f64>),
    transformation: Option<Affine2<f64>>,
    stroke_color: String,
    outline_stroke: Option<f64>,
    fill_color: String,
    line_join: String,
    line_cap: String,
    pen_width: f64,
    mask: Option<Geometry<f64>>,
    clip_previous: bool,
    hatch_pattern: Rc<dyn HatchPattern>,
    hatch_angle: f64,
}


impl Operation {
    fn consistent(&self, other: &Operation) -> bool {
        if self.stroke_color == other.stroke_color &&
            self.outline_stroke == other.outline_stroke &&
            self.fill_color == other.fill_color &&
            self.line_join == other.line_join &&
            self.line_cap == other.line_cap &&
            self.pen_width == other.pen_width &&
            self.hatch_angle == other.hatch_angle &&
            self.clip_previous == other.clip_previous // &&
        {
            true
        } else {
            false
        }
    }

    /// Helper function for converting polygons into sets of strings.
    fn poly2lines(poly: &Polygon<f64>, pen_width: f64,
                  hatch_angle: f64, hatch_pattern: Rc<dyn HatchPattern>)
                  -> (MultiLineString<f64>, MultiLineString<f64>)
    {
        let mut strokes = MultiLineString::new(vec![]);
        let mut fills = MultiLineString::new(vec![]);
        // Push the exterior
        strokes.0.push(poly.exterior().clone());
        for interior in poly.interiors() {
            strokes.0.push(interior.clone())
        }
        let hatch_pattern = hatch_pattern.deref();
        let hatches = poly
            .hatch(hatch_pattern, hatch_angle,
                   pen_width * 0.8, pen_width * 0.8)
            .unwrap_or(MultiLineString::new(vec![]));
        fills.0.append(&mut hatches.0.clone());
        (strokes, fills)
    }

    /// Helper to transform geometry when we have an affine transform set.
    fn xform_coord((x, y): &(f64, f64), affine: &Affine2<f64>) -> (f64, f64) {
        let out = affine * NPoint2::new(*x, *y);
        (out.x, out.y)
    }

    fn render_to_lines(&self) -> (MultiLineString<f64>, MultiLineString<f64>) {
        // Get the transformed geo, or just this geo at 1:1
        let txgeo = self.content.clone();

        // Masking was moved into the add_operation code.

        // Then turn it into outlines and fills
        let (outlines, fills) = match txgeo {
            Geometry::MultiLineString(mls) =>
                (mls.clone(), MultiLineString::new(vec![])),
            Geometry::LineString(ls) =>
                (MultiLineString::new(vec![ls.clone()]),
                 MultiLineString::new(vec![])),
            Geometry::Polygon(poly) =>
                Self::poly2lines(&poly, self.pen_width, self.hatch_angle,
                                 self.hatch_pattern.clone()),
            Geometry::MultiPolygon(polys) => {
                let mut strokes = MultiLineString::new(vec![]);
                let mut fills = MultiLineString::new(vec![]);
                for poly in polys {
                    let (new_strokes, new_fills) =
                        Self::poly2lines(&poly, self.pen_width, self.hatch_angle,
                                         self.hatch_pattern.clone());
                    strokes.0.append(&mut new_strokes.0.clone());
                    fills.0.append(&mut new_fills.0.clone());
                }
                (strokes, fills)
            }
            _ => (MultiLineString::new(vec![]), MultiLineString::new(vec![]))
        };

        // Finally, if we have outline stroke, then outline the existing strokes.
        let outlines = match self.outline_stroke {
            Some(stroke) => outlines
                .outline_fill_stroke_with_hatch(stroke,
                                                self.pen_width,
                                                &LineHatch {},
                                                self.hatch_angle)
                .unwrap_or(outlines),
            None => outlines
        };
        (outlines, fills)
    }
}


/// OPLayer is an operation layer, rendered into lines for drawing.
pub struct OPLayer {
    stroke_lines: MultiLineString<f64>,
    fill_lines: MultiLineString<f64>,
    stroke: String,
    fill: String,
    stroke_width: f64,
    stroke_linejoin: String,
    stroke_linecap: String,
}

impl OPLayer {
    pub fn to_lines(&self) -> (MultiLineString<f64>, MultiLineString<f64>) {
        (self.stroke_lines.clone(), self.fill_lines.clone())
    }
}


/// # Context
///
/// A Context is a _drawing_ context, used to perform operations against a
/// pseudo-canvas. Those operations are later collected up and turned into
/// an SVG, including line strokes, fills, and all the other useful tools
/// that we need to drive a plotter robot.
///
/// # Example
///
/// ```rust
/// use aoer_plotty_rs::context::Context;
///
/// let mut ctx = Context::new();
/// ctx.stroke("black")
///    .fill("red")
///    .pen(0.5)
///    .outline(Some(5.0))
///    .poly(vec![(0.0,0.0),
///           (25.0,0.0),
///           (25.0,25.0),
///           (0.0,25.0)],
/// vec![])
///    .outline(None)
///    .hatch(135.0)
///    .stroke("blue")
///    .fill("yellow")
///    .circle(12.5,12.5, 5.0)
///    .push()
///    .hatch(180.0)
///    .stroke("red")
///    .fill("green")
///    .circle(17.5,12.5,2.5)
///    .pop().unwrap()
///    .hatch(0.0)
///    .clip(true)
///    .circle(7.5,12.5,2.5)
///    .clip(false)
///    .stroke("brown")
///    .pen(1.0)
///    .line(0.0, 0.0, 3.0, 3.0)
///    .pen(0.1)
///    .outline(Some(1.0))
///    .stroke("pink")
///    .line(3.0, 0.0, 0.0, 3.0)
///    .stroke("purple")
///    .spline(&vec![(0.0, 25.0), (0.0, 25.0), (10.0, 20.0), (20.0,25.0), (25.0, 25.0)],
///            8, 0.5)
///    .push()  // Prepare for this transformation stuff...
///    .transform(Some(
///        &(Context::translate_matrix(25.0, 25.0)
///        * Context::rotate_matrix(45.0)
///        * Context::scale_matrix(1.0, 0.5)
///    ))) // Holy crap we can multiply these?! ;)
///    .stroke("cyan")
///    .circle(0.0, 0.0, 8.0)
///    .pop().unwrap() // We're back to purple and regular coords
///    .outline(None)
///     .stroke("green")
///     .regular_poly(8, 80.0, 80.0, 20.0, 0.0)
///     .star_poly(5, 30.0, 80.0, 10.0, 20.0, 0.0)
/// ;
/// ```
/// ![context_basic][context_basic]
#[embed_doc_image("context_basic", "images/context_basic.png")]
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
    mask: Option<Geometry<f64>>,
    clip_previous: bool,
    hatch_pattern: Rc<dyn HatchPattern>,
    hatch_angle: f64,
    stack: Vec<Context>,
}


impl Context {
    /// Helper to create a scaling matrix
    pub fn scale_matrix(sx: f64, sy: f64) -> Affine2<f64> {
        Affine2::from_matrix_unchecked(Matrix3::new(
            sx, 0.0, 0.0,
            0.0, sy, 0.0,
            0.0, 0.0, 1.0))
    }

    /// Viewbox helper. Useful to create an arbitrary viewbox for
    /// your SVGs.
    pub fn viewbox(x0: f64, y0: f64, x1: f64, y1: f64) -> Rect<f64> {
        Rect::new(
            coord! {x: x0, y: y0},
            coord! {x: x1, y: y1})
    }

    /// Helper to create a translation matrix
    pub fn translate_matrix(tx: f64, ty: f64) -> Affine2<f64> {
        Affine2::from_matrix_unchecked(Matrix3::new(
            1.0, 0.0, tx,
            0.0, 1.0, ty,
            0.0, 0.0, 1.0))
    }

    /// Angle is in degrees because I am a terrible person.
    /// Also, compass degrees. For an SVG anyhow. I am a bastard.
    pub fn rotate_matrix(degrees: f64) -> Affine2<f64> {
        let angle = PI * (degrees / 180.0);
        Affine2::from_matrix_unchecked(Matrix3::new(
            angle.cos(), -angle.sin(), 0.0,
            angle.sin(), angle.cos(), 0.0,
            0.0, 0.0, 1.0))
    }

    /// I can haz a new default drawing context?
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
            mask: None,
            clip_previous: false,
            hatch_pattern: NoHatch::gen(),
            hatch_angle: 0.0,
            stack: vec![],
        }
    }

    /// Masks any further operations with a clipping polygon. Only items
    /// inside the clipping poly will be used.
    pub fn mask_poly(&mut self, exterior: Vec<(f64, f64)>, interiors: Vec<Vec<(f64, f64)>>) -> &mut Self {
        let mask = Geometry::Polygon(
            Polygon::<f64>::new(
                LineString::new(
                    exterior
                        .iter()
                        .map(|(x, y)| coord! {x:*x, y:*y})
                        .collect()
                ),
                interiors
                    .iter()
                    .map(|interior| {
                        LineString::<f64>::new(interior
                            .iter()
                            .map(|(x, y)| coord! {x:*x, y:*y})
                            .collect::<Vec<Coordinate<f64>>>())
                    })
                    .collect(),
            ));
        self.set_mask(&Some(mask));
        self
    }

    pub fn mask_box(&mut self, x0: f64, y0: f64, x1: f64, y1: f64) -> &mut Self {
        self.mask_poly(vec![
            (x0, y0),
            (x1, y0),
            (x1, y1),
            (x0, y1),
            (x0, y0),
        ], vec![])
    }

    /// Sets the mask to Geometry, or None.
    pub fn set_mask(&mut self, mask: &Option<Geometry<f64>>) -> &mut Self {
        self.mask = match mask {
            Some(maskgeo) => Some(match &self.transformation {
                Some(affine) => maskgeo.map_coords(
                    |xy| Operation::xform_coord(xy, affine)),
                None => maskgeo.clone()
            }),
            None => mask.clone()
        };
        self
    }

    /// Pushes the current context onto the stack.
    pub fn push(&mut self) -> &mut Self {
        self.stack.push(Self {
            operations: vec![],
            transformation: match self.transformation.clone() {
                Some(transformation) => Some(transformation),
                None => None
            },
            stroke_color: self.stroke_color.clone(),
            outline_stroke: self.outline_stroke.clone(),
            fill_color: self.fill_color.clone(),
            line_join: self.line_join.clone(),
            line_cap: self.line_cap.clone(),
            pen_width: self.pen_width.clone(),
            mask: self.mask.clone(),
            clip_previous: self.clip_previous.clone(),
            hatch_pattern: self.hatch_pattern.clone(),
            hatch_angle: self.hatch_angle,
            stack: vec![],
        });
        self
    }

    /// Pops the previous context off the stack
    pub fn pop(&mut self) -> Result<&mut Self, ContextError> {
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
        self.hatch_angle = other.hatch_angle;
        self.clip_previous = other.clip_previous.clone();
        Ok(self)
    }

    /// Set the transformation matrix for subsequent ops. Take a look at the transformation
    /// helpers ([`crate::context::Context::scale_matrix`],
    /// [`crate::context::Context::translate_matrix`], and
    /// [`crate::context::Context::rotate_matrix`], which are great
    /// if you don't want to generate your own unsafe Affine2 transformations. Also, usefully,
    /// these transformations can be COMPOSED via multiplication. Note that the order of the
    /// compositions is right-to-left, so the last in the chain of multiplications is the
    /// first one to be performed. See the example in context_basic.rs for more info.
    pub fn transform(&mut self, transformation: Option<&Affine2<f64>>) -> &mut Self {
        self.transformation = match transformation {
            Some(tx) => Some(tx.clone()),
            None => None
        };
        self
    }

    /// Adds any arbitrary Geometry type (geo_types geometry)
    fn add_operation(&mut self, geometry: Geometry<f64>) {
        let mut op = Operation {
            content: geometry,
            rendered: (MultiLineString::new(vec![]),
                       MultiLineString::new(vec![])),
            transformation: self.transformation.clone(),
            stroke_color: self.stroke_color.clone(),
            outline_stroke: self.outline_stroke.clone(),
            fill_color: self.fill_color.clone(),
            line_join: self.line_join.clone(),
            line_cap: self.line_cap.clone(),
            pen_width: self.pen_width.clone(),
            mask: self.mask.clone(),
            clip_previous: self.clip_previous.clone(),
            hatch_pattern: self.hatch_pattern.clone(),
            hatch_angle: self.hatch_angle,
        };
        if let Some(tx) = &op.transformation {
            op.content = op.content.map_coords(|xy| Operation::xform_coord(xy, tx));
        }
        op.content = match &op.mask {
            Some(mask) => {
                let ggeo = try_to_geos_geometry(&op.content)
                    .unwrap_or(geos::Geometry::create_empty_collection(GeometryTypes::GeometryCollection)
                        .unwrap());
                let mggeo = try_to_geos_geometry(mask)
                    .unwrap_or(geos::Geometry::create_empty_collection(GeometryTypes::GeometryCollection)
                        .unwrap());
                let masked_geo = ggeo.intersection(&mggeo)
                    .unwrap_or(geos::Geometry::create_empty_collection(GeometryTypes::GeometryCollection)
                        .unwrap());
                geo_types::Geometry::<f64>::try_from(masked_geo).unwrap_or(Geometry::GeometryCollection::<f64>(Default::default()))
            }
            None => op.content
        };

        op.rendered = op.render_to_lines();
        self.operations.push(op);
    }


    /// Adds a geometry to the operations list. Has some checking to make it safe
    /// for general users.
    pub fn geometry(&mut self, geometry: &Geometry<f64>) -> &mut Self {
        match &geometry {
            Geometry::LineString(_ls) => {
                self.add_operation(geometry.clone());
            }
            Geometry::MultiLineString(_mls) => {
                self.add_operation(geometry.clone());
            }
            Geometry::Polygon(_poly) => {
                self.add_operation(geometry.clone());
            }
            Geometry::MultiPolygon(_mp) => {
                self.add_operation(geometry.clone());
            }
            Geometry::Rect(rect) => self.add_operation(Geometry::Polygon(Polygon::new(
                LineString::new(vec![
                    coord! {x: rect.min().x, y: rect.min().y},
                    coord! {x: rect.max().x, y: rect.min().y},
                    coord! {x: rect.max().x, y: rect.max().y},
                    coord! {x: rect.min().x, y: rect.max().y},
                    coord! {x: rect.min().x, y: rect.min().y},
                ]),
                vec![],
            ))),
            Geometry::Triangle(tri) => self.add_operation(Geometry::Polygon(Polygon::new(
                LineString::new(vec![
                    coord! {x: tri.0.x, y:tri.0.y},
                    coord! {x: tri.1.x, y:tri.1.y},
                    coord! {x: tri.2.x, y:tri.2.y},
                    coord! {x: tri.0.x, y:tri.0.y},
                ]),
                vec![],
            ))),
            Geometry::GeometryCollection(collection) => {
                for item in collection {
                    self.geometry(item);
                }
            }
            Geometry::Line(line) => self.add_operation(
                Geometry::LineString(LineString(vec![
                    coord! {x: line.start.x, y: line.start.y},
                    coord! {x: line.end.x, y: line.end.y},
                ]))),
            Geometry::Point(pt) => {
                self.circle(
                    pt.0.x, pt.0.y, self.pen_width / 2.0,
                );
            }
            Geometry::MultiPoint(points) => {
                for pt in points {
                    self.circle(
                        pt.0.x, pt.0.y, self.pen_width / 2.0,
                    );
                }
            }
        };
        self
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
                  num_interpolated_segments: u32, tension: f64) -> &mut Self {
        let spline_opts = SplineOpts::new()
            .num_of_segments(num_interpolated_segments)
            .tension(tension);
        let points = match Points::try_from(points) {
            Ok(pts) => pts,
            Err(_e) => return self
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
                                .map(|pt| coord! {x: pt.x, y: pt.y})
                                .collect()
                        )));
                self
            }
            Err(_e) => self
        }
    }

    /// centerpoint arc
    /// Draw an arc around x0,y0 with the given radius, from deg0 to deg1. Arcs will always be
    /// coords oriented clockwise from "north" on an SVG. ie: 45 to 135 will be NE to SE.
    pub fn arc_center(&mut self, x0: f64, y0: f64, radius: f64, deg0: f64, deg1: f64) -> &mut Self {
        // let radius = radius.abs();
        // // Clamp the angle.
        // let deg0 = PI_F64 * ((deg0 % 360.0) / 180.0);
        // let deg1 = PI_F64 * ((deg1 % 360.0) / 180.0);
        // let (deg0, deg1) = if deg0 > deg1 {
        //     (deg1, deg0)
        // } else {
        //     (deg0, deg1)
        // };
        // let sides = 1000.min(32.max(usize::from_f64(radius).unwrap_or(1000) * 4));
        // let segments = (deg1 - deg0) * f64::from(sides as i32).floor();
        // let seg_size = (deg1 - deg0) / segments;
        // let mut ls = LineString::<f64>::new(vec![]);
        // let mut angle = deg0;
        // for _segment in 0..(segments as i32) {
        //     ls.0.push(coord! {x: x0+radius*angle.sin(), y: y0+radius*angle.cos()});
        //     angle += seg_size;
        // }
        // if deg1 - angle > 0.0 {
        //     ls.0.push(coord! {x: x0+radius*deg1.sin(), y: y0+radius*deg1.cos()});
        // }
        let ls = crate::geo_types::shapes::arc_center(x0, y0, radius, deg0, deg1);
        let ls = ls.map_coords(|(x,y)| (x.clone(), -y.clone()));
        self.add_operation(Geometry::LineString(ls));

        self
    }

    /// What it says on the box. Draws a simple rectangle on the context.
    pub fn rect(&mut self, x0: f64, y0: f64, x1: f64, y1: f64) -> &mut Self {
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
    pub fn poly(&mut self, exterior: Vec<(f64, f64)>, interiors: Vec<Vec<(f64, f64)>>) -> &mut Self {
        self.add_operation(
            Geometry::Polygon(
                Polygon::<f64>::new(
                    LineString::new(
                        exterior
                            .iter()
                            .map(|(x, y)| coord! {x:*x, y:*y})
                            .collect()
                    ),
                    interiors
                        .iter()
                        .map(|interior| {
                            LineString::<f64>::new(interior
                                .iter()
                                .map(|(x, y)| coord! {x:*x, y:*y})
                                .collect::<Vec<Coordinate<f64>>>())
                        })
                        .collect(),
                )));
        self
    }

    /// Draws a circle. Actually just buffers a point, and returns a polygon
    /// which it draws on the context.
    pub fn circle(&mut self, x0: f64, y0: f64, radius: f64) -> &mut Self {
        // let radius = radius.abs();
        // let sides = 1000.min(32.max(usize::from_f64(radius).unwrap_or(1000) * 4));
        // self.regular_poly(sides, x0, y0, radius, 0.0)
        self.add_operation(shapes::circle(x0, y0, radius));
        self
    }

    /// Circumscribed regular polygon. The vertices of the polygon will be situated on a
    /// circle defined by the given radius. Polygon will be centered at x,y.
    pub fn regular_poly(&mut self, sides: usize, x: f64, y: f64, radius: f64, rotation: f64) -> &mut Self {
        // all the way around to the start again, and hit the first point twice to close it.
        // if sides < 3 { return self; };
        //
        // let geo = Geometry::Polygon(Polygon::new(LineString::new((0..(sides + 2))
        //     .map(|i| {
        //         let angle = rotation - PI_F64 / 2.0 +
        //             (f64::from(i as i32) / f64::from(sides as i32)) * (2.0 * PI_F64);
        //         coord! {x: x+angle.cos() * radius, y: y+angle.sin() * radius}
        //     }).collect()
        // ), vec![]));
        let geo = shapes::regular_poly(sides, x, y, radius, rotation);
        self.add_operation(geo);
        self
    }

    /// Regular star polygon. This is effectively a _star_ shape with the number of points indicated,
    /// and with inner and outer radiuses which correspond to the valleys and tips of the star
    /// respectively. Note: this is not a star polygon in the strict mathematical sense. This is
    /// just a polygon that is in the shape of a star. I may or may not get to regular star polygons
    /// (in the canonical mathematical sense) at some point.
    pub fn star_poly(&mut self, sides: usize, x: f64, y: f64, inner_radius: f64, outer_radius: f64, rotation: f64) -> &mut Self {
        if sides < 3 { return self; };
        let mut exterior = LineString::<f64>::new(vec![]);
        for i in 0..sides {
            let angle_a = rotation - PI_F64 / 2.0 +
                (f64::from(i as i32) / f64::from(sides as i32)) * (2.0 * PI_F64);
            let angle_b = rotation - PI_F64 / 2.0 +
                ((f64::from(i as i32) + 0.5) / f64::from(sides as i32)) * (2.0 * PI_F64);
            exterior.0.push(coord! {
                x: x+angle_a.cos() * outer_radius,
                y: y+angle_a.sin() * outer_radius});
            exterior.0.push(coord! {
                x: x+angle_b.cos() * inner_radius,
                y: y+angle_b.sin() * inner_radius});
        }
        // and close it...
        exterior.0.push(coord! {
            x: x+(rotation - PI_F64/2.0).cos() * outer_radius,
            y: y+(rotation - PI_F64/2.0).sin() * outer_radius});
        self.add_operation(Geometry::Polygon(Polygon::new(exterior, vec![])));
        self
    }

    /// Sets the clipping state. Any subsequent objects will clip their predecessors.
    /// Note that this is an EXPENSIVE operation, so you might want to leave it off
    /// if you're sure you won't have intersections.
    pub fn clip(&mut self, clip: bool) -> &mut Self {
        self.clip_previous = clip;
        self
    }

    /// Sets the stroke color
    pub fn stroke(&mut self, color: &str) -> &mut Self {
        self.stroke_color = color.to_string();
        self
    }

    /// Sets the fill color
    pub fn fill(&mut self, color: &str) -> &mut Self {
        self.fill_color = color.to_string();
        self
    }

    /// Sets the hatch state, either None for no hatching or
    /// Some(angle) to set a hatching angle. Will use the current
    /// pen width as the spacing between hatch lines.
    pub fn hatch(&mut self, angle: f64) -> &mut Self {
        self.hatch_angle = angle;
        self
    }

    /// Sets the pen width
    pub fn pen(&mut self, width: f64) -> &mut Self {
        self.pen_width = width;
        self
    }

    /// Set the hatch pattern
    pub fn pattern(&mut self, pattern: Rc<dyn HatchPattern>) -> &mut Self {
        self.hatch_pattern = pattern.clone();
        self
    }

    /// Neat option. Instead of drawing a complex poly, why not just
    /// set outline, and have the lines/polys/whatever you subsequently
    /// draw get buffered and hatched to imitate a really thick pen?
    /// I knew you'd like this one :D
    pub fn outline(&mut self, stroke: Option<f64>) -> &mut Self {
        self.outline_stroke = stroke;
        self
    }


    /// Flatten will take a context and "flatten" together all polygons
    /// of a given color and "depth". What that means is that we watch for
    /// changes to fill/color/etc, and set those as boundaries. Then every
    /// geometry within a set of boundaries is flattened as "unions" into
    /// a single geometry. This is nice because overlapping polygons get
    /// turned into a single unified polygon, and their fills are no longer
    /// disjoint (and they don't have unexpected overlapping boundary lines).
    /// See the 10_hello example for more details.
    /// Unlike the other methods, this one generates an entirely new context
    /// including a NEW HISTORY, so you can't use push/pop to go back, and
    /// the individual operations are (obviously) lost.
    pub fn flatten(&self) -> Self {
        let mut new_ctx = Context::new();
        new_ctx.add_operation(Geometry::MultiLineString(MultiLineString::new(vec![])));
        let mut last_operation = new_ctx.operations[0].clone();
        let tmp_gt_op = Geometry::GeometryCollection(GeometryCollection::new_from(vec![]));
        let mut current_geometry = try_to_geos_geometry(
            &tmp_gt_op)
            .unwrap_or(geos::Geometry::create_empty_collection(GeometryTypes::GeometryCollection)
                .expect("Failed to generate default geos::geometry"));
        // let mut interim_geo = geos::Geometry::create_empty_collection(GeometryTypes::GeometryCollection)?;
        for operation in self.operations.iter() {
            if operation.consistent(&last_operation) {
                // Union current_geometry with the operation
                let cgeo = try_to_geos_geometry(&operation.content)
                    .unwrap_or(geos::Geometry::create_empty_collection(GeometryTypes::GeometryCollection)
                        .expect("Failed to generate default geos::geometry"));
                // current_geometry = cgeo.union(&current_geometry)
                //     .unwrap_or(Geom::clone(&current_geometry));
                current_geometry = geos::Geometry::create_geometry_collection(vec![current_geometry, cgeo])
                    .expect("Cannot append geometry into collection.");
            } else {
                // Duplicate the state into the context, and create a new current_geometry bundle
                new_ctx.stroke_color = operation.stroke_color.clone();
                new_ctx.outline_stroke = operation.outline_stroke.clone();
                new_ctx.fill_color = operation.fill_color.clone();
                new_ctx.line_join = operation.line_join.clone();
                new_ctx.line_cap = operation.line_cap.clone();
                new_ctx.pen_width = operation.pen_width.clone();
                new_ctx.clip_previous = operation.clip_previous.clone();
                new_ctx.hatch_pattern = operation.hatch_pattern.clone();
                new_ctx.hatch_angle = operation.hatch_angle;

                current_geometry = current_geometry
                    .unary_union()
                    .unwrap_or(current_geometry);

                new_ctx.geometry(&geo_types::Geometry::try_from(current_geometry)
                    .unwrap_or(
                        Geometry::GeometryCollection(
                            GeometryCollection::new_from(vec![]))));
                last_operation = operation.clone();
                current_geometry = try_to_geos_geometry(
                    &operation.content)
                    .unwrap_or(geos::Geometry::create_empty_collection(GeometryTypes::GeometryCollection)
                        .expect("If we failed to convert or fallback, something is very wrong."));
            }
        }
        // get the last one.
        current_geometry = current_geometry
            .unary_union()
            .unwrap_or(current_geometry);
        new_ctx.geometry(&geo_types::Geometry::try_from(current_geometry)
            .unwrap_or(
                Geometry::GeometryCollection(
                    GeometryCollection::new_from(vec![]))));
        new_ctx
    }

    pub fn to_geo(&self) -> Result<Geometry<f64>, Box<dyn Error>> {
        let mut all: Vec<Geometry<f64>> = vec![];
        for operation in &self.operations {
            // all.add(GeometryCollection::<f64>::new_from(vec![operation.content.clone()]));
            all.push(operation.content.clone().into());
        }
        Ok(Geometry::GeometryCollection(GeometryCollection::<f64>::new_from(all)))
    }

    /// Generate layers of perimeters and fills
    pub fn to_layers(&self) -> Vec<OPLayer> {
        let mut oplayers: Vec<OPLayer> = vec![];
        for op in &self.operations {
            // let (stroke, fill) = op.render_to_lines();
            let (stroke, fill) = op.rendered.clone();
            oplayers.push(OPLayer {
                stroke_lines: stroke,
                fill_lines: fill,
                stroke: op.stroke_color.clone(),
                fill: op.fill_color.clone(),
                stroke_width: op.pen_width,
                stroke_linejoin: op.line_join.clone(),
                stroke_linecap: op.line_cap.clone(),
            });
        }
        assert_eq!(&self.operations.len(), &oplayers.len());

        // Iterate the layers, and clip their predecessors where appropriate.
        // NOTE: CLIPPING IS S_L_O_W AF.
        if self.operations.len() > 1 {
            for i in 0..(self.operations.len() - 1) {
                for j in (i + 1)..self.operations.len() {
                    if self.operations[j].clip_previous {
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
        oplayers
    }

    /// Take this giant complex thing and generate and SVG Document, or an error. Whatever.
    pub fn to_svg(&self, arrangement: &Arrangement<f64>) -> Result<Document, ContextError> {
        let oplayers = self.to_layers();

        let mut svg = arrangement
            .create_svg_document()
            .or(Err(ContextError::SvgGenerationError("Failed to create raw svg doc".into()).into()))?;

        let mut id = 0;
        for oplayer in oplayers {
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
    use geo_types::{Rect, Triangle};
    use super::*;

    #[test]
    fn test_context_new() {
        let context = Context::new();
        assert_eq!(context.transformation, None);
    }

    #[test]
    fn test_minimal_rect() {
        let mut context = Context::new();
        context.stroke("red")
            .pen(0.8)
            .fill("blue")
            .hatch(45.0)
            .rect(10.0, 10.0, 50.0, 50.0);
        let arrangement = Arrangement::FitCenterMargin(
            10.0,
            Rect::new(coord! {x: 0.0, y: 0.0}, coord! {x:100.0, y:100.0}),
            false);
        context.to_svg(&arrangement).unwrap();
    }

    #[test]
    fn test_arc_c() {
        let mut context = Context::new();
        context.stroke("red")
            .pen(0.8)
            .fill("blue")
            .hatch(45.0)
            .arc_center(0.0, 0.0, 10.0, 45.0, 180.0);
        let arrangement = Arrangement::unit(&Rect::new(coord! {x: 0.0, y: 0.0}, coord! {x:100.0, y:100.0}));
        let svg1 = context.to_svg(&arrangement).unwrap();
        let mut context = Context::new();
        context.stroke("red")
            .pen(0.8)
            .fill("blue")
            .hatch(45.0)
            .arc_center(0.0, 0.0, 10.0, 180.0, 45.0);
        let arrangement = Arrangement::unit(&Rect::new(coord! {x: 0.0, y: 0.0}, coord! {x:100.0, y:100.0}));
        let svg2 = context.to_svg(&arrangement).unwrap();
        // println!("SVG ARC IS: {}", svg2.to_string());
        // Make sure that order of angles is irrelevant
        assert_eq!(svg2.to_string(), svg1.to_string());
    }

    #[test]
    fn test_minimal_poly() {
        let mut context = Context::new();
        context.stroke("red");
        context.pen(0.8);
        context.fill("blue");
        context.hatch(45.0);
        context.poly(vec![(10.0, 10.0), (50.0, 10.0), (25.0, 25.0)],
                     vec![]);
        let arrangement = Arrangement::FitCenterMargin(
            10.0,
            Rect::new(coord! {x: 0.0, y: 0.0}, coord! {x:100.0, y:100.0}),
            false);
        context.to_svg(&arrangement).unwrap();
    }

    #[test]
    fn test_regular_poly() {
        let mut context = Context::new();
        context.stroke("red");
        context.pen(0.8);
        context.fill("blue");
        context.hatch(45.0);
        context.regular_poly(4, 50.0, 50.0, 100.0, 0.0);
        let arrangement = Arrangement::FitCenterMargin(
            10.0,
            Rect::new(coord! {x: 0.0, y: 0.0}, coord! {x:100.0, y:100.0}),
            false);
        context.to_svg(&arrangement).unwrap();
    }

    #[test]
    fn test_5_pointed_star() {
        let mut context = Context::new();
        context.stroke("red")
            .pen(0.8)
            .fill("blue")
            .hatch(45.0)
            .star_poly(5, 50.0, 50.0, 20.0, 40.0, 0.0);
    }

    #[test]
    fn test_flatten_simple() {
        let mut context = Context::new();
        context.stroke("red");
        context.pen(0.5);
        context.fill("blue");
        context.hatch(45.0);
        context.rect(10.0, 10.0, 30.0, 30.0);
        context.rect(20.0, 20.0, 40.0, 40.0);
        context = context.flatten();
        let arrangement = Arrangement::unit(&Rect::new(coord! {x: 0.0, y: 0.0}, coord! {x:100.0, y:100.0}));
        let svg = context.to_svg(&arrangement).unwrap();
        assert_eq!(svg.to_string(),
                   concat!(
                   "<svg height=\"100mm\" viewBox=\"0 0 100 100\" width=\"100mm\" xmlns=\"http://www.w3.org/2000/svg\">\n",
                   "<path d=\"M30,10 L10,10 L10,30 L20,30 L20,40 L40,40 L40,20 L30,20 L30,10\" fill=\"none\" id=\"outline-0\" ",
                   "stroke=\"red\" stroke-linecap=\"round\" stroke-linejoin=\"round\" stroke-width=\"0.5\"/>\n</svg>"
                   ));
    }

    #[test]
    fn test_flatten_complex() {
        let mut context = Context::new();
        context.stroke("red");
        context.pen(0.5);
        context.fill("blue");
        // context.hatch(45.0);
        // context.pattern(LineHatch::gen());
        context.rect(10.0, 10.0, 30.0, 30.0);
        context.rect(20.0, 20.0, 40.0, 40.0);
        context.rect(32.0, 32.0, 48.0, 48.0);
        context.stroke("black")
            .clip(true)
            .rect(22.0, 22.0, 38.0, 38.0);

        context = context.flatten();
        let arrangement = Arrangement::unit(&Rect::new(coord! {x: 0.0, y: 0.0}, coord! {x:100.0, y:100.0}));
        let svg = context.to_svg(&arrangement).unwrap();
        // println!("svg: {}", svg.to_string());
        assert_eq!(svg.to_string(),
                   concat!(
                   "<svg height=\"100mm\" viewBox=\"0 0 100 100\" width=\"100mm\" xmlns=\"http://www.w3.org/2000/svg\">\n",
                   "<path d=\"\" fill=\"none\" id=\"outline-0\" stroke=\"black\" stroke-linecap=\"round\" stroke-linejoin=\"round\" stroke-width=\"0.5\"/>\n",
                   "<path d=\"\" fill=\"none\" id=\"fill-0\" stroke=\"black\" stroke-linecap=\"round\" stroke-linejoin=\"round\" stroke-width=\"0.5\"/>\n",
                   "<path d=\"M30,10 L10,10 L10,30 L20,30 L20,40 L32,40 L32,48 L48,48 L48,32 L40,32 L40,20 L30,20 L30,10\" fill=\"none\" id=\"outline-1\" ",
                   "stroke=\"black\" stroke-linecap=\"round\" stroke-linejoin=\"round\" stroke-width=\"0.5\"/>\n",
                   "<path d=\"\" fill=\"none\" id=\"fill-1\" stroke=\"blue\" stroke-linecap=\"round\" stroke-linejoin=\"round\" stroke-width=\"0.5\"/>\n",
                   "<path d=\"M22,22 L38,22 L38,38 L22,38 L22,22\" fill=\"none\" id=\"outline-2\" stroke=\"black\" stroke-linecap=\"round\" ",
                   "stroke-linejoin=\"round\" stroke-width=\"0.5\"/>\n</svg>"
                   ));
    }

    #[test]
    fn test_to_geo() {
        let mut context = Context::new();
        context
            .stroke("red")
            .pen(0.8)
            .fill("blue")
            .hatch(45.0)
            .rect(10.0, 10.0, 90.0, 90.0);

        let foo = context.to_geo().unwrap();
        // println!("OFOO IS {:?}", &foo);
        let mut context = Context::new();
        context.stroke("green");
        context.pen(0.8);
        context.fill("purple");
        context.geometry(&foo);
        let arrangement = Arrangement::unit(&Rect::new(coord! {x: 0.0, y: 0.0}, coord! {x:100.0, y:100.0}));
        /*let svg =*/ context.to_svg(&arrangement).unwrap();
        // println!("svg : {}", svg.to_string());
    }

    #[test]
    fn test_various_geometry() {
        let mut context = Context::new();
        context.stroke("red");
        context.pen(0.8);
        context.fill("blue");
        context.hatch(45.0)
            .geometry(
                &Geometry::Polygon(
                    Polygon::new(
                        LineString::new(
                            vec![
                                coord! {x: 0.0, y:0.0},
                                coord! {x: 100.0, y:0.0},
                                coord! {x:100.0, y:100.0},
                                coord! {x:0.0, y:100.0},
                                coord! {x: 0.0, y:0.0},
                            ]), vec![])))
            .geometry(
                &Geometry::Triangle(
                    Triangle::new(
                        coord! {x: 0.0, y:0.0},
                        coord! {x: 100.0, y:0.0},
                        coord! {x:100.0, y:100.0})));
        let arrangement = Arrangement::unit(&Rect::new(coord! {x: 0.0, y: 0.0}, coord! {x:100.0, y:100.0}));
        let svg = context.to_svg(&arrangement).unwrap();
        assert_eq!(svg.to_string(), concat!(
        "<svg height=\"100mm\" viewBox=\"0 0 100 100\" width=\"100mm\" xmlns=\"http://www.w3.org/2000/svg\">\n",
        "<path d=\"M0,0 L100,0 L100,100 L0,100 L0,0\" fill=\"none\" id=\"outline-0\" stroke=\"red\" stroke-linecap=\"round\" stroke-linejoin=\"round\" stroke-width=\"0.8\"/>\n",
        "<path d=\"M0,0 L100,0 L100,100 L0,0\" fill=\"none\" id=\"outline-0\" stroke=\"red\" stroke-linecap=\"round\" stroke-linejoin=\"round\" stroke-width=\"0.8\"/>\n",
        "</svg>"
        ));
    }
}
