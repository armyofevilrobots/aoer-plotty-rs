use std::borrow::BorrowMut;
use std::ops::Deref;
use std::rc::Rc;
use geo_types::{Geometry, MultiLineString, Polygon};
use crate::prelude::{HatchPattern, LineHatch, OutlineFillStroke, Hatch};
use geo::map_coords::MapCoords;
use geos::{Geom, GeometryTypes};
// use geos::GeometryTypes::Point;
use nalgebra::{Affine2, Point2 as NPoint2};
use crate::geo_types::clip::{try_to_geos_geometry};
pub use kurbo::BezPath;
pub use kurbo::Point as BezPoint;
use geo::simplify::Simplify;


/// Operations are private items used to store the operation stack
/// consisting of a combination of Geometry and Context state.
#[derive(Clone)]
pub struct Operation {
    pub(crate) accuracy: f64,
    pub(crate) content: Geometry<f64>,
    pub(crate) rendered: (MultiLineString<f64>, MultiLineString<f64>),
    pub(crate) transformation: Option<Affine2<f64>>,
    pub(crate) stroke_color: String,
    pub(crate) outline_stroke: Option<f64>,
    pub(crate) fill_color: String,
    pub(crate) line_join: String,
    pub(crate) line_cap: String,
    pub(crate) pen_width: f64,
    pub(crate) mask: Option<Geometry<f64>>,
    pub(crate) clip_previous: bool,
    pub(crate) hatch_pattern: Rc<dyn HatchPattern>,
    pub(crate) hatch_angle: f64,
}


impl Operation {
    /// Transform content by my transformation
    pub fn transformed(&self, content: &Geometry<f64>) -> Geometry<f64> {
        if let Some(tx) = &self.transformation.clone() {
            // let mut content = content.clone();
            content.map_coords(|xy| Operation::xform_coord(xy, tx))
        } else {
            content.clone()
        }
    }

    pub fn render(mut self) -> Self {
        if let Some(tx) = &self.transformation {
            self.content = self.content.map_coords(|xy| Operation::xform_coord(xy, tx));
        }
        self.content = match &self.mask {
            Some(mask) => {
                let ggeo = try_to_geos_geometry(&self.content)
                    .unwrap_or(geos::Geometry::create_empty_collection(GeometryTypes::GeometryCollection)
                        .unwrap());
                let mggeo = try_to_geos_geometry(mask)
                    .unwrap_or(geos::Geometry::create_empty_collection(GeometryTypes::GeometryCollection)
                        .unwrap());
                let masked_geo = ggeo.intersection(&mggeo)
                    .unwrap_or(geos::Geometry::create_empty_collection(GeometryTypes::GeometryCollection)
                        .unwrap());
                geo_types::Geometry::<f64>::try_from(masked_geo)
                    .unwrap_or(Geometry::GeometryCollection::<f64>(Default::default()))
            }
            None => self.content
        };

        self.rendered = self.render_to_lines();
        self
    }

    pub fn consistent(&self, other: &Operation) -> bool {
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
        // let mut fills = MultiLineString::new(vec![]);
        // Push the exterior
        strokes.0.push(poly.exterior().clone());
        for interior in poly.interiors() {
            strokes.0.push(interior.clone())
        }
        let hatch_pattern = hatch_pattern.deref();
        // println!("Hatching with pattern: {:?}", &hatch_pattern);
        let hatches = poly
            .hatch(hatch_pattern, hatch_angle,
                   pen_width, pen_width)
            .unwrap_or(MultiLineString::new(vec![]));
        // fills.0.append(&mut hatches.0.clone());
        (strokes, hatches)
    }

    /// Helper to transform geometry when we have an affine transform set.
    pub fn xform_coord((x, y): &(f64, f64), affine: &Affine2<f64>) -> (f64, f64) {
        let out = affine * NPoint2::new(*x, *y);
        (out.x, out.y)
    }

    fn help_render_geo(txgeo: &Geometry<f64>, pen_width: f64,
                       hatch_angle: f64, hatch_pattern: Rc<dyn HatchPattern>) -> (MultiLineString<f64>, MultiLineString<f64>) {
        match txgeo {
            Geometry::MultiLineString(mls) =>
                (mls.clone(), MultiLineString::new(vec![])),
            Geometry::LineString(ls) =>
                (MultiLineString::new(vec![ls.clone()]),
                 MultiLineString::new(vec![])),
            Geometry::Polygon(poly) =>
                Self::poly2lines(&poly, pen_width, hatch_angle,
                                 hatch_pattern.clone()),
            Geometry::MultiPolygon(polys) => {
                let mut strokes = MultiLineString::new(vec![]);
                let mut fills = MultiLineString::new(vec![]);
                for poly in polys {
                    let (new_strokes, new_fills) =
                        Self::poly2lines(&poly, pen_width, hatch_angle,
                                         hatch_pattern.clone());
                    strokes.0.append(&mut new_strokes.0.clone());
                    fills.0.append(&mut new_fills.0.clone());
                }
                (strokes, fills)
            }
            Geometry::GeometryCollection(collection) => {
                let mut strokes = MultiLineString::new(vec![]);
                let mut fills = MultiLineString::new(vec![]);
                for item in collection {
                    // println!("Adding geo collection item: {:?}", &item);
                    let (mut tmpstrokes, mut tmpfills) = Operation::help_render_geo(
                        item,
                        pen_width,
                        hatch_angle,
                        hatch_pattern.clone());
                    strokes.0.append(tmpstrokes.0.borrow_mut());
                    fills.0.append(tmpfills.0.borrow_mut());
                }
                // println!("Got strokes, fills of: \n{:?}, \n{:?}\n\n", &strokes, &fills);
                (strokes, fills)
            }

            _ => (MultiLineString::new(vec![]), MultiLineString::new(vec![]))
        }
    }

    pub fn render_to_lines(&self) -> (MultiLineString<f64>, MultiLineString<f64>) {
        // Get the transformed geo, or just this geo at 1:1
        // let txgeo = self.content.clone();

        // Masking was moved into the add_operation code.

        // Then turn it into outlines and fills
        let (outlines, fills) =
            Operation::help_render_geo(&self.content,
                                       self.pen_width,
                                       self.hatch_angle, self.hatch_pattern.clone());

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
        (outlines.simplify(&self.accuracy), fills.simplify(&self.accuracy))
    }
}


/// OPLayer is an operation layer, rendered into lines for drawing.
pub struct OPLayer {
    pub(crate) stroke_lines: MultiLineString<f64>,
    pub(crate) fill_lines: MultiLineString<f64>,
    pub(crate) stroke: String,
    pub(crate) fill: String,
    pub(crate) stroke_width: f64,
    pub(crate) stroke_linejoin: String,
    pub(crate) stroke_linecap: String,
}

impl OPLayer {
    pub fn to_lines(&self) -> (MultiLineString<f64>, MultiLineString<f64>) {
        (self.stroke_lines.clone(), self.fill_lines.clone())
    }

    pub fn stroke(&self) -> String { self.stroke.clone() }
    pub fn fill(&self) -> String { self.fill.clone() }
}
