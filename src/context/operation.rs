use crate::context::line_filter::LineFilter;
use crate::prelude::{Hatch, Hatches, OutlineFillStroke};
use geo::coord;
use geo::map_coords::MapCoords;
use geo::Coord;
use geo_types::{Geometry, MultiLineString, MultiPolygon, Polygon};
use geos::{Geom, GeometryTypes};
use std::borrow::BorrowMut;
use std::sync::Arc;
// use geos::GeometryTypes::Point;
use crate::geo_types::clip::try_to_geos_geometry;
use geo::simplify::Simplify;
pub use kurbo::BezPath;
pub use kurbo::Point as BezPoint;
use nalgebra::{Affine2, Point2 as NPoint2};
use serde::{Deserialize, Serialize};

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
    pub(crate) hatch_pattern: Hatches,
    pub(crate) hatch_angle: f64,
    pub(crate) hatch_scale: Option<f64>,
    pub(crate) stroke_filter: Option<Arc<Box<dyn LineFilter>>>,
    pub(crate) hatch_filter: Option<Arc<Box<dyn LineFilter>>>,
}

impl Operation {
    /// Transform content by my transformation
    pub fn transformed(&self, content: &Geometry<f64>) -> Geometry<f64> {
        if let Some(tx) = &self.transformation.clone() {
            // let mut content = content.clone();
            content.map_coords(|xy| Operation::xform_coord(&xy, tx))
        } else {
            content.clone()
        }
    }

    pub fn render(mut self) -> Self {
        if let Some(tx) = &self.transformation {
            self.content = self
                .content
                .map_coords(|xy| Operation::xform_coord(&xy, tx));
        }
        self.content = match &self.mask {
            Some(mask) => {
                let ggeo = try_to_geos_geometry(&self.content).unwrap_or(
                    geos::Geometry::create_empty_collection(GeometryTypes::GeometryCollection)
                        .unwrap(),
                );
                let mggeo = try_to_geos_geometry(mask).unwrap_or(
                    geos::Geometry::create_empty_collection(GeometryTypes::GeometryCollection)
                        .unwrap(),
                );
                let masked_geo = ggeo.intersection(&mggeo).unwrap_or(
                    geos::Geometry::create_empty_collection(GeometryTypes::GeometryCollection)
                        .unwrap(),
                );
                geo_types::Geometry::<f64>::try_from(masked_geo)
                    .unwrap_or(Geometry::GeometryCollection::<f64>(Default::default()))
            }
            None => self.content,
        };

        self.rendered = self.render_to_lines();
        self
    }

    pub fn consistent(&self, other: &Operation) -> bool {
        if self.stroke_color == other.stroke_color
            && self.outline_stroke == other.outline_stroke
            && self.fill_color == other.fill_color
            && self.line_join == other.line_join
            && self.line_cap == other.line_cap
            && self.pen_width == other.pen_width
            && self.hatch_angle == other.hatch_angle
            && self.hatch_scale == other.hatch_scale
            && self.clip_previous == other.clip_previous
        // &&
        {
            true
        } else {
            false
        }
    }

    /// Helper function for converting polygons into sets of strings.
    fn poly2lines(
        poly: &Polygon<f64>,
        pen_width: f64,
        hatch_angle: f64,
        hatch_scale: Option<f64>,
        hatch_pattern: Hatches,
    ) -> (MultiLineString<f64>, MultiLineString<f64>) {
        let mut strokes = MultiLineString::new(vec![]);
        // let mut fills = MultiLineString::new(vec![]);
        // Push the exterior
        strokes.0.push(poly.exterior().clone());
        for interior in poly.interiors() {
            strokes.0.push(interior.clone())
        }
        // let hatch_pattern = hatch_pattern.deref();
        // println!("Hatching with pattern: {:?}", &hatch_pattern);
        let hatches = poly
            .hatch(
                hatch_pattern,
                hatch_angle,
                match hatch_scale {
                    Some(scale) => scale,
                    None => pen_width,
                },
                pen_width,
            )
            .unwrap_or(MultiLineString::new(vec![]));
        // fills.0.append(&mut hatches.0.clone());
        (strokes, hatches)
    }

    /// Helper function for converting multipolygons into sets of strings.
    fn mpoly2lines(
        mpoly: &MultiPolygon<f64>,
        pen_width: f64,
        hatch_angle: f64,
        hatch_scale: Option<f64>,
        hatch_pattern: Hatches,
    ) -> (MultiLineString<f64>, MultiLineString<f64>) {
        let mut strokes = MultiLineString::new(vec![]);
        // let mut fills = MultiLineString::new(vec![]);
        // Push the exterior
        for poly in mpoly {
            strokes.0.push(poly.exterior().clone());
            for interior in poly.interiors() {
                strokes.0.push(interior.clone())
            }
        }
        // let hatch_pattern = hatch_pattern.deref();
        // println!("Hatching with pattern: {:?}", &hatch_pattern);
        let hatches = mpoly
            .hatch(
                hatch_pattern,
                hatch_angle,
                match hatch_scale {
                    Some(scale) => scale,
                    None => pen_width,
                },
                pen_width,
            )
            .unwrap_or(MultiLineString::new(vec![]));
        // fills.0.append(&mut hatches.0.clone());
        (strokes, hatches)
    }

    /// Helper to transform geometry when we have an affine transform set.
    //pub fn xform_coord((x, y): &(f64, f64), affine: &Affine2<f64>) -> (f64, f64) {
    pub fn xform_coord(xy: &Coord, affine: &Affine2<f64>) -> Coord {
        let out = affine * NPoint2::new(xy.x, xy.y);
        coord!(x: out.x, y: out.y)
    }

    fn help_render_geo(
        txgeo: &Geometry<f64>,
        pen_width: f64,
        hatch_angle: f64,
        hatch_scale: Option<f64>,
        hatch_pattern: Hatches,
    ) -> (MultiLineString<f64>, MultiLineString<f64>) {
        match txgeo {
            Geometry::MultiLineString(mls) => (mls.clone(), MultiLineString::new(vec![])),
            Geometry::LineString(ls) => (
                MultiLineString::new(vec![ls.clone()]),
                MultiLineString::new(vec![]),
            ),
            Geometry::Polygon(poly) => Self::poly2lines(
                &poly,
                pen_width,
                hatch_angle,
                hatch_scale,
                hatch_pattern.clone(),
            ),
            Geometry::MultiPolygon(polys) => {
                Self::mpoly2lines(
                    &polys,
                    pen_width,
                    hatch_angle,
                    hatch_scale,
                    hatch_pattern.clone(),
                )
                // let mut strokes = MultiLineString::new(vec![]);
                // let mut fills = MultiLineString::new(vec![]);
                // for poly in polys {
                //     let (new_strokes, new_fills) =
                //         Self::poly2lines(&poly, pen_width, hatch_angle, hatch_pattern.clone());
                //     strokes.0.append(&mut new_strokes.0.clone());
                //     fills.0.append(&mut new_fills.0.clone());
                // }
                // (strokes, fills)
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
                        hatch_scale,
                        hatch_pattern.clone(),
                    );
                    strokes.0.append(tmpstrokes.0.borrow_mut());
                    fills.0.append(tmpfills.0.borrow_mut());
                }
                // println!("Got strokes, fills of: \n{:?}, \n{:?}\n\n", &strokes, &fills);
                (strokes, fills)
            }

            _ => (MultiLineString::new(vec![]), MultiLineString::new(vec![])),
        }
    }

    fn vectorize_flat_geo(geo: &Geometry<f64>) -> Vec<Geometry<f64>> {
        let mut out: Vec<Geometry<f64>> = vec![];
        out.append(&mut match geo {
            Geometry::Point(p) => vec![Geometry::Point(p.clone())],
            Geometry::Line(l) => vec![Geometry::Line(l.clone())],
            Geometry::LineString(ls) => vec![Geometry::LineString(ls.clone())],
            Geometry::Polygon(p) => vec![Geometry::Polygon(p.clone())],
            Geometry::MultiPoint(mp) => vec![Geometry::MultiPoint(mp.clone())],
            Geometry::MultiLineString(mls) => vec![Geometry::MultiLineString(mls.clone())],
            Geometry::MultiPolygon(mp) => vec![Geometry::MultiPolygon(mp.clone())],
            Geometry::GeometryCollection(coll) => coll.iter().map(|gc| gc.to_owned()).collect(),
            Geometry::Rect(r) => vec![Geometry::Rect(r.clone())],
            Geometry::Triangle(t) => vec![Geometry::Triangle(t.clone())],
        });
        out
    }

    pub fn render_to_lines(&self) -> (MultiLineString<f64>, MultiLineString<f64>) {
        // Get the transformed geo, or just this geo at 1:1
        // First let's flatten that shit out.
        let flat_geo = Self::vectorize_flat_geo(&self.content);

        let ofvec: Vec<(MultiLineString<f64>, MultiLineString<f64>)> = flat_geo
            .iter()
            //.par_iter()
            .map(|g| {
                Self::help_render_geo(
                    &g,
                    self.pen_width,
                    self.hatch_angle,
                    self.hatch_scale,
                    self.hatch_pattern.clone(),
                )
            })
            .collect();
        let (mut outlines, mut fills) =
            (MultiLineString::new(vec![]), MultiLineString::new(vec![]));
        for (mut outline, mut fill) in ofvec {
            outlines.0.append(&mut outline.0);
            fills.0.append(&mut fill.0);
        }

        if let Some(filter) = &self.stroke_filter {
            outlines = filter.apply(&outlines);
        }
        if let Some(filter) = &self.hatch_filter {
            fills = filter.apply(&fills);
        }

        // Finally, if we have outline stroke, then outline the existing strokes.
        let outlines = match self.outline_stroke {
            Some(stroke) => outlines
                .outline_fill_stroke_with_hatch(
                    stroke,
                    self.pen_width,
                    Hatches::line(),
                    self.hatch_angle,
                )
                .unwrap_or(outlines),
            None => outlines,
        };
        (
            outlines.simplify(&self.accuracy),
            fills.simplify(&self.accuracy),
        )
    }
}

/// OPLayer is an operation layer, rendered into lines for drawing.
#[derive(Debug, Clone, Serialize, Deserialize)]
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

    pub fn stroke(&self) -> String {
        self.stroke.clone()
    }
    pub fn fill(&self) -> String {
        self.fill.clone()
    }

    pub fn stroke_width(&self) -> f64 {
        self.stroke_width.clone()
    }
}

#[cfg(test)]
pub mod test {
    use geo::LineString;
    use geo_types::Coord;

    use crate::context::line_filter::SketchyLineFilter;

    use super::*;

    #[test]
    pub fn test_stroke_filter() {
        let foo = SketchyLineFilter::new(0.1, 3.);
        // let geo: Geometry<f64> =
        //     Line::new(Coord { x: 0.0, y: 0.0 }, Coord { x: 50.0, y: 50.0 }).into();
        let mls = MultiLineString(vec![LineString::new(vec![
            Coord { x: 0.0, y: 0.0 },
            Coord { x: 50.0, y: 50.0 },
        ])]);
        let new_mls = foo.apply(&mls);
        println!("New MLS: {:?}", &new_mls);
        // let geo = foo.apply(&geo);
        // println!("geo: {:?}", &geo);
    }
}
