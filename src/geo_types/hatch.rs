use crate::geo_types::buffer::Buffer;
use crate::geo_types::shapes::circle;
use embed_doc_image::embed_doc_image;
use geo::bounding_rect::BoundingRect;
use geo::rotate::Rotate;
use geo::{Coord, Geometry as GeoGeometry, Simplify};
use geo_offset::Offset;
use geo_types::{coord, LineString, MultiLineString, MultiPolygon, Polygon, Rect};
use geos::{Geom, Geometry};
use rand::prelude::*;
use rayon::prelude::IntoParallelRefIterator;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::f64::consts::PI;
use std::fmt::{Debug, Display, Formatter};
use std::sync::Arc;

/// Useful for converting a line into a polygon as if it were stroked. Only supports
/// round caps and joins for now.
pub trait OutlineStroke {
    fn outline_stroke(&self, stroke_weight: f64) -> Result<MultiPolygon<f64>, Box<dyn Error>>;
}

impl OutlineStroke for MultiLineString<f64> {
    fn outline_stroke(&self, stroke_weight: f64) -> Result<MultiPolygon<f64>, Box<dyn Error>> {
        geo_types::Geometry::MultiLineString(self.clone()).buffer(stroke_weight / 2.0)
    }
}

impl OutlineStroke for LineString<f64> {
    fn outline_stroke(&self, stroke_weight: f64) -> Result<MultiPolygon<f64>, Box<dyn Error>> {
        geo_types::Geometry::LineString(self.clone()).buffer(stroke_weight / 2.0)
    }
}

/// Turns out that one of the most common things we do to a line is to stroke it with a weight,
/// turning  it into a series of outline LineStrings, which are in turn filled with a hatch.
/// This trait combines those into a simple single operation.
pub trait OutlineFillStroke {
    fn outline_fill_stroke_with_hatch(
        &self,
        stroke_weight: f64,
        pen_width: f64,
        pattern: Arc<Box<dyn HatchPattern>>, //Hatches,
        angle: f64,
    ) -> Result<MultiLineString<f64>, Box<dyn Error>>;
}

impl OutlineFillStroke for MultiLineString<f64> {
    fn outline_fill_stroke_with_hatch(
        &self,
        stroke_weight: f64,
        pen_width: f64,
        pattern: Arc<Box<dyn HatchPattern>>, //Hatches,
        angle: f64,
    ) -> Result<MultiLineString<f64>, Box<dyn Error>> {
        let polys = self.outline_stroke(stroke_weight)?;
        let mut lines_list: MultiLineString<f64> =
            MultiLineString::new(polys.0.iter().map(|p| p.exterior().clone()).collect());
        for poly in &polys {
            for interior in poly.interiors() {
                lines_list.0.push(interior.clone())
            }
        }

        lines_list
            .0
            .append(&mut polys.hatch(pattern, angle, pen_width, pen_width * 0.5)?.0);
        Ok(lines_list)
    }
}

/// #InvalidHatchGeometry
/// A bunch of excuses that the hatching traits will throw ;)
/// CouldNotGenerateHatch is just the total failure of the system.
/// InvalidBoundary means that we couldn't create a container boundary for the hatchlines.
/// InvalidResultGeometry means we calculated SOMEHING, but it's irrevocably broken.
#[derive(Debug)]
pub enum InvalidHatchGeometry {
    CouldNotGenerateHatch,
    InvalidBoundary,
    InvalidResultGeometry,
}

/// Display is required for these InvalidHatchGeometry Errors to be used in our Result fields.
impl Display for InvalidHatchGeometry {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match *self {
            InvalidHatchGeometry::CouldNotGenerateHatch => f.write_str("Could not generate hatch"),
            InvalidHatchGeometry::InvalidBoundary => {
                f.write_str("Could not process boundary geometry")
            }
            InvalidHatchGeometry::InvalidResultGeometry => {
                f.write_str("Processed hatch, but result geometry was invalid")
            }
        }
    }
}

impl Error for InvalidHatchGeometry {}

/// # HatchPattern
/// Returns a MultiLineString which draws a hatch pattern which fills the entire
/// bbox area. Set up as a trait so the developer can add new patterns at their
/// leisure.
pub trait HatchPattern: Debug + Send + Sync {
    fn generate(&self, bbox: &Rect<f64>, scale: f64, pen: f64) -> MultiLineString<f64>;
}

/// # Hatch
/// Trait which can be implemented for various geo_types, to provide fills
/// on their interiors. Requires an instance of a Pattern type &lt;P&gt;, which
/// will be used to generate the hatch lines. Angle is the angle to rotate
/// the hatch pattern in Radians. Scale is the distance between lines (although
/// other Pattern types may not honor this, and generate alternative based scales
/// for the 1.0 value. Inset is the distance to inset the boundary before filling,
/// and it's a good idea to inset by approximately the scale value to keep the line
/// endpoints inside of the boundary/container.
///
/// # Example hatching
/// ```rust
///
/// use std::sync::Arc;
/// use aoer_plotty_rs::geo_types::hatch::LineHatch;
/// use geo_types::Polygon;
/// use wkt::{TryFromWkt, Wkt, geo_types_from_wkt} ;
/// use aoer_plotty_rs::geo_types::hatch::{Hatch};
/// use std::str::FromStr;
/// let geoms: Polygon::<f64> = Polygon::try_from(
///     Wkt::<f64>::from_str("POLYGON ((350 100, 450 450, 150 400, 100 200, 350 100), (200 300, 350 350, 300 200, 200 300))")
///         .expect("Failed to load WKT"))
///     .expect("Failed to load box");
/// let hatch = geoms.hatch(Arc::new(Box::new(LineHatch{})), 45.0, 5.0, 2.5).expect("Got some hatches in here failin'");
/// ```
/// ![hatch-example-1][hatch-example-1]
#[embed_doc_image("hatch-example-1", "images/hatch-demo-1.png")]
pub trait Hatch {
    fn hatch(
        &self,
        pattern: Arc<Box<dyn HatchPattern>>,
        angle: f64,
        scale: f64,
        pen: f64,
    ) -> Result<MultiLineString<f64>, InvalidHatchGeometry>;
}

/// The no-hatch option
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
pub struct NoHatch {}

impl NoHatch {
    pub fn gen() -> Arc<Box<dyn HatchPattern>> {
        Arc::new(Box::new(Self::default()))
    }
}

impl HatchPattern for NoHatch {
    fn generate(&self, _bbox: &Rect<f64>, _scale: f64, _pen: f64) -> MultiLineString<f64> {
        MultiLineString::new(vec![])
    }
}

/// The basic built in parallel LineHatch.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
pub struct LineHatch {}

impl LineHatch {
    pub fn gen() -> Arc<Box<dyn HatchPattern>> {
        Arc::new(Box::new(Self::default()))
    }
}

impl HatchPattern for LineHatch {
    fn generate(&self, bbox: &Rect<f64>, scale: f64, _pen: f64) -> MultiLineString<f64> {
        let min = bbox.min();
        let max = bbox.max();
        let mut y = min.y;
        let mut count = 0u32;
        // MultiLineString::<T>::new(
        let mut lines: Vec<geo_types::LineString<f64>> = vec![];
        while y < max.y {
            if count % 2 == 0 {
                lines.push(geo_types::LineString::<f64>::new(vec![
                    coord! {x: min.x, y: y},
                    coord! {x: max.x, y: y},
                ]));
            } else {
                lines.push(geo_types::LineString::<f64>::new(vec![
                    coord! {x: max.x, y: y},
                    coord! {x: min.x, y: y},
                ]));
            }
            y += scale;
            count += 1;
        }
        let out = MultiLineString::<f64>::new(lines);
        out
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
pub struct CrossHatch {}

impl CrossHatch {
    pub fn gen() -> Arc<Box<dyn HatchPattern>> {
        Arc::new(Box::new(Self::default()))
    }
}

impl HatchPattern for CrossHatch {
    fn generate(&self, bbox: &Rect<f64>, scale: f64, _pen: f64) -> MultiLineString<f64> {
        let min = bbox.min();
        let max = bbox.max();
        let mut y = min.y;
        let mut count = 0u32;
        let mut lines: Vec<geo_types::LineString<f64>> = vec![];
        while y < max.y {
            if count % 2 == 0 {
                lines.push(geo_types::LineString::<f64>::new(vec![
                    coord! {x: min.x, y: y},
                    coord! {x: max.x, y: y},
                ]));
            } else {
                lines.push(geo_types::LineString::<f64>::new(vec![
                    coord! {x: max.x, y: y},
                    coord! {x: min.x, y: y},
                ]));
            }
            y += scale;
            count += 1;
        }
        let mut x = min.x;
        count = 0u32;
        while x < max.x {
            if count % 2 == 0 {
                lines.push(geo_types::LineString::<f64>::new(vec![
                    coord! {x: x, y: min.y},
                    coord! {x: x, y: max.y},
                ]));
            } else {
                lines.push(geo_types::LineString::<f64>::new(vec![
                    coord! {x: x, y: max.y},
                    coord! {x: x, y: min.y},
                ]));
            }
            x += scale;
            count += 1;
        }
        //println!("HATCH LINES ARE: {:?}", &lines);
        MultiLineString::<f64>::new(lines)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct RadiusHatch {
    pub x: f64,
    pub y: f64,
}

impl RadiusHatch {
    pub fn gen() -> Arc<Box<dyn HatchPattern>> {
        Arc::new(Box::new(Self::default()))
    }
}

impl HatchPattern for RadiusHatch {
    fn generate(&self, bbox: &Rect<f64>, scale: f64, _pen: f64) -> MultiLineString {
        let (x1, y1) = bbox.min().x_y();
        let (x2, y2) = bbox.max().x_y();
        // println!("Center for bbox is: {:?}", bbox.center());
        let mut max_radius = 0.0f64;
        let mut min_radius = scale / 2.; //f64::MAX;
        for (x, y) in vec![(x1, y1), (x2, y1), (x2, y2), (x1, y2)] {
            let tmp_rad = ((x - self.x).powi(2) + (y - self.y).powi(2)).sqrt();
            if tmp_rad > max_radius {
                max_radius = tmp_rad;
            }
            if tmp_rad < min_radius {
                min_radius = tmp_rad;
            }
        }
        let mut lines: Vec<LineString> = vec![];
        let mut r = min_radius;
        while r < max_radius {
            let c = circle(self.x, self.y, r);
            if let GeoGeometry::Polygon(tmp_lines) = c.into() {
                lines.push(tmp_lines.exterior().clone());
            }
            r += scale;
        }
        // println!("Lines for radius fill are: {:?}", &lines);

        MultiLineString::<f64>::new(lines)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub enum SpiralDirection {
    #[default]
    Deasil,
    Widdershins,
}

pub const CLOCKWISE: SpiralDirection = SpiralDirection::Deasil;
pub const COUNTERCLOCKWISE: SpiralDirection = SpiralDirection::Widdershins;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct SpiralHatch {
    pub x: f64,
    pub y: f64,
    pub direction: SpiralDirection,
}

impl SpiralHatch {
    pub fn gen() -> Arc<Box<dyn HatchPattern>> {
        Arc::new(Box::new(Self::default()))
    }
}

impl HatchPattern for SpiralHatch {
    fn generate(&self, bbox: &Rect<f64>, scale: f64, _pen: f64) -> MultiLineString {
        let (x1, y1) = bbox.min().x_y();
        let (x2, y2) = bbox.max().x_y();
        // println!("Center for bbox is: {:?}", bbox.center());
        let mut max_radius = 0.0f64;
        let mut min_radius = scale / 2.; //f64::MAX;
        for (x, y) in vec![(x1, y1), (x2, y1), (x2, y2), (x1, y2)] {
            let tmp_rad = ((x - self.x).powi(2) + (y - self.y).powi(2)).sqrt();
            if tmp_rad > max_radius {
                max_radius = tmp_rad;
            }
            if tmp_rad < min_radius {
                min_radius = tmp_rad;
            }
        }
        let mut points: Vec<Coord> = vec![];
        let mut r = min_radius;
        let mut theta = 0.0_f64;

        while r < max_radius {
            // This keeps segment length around PI mm or less.
            let ainc = (2. * PI / 16.).min(PI / r);
            // println!("AINC: {}, YAINC: {}", ainc, ainc.sin() * r);
            let r1 = r + (ainc / (2. * PI)) * scale;
            let x = self.x + theta.cos() * r;
            let y = self.y + theta.sin() * r;
            theta = theta
                + if self.direction == SpiralDirection::Widdershins {
                    ainc
                } else {
                    -ainc
                };
            points.push(Coord { x: x, y: y });
            r = r1;
        }
        // println!("Lines for radius fill are: {:?}", &lines);

        MultiLineString::<f64>::new(vec![LineString::new(points)])
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
pub struct CircleHatch {}

impl CircleHatch {
    pub fn gen() -> Arc<Box<dyn HatchPattern>> {
        Arc::new(Box::new(Self::default()))
    }
}

impl HatchPattern for CircleHatch {
    fn generate(&self, bbox: &Rect<f64>, scale: f64, pen: f64) -> MultiLineString {
        let (x1, y1) = bbox.min().x_y();
        let (x2, y2) = bbox.max().x_y();
        let mut lines: Vec<LineString> = vec![];
        let mut ix: usize = 0;
        let mut x = x1 - 2. * scale;
        let r2 = 2.0_f64.sqrt();
        while x < x2 + 2. * (scale + pen) {
            let (ofsx, ofsy) = if ix % 2 == 0 {
                (scale - pen, -(scale + pen * r2) * r2)
            } else {
                //(-(scale - pen), 0.)
                (-scale + pen / 2., 0.)
            };

            let mut y = y1 - scale;
            while y < y2 + 2. * (scale + pen) {
                // println!("Adding circle at x:{}, y:{} @ofs:{:?}", x, y, (ofsx, ofsy));
                let c = circle(x + ofsx, y + ofsy, scale / 2.);
                if let GeoGeometry::Polygon(tmp_lines) = c.into() {
                    // Tricky. We reverse every other circle for better back and forth
                    // optimization of hatch drawing.
                    if ix % 2 == 0 {
                        lines.push(tmp_lines.exterior().clone());
                    } else {
                        let mut tmp_lines = tmp_lines.exterior().clone();
                        tmp_lines.0.reverse();
                        lines.push(tmp_lines)
                    }
                }
                y += scale + pen;
            }
            ix += 1;
            x += scale - (pen * r2 / 2.);
        }
        // println!("Lines for radius fill are: {:?}", &lines);

        MultiLineString::<f64>::new(lines)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
pub struct FastHexHatch {}

impl FastHexHatch {
    pub fn gen() -> Arc<Box<dyn HatchPattern>> {
        Arc::new(Box::new(Self::default()))
    }
}

impl HatchPattern for FastHexHatch {
    fn generate(&self, bbox: &Rect<f64>, scale: f64, _pen: f64) -> MultiLineString {
        let (x1, y1) = bbox.min().x_y();
        let (x2, y2) = bbox.max().x_y();
        let mut lines: Vec<LineString> = vec![];
        let mut ix: usize = 0;
        let mut x = x1 - 2. * scale;
        let sidelen = scale / 2.;
        let rin = scale * (PI / 6.).cos() / 2.;
        // println!("SIDELEN: {} , RIN: {} , ", sidelen, rin);

        while x <= x2 + scale {
            let (inc, mul) = if ix % 2 == 0 {
                (rin * 2., 1.)
            } else {
                (0.0, -1.)
            };
            let mut y = y1 - 2. * scale;
            let mut aline: LineString<f64> = LineString::new(vec![]);
            while y <= y2 + scale {
                // y2 + 2. * scale {
                aline.0.push(coord! {x:x, y:y});
                aline.0.push(coord! {x:x+mul*rin, y: y+sidelen/2.});
                aline.0.push(coord! {x:x+mul*rin, y: y+sidelen/2.+sidelen});
                aline.0.push(coord! {x:x, y: y+scale});
                aline.0.push(coord! {x:x, y: y+scale+sidelen});
                y = y + scale + sidelen;
            }
            // println!("ALINE: {:?}", &aline);
            if ix % 2 != 0 {
                aline.0.reverse();
            }
            lines.push(aline);
            ix += 1;
            x += inc + 0.00001; // We do slightly over the inc, to ensure no overlapping lines.
        }
        // println!("Lines for radius fill are: {:?}", &lines);

        MultiLineString::<f64>::new(lines)
    }
}

/// This is a hatch pattern that is reminiscent of the old C64 program
/// that looked kinda like this:
/// \/\\\/\
/// /\//\/\
/// \\\//\/
/// ie:
/// 10 PRINT CHR$(205.5+RND(1)); : GOTO 10
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
pub struct GotoTenHatch {
    seed: u64,
}

impl GotoTenHatch {
    pub fn gen(seed: u64) -> Arc<Box<dyn HatchPattern>> {
        Arc::new(Box::new(GotoTenHatch { seed }))
    }
}

impl HatchPattern for GotoTenHatch {
    fn generate(&self, bbox: &Rect<f64>, scale: f64, _pen: f64) -> MultiLineString<f64> {
        let min = bbox.min();
        let max = bbox.max();
        let mut y = min.y - scale;
        let mut rng = rand::rngs::SmallRng::seed_from_u64(self.seed);
        // MultiLineString::<T>::new(
        let mut lines: Vec<geo_types::LineString<f64>> = vec![];

        while y < max.y + scale {
            let mut x = min.x - scale;
            while x < max.x + scale {
                if rng.gen_bool(0.5) {
                    lines.push(geo_types::LineString::<f64>::new(vec![
                        coord! {x: x, y: y},
                        coord! {x: x+scale, y: y+scale},
                    ]));
                } else {
                    lines.push(geo_types::LineString::<f64>::new(vec![
                        coord! {x: x, y: y+scale},
                        coord! {x: x+scale, y: y},
                    ]));
                }
                x += scale
            }
            y += scale;
        }
        MultiLineString::<f64>::new(lines)
    }
}

/// Internal helper function for flattening a ton of Geometry which contains LineStrings, into
/// a single MultiLineString for drawing on whatever output device we want.
fn gt_flatten_mlines(
    geo: geo_types::Geometry<f64>,
    mut existing: MultiLineString<f64>,
) -> MultiLineString<f64> {
    match geo {
        geo_types::Geometry::GeometryCollection::<f64>(new_geo) => {
            for geo in new_geo {
                existing = gt_flatten_mlines(geo, existing)
            }
            existing
        }
        geo_types::Geometry::MultiLineString::<f64>(mut ms) => {
            existing.0.append(&mut ms.0);
            existing
        }
        geo_types::Geometry::LineString::<f64>(ls) => {
            existing.0.push(ls);
            existing
        }
        _ => existing,
    }
}

impl Hatch for MultiPolygon<f64> {
    fn hatch(
        &self,
        pattern: Arc<Box<dyn HatchPattern>>, //Hatches,
        angle: f64,
        scale: f64,
        pen: f64,
    ) -> Result<MultiLineString<f64>, InvalidHatchGeometry> {
        // let mpolys = if pen != 0.0 {
        //     self.offset(-pen)
        //         .or(Err(InvalidHatchGeometry::InvalidBoundary))?
        // } else {
        //     self.to_owned()
        // };
        let mpolys = self.to_owned();
        let mut mlines = MultiLineString::<f64>::new(vec![]);

        // for poly in mpolys {
        //     let mut tmplines = poly.hatch(pattern.clone(), angle, scale, 0.0)?;
        //     mlines.0.append(&mut tmplines.0);
        // }
        let hatchlines: Vec<Result<MultiLineString<f64>, InvalidHatchGeometry>> = mpolys
            .0
            .par_iter()
            //.map(|p| p.hatch(pattern.clone(), angle, scale, scale.max(inset)))
            .map(|p| p.hatch(pattern.clone(), angle, scale, scale.min(pen)))
            .collect();
        // let mut out = MultiLineString::<f64>::new(vec![]);

        for result in hatchlines {
            match result {
                Ok(mls) => mlines.0.append(&mut mls.0.clone()),
                _ => (),
            }
        }

        Ok(mlines)
    }
}

pub fn dirty_inset(mls_geo: &mut geo_types::Geometry<f64>, inset: f64) {
    // Only works for MultiLineString, eh?
    match mls_geo {
        geo_types::Geometry::MultiLineString(mls) => {
            // If this MultiLineString only has 2 or fewer entries, just forget the whole thing.
            if mls.0.len() < 3 {
                mls.0 = vec![];
            } else {
                // We assume that the first/last are superfluous linestrings.
                // Also, the 0 LineString definitely overlaps the 'top' of the item,
                // so we have to remove that one, but the next one is adjacent, and usually
                // too far up, so that one goes too.
                mls.0 = mls.0[0..mls.0.len()]
                    .into_iter()
                    // Skip invalid linestrings
                    .filter(|ls| ls.0.len() >= 2)
                    .map(|ls| {
                        let ls_vec = ls.0[1] - ls.0[0];
                        let ls_vec_len = (ls_vec.x.powi(2) + ls_vec.y.powi(2)).sqrt();
                        // Don't draw too-short hatches
                        if ls_vec_len < (2. * inset) {
                            return LineString::new(vec![]);
                        }
                        let ls_vec = ls_vec / ls_vec_len;
                        let out = LineString::new(vec![
                            ls.0[0] + (ls_vec * inset),
                            ls.0[1] - (ls_vec * inset),
                        ]);
                        out
                    })
                    .collect()
            }
        }
        _ => (),
    }
}

impl Hatch for Polygon<f64> {
    fn hatch(
        &self,
        pattern: Arc<Box<dyn HatchPattern>>, //Hatches,
        angle: f64,
        scale: f64,
        pen: f64,
    ) -> Result<MultiLineString<f64>, InvalidHatchGeometry> {
        // let _perimeter = if inset != 0.0 {
        //     let mpolys = self
        //         .offset(-inset)
        //         .or(Err(InvalidHatchGeometry::InvalidBoundary))?;
        //     return mpolys.hatch(pattern.clone(), angle, scale, 0.0);
        // } else {
        //     self
        // };
        // HAHAHAHA Holyshit
        // TODO: Contract the BOUNDING BOX for the original generation of the hatch lines
        let _perimeter = self;
        let bbox = self
            .bounding_rect()
            .ok_or(InvalidHatchGeometry::CouldNotGenerateHatch)?
            .to_polygon()
            .rotate_around_centroid(angle)
            .bounding_rect()
            .ok_or(InvalidHatchGeometry::CouldNotGenerateHatch)?;

        // TODO: shorten each line we generate by the offset on each end
        //       make sure it's not a zero length line afterwards.
        // let hatch_lines_raw = pattern.generate(&bbox, scale);

        let hatch_lines: Vec<geo_types::LineString<f64>> = pattern
            .generate(&bbox, scale, pen)
            .rotate_around_centroid(angle)
            .iter()
            .map(|x| x.to_owned())
            .collect();
        if hatch_lines.is_empty() {
            return Ok(MultiLineString::new(vec![]));
        }

        let inset_perimeter = self
            .simplify(&(pen / 2.))
            .offset(-pen)
            .expect("Failed to inset polygon"); //.unwrap_or(self.clone());

        let geo_perimeter: geos::Geometry = inset_perimeter //self
            .try_into()
            .or(Err(InvalidHatchGeometry::InvalidBoundary))?;

        // println!("PRE INSET: {:?}", &hatch_lines);
        let geo_hatchlines = Geometry::create_geometry_collection(
            hatch_lines
                .par_iter()
                .map(|hatch_line| hatch_line.clone().try_into().expect("Invalid hatch lines"))
                .collect(),
        )
        .or(Err(InvalidHatchGeometry::CouldNotGenerateHatch))?;

        let hatched_object = geo_perimeter
            .intersection(&geo_hatchlines)
            .or(Err(InvalidHatchGeometry::CouldNotGenerateHatch))?;

        // println!(
        //     "Out Hatch lines geo is {:?}",
        //     hatched_object.to_wkt().unwrap()
        // );
        let out: geo_types::Geometry<f64> = hatched_object
            .try_into()
            .or(Err(InvalidHatchGeometry::InvalidResultGeometry))?;
        //dirty_inset(&mut out, scale.max(inset)); // Mutates in place.

        // println!("Out Hatch lines geo is {:?} with inset {}", out, inset);

        //dirty_inset(&mut out, scale.min(inset)); // Mutates in place.
        let out = gt_flatten_mlines(out, MultiLineString::new(vec![]));
        Ok(out)
    }
}

#[cfg(test)]
mod test {
    use super::Hatch;
    use super::*;
    use geos::Geometry;
    use std::f64::consts::PI;

    #[test]
    fn test_box_hatch() {
        let rect = Rect::<f64>::new(coord! {x: 0.0, y: 0.0}, coord! {x: 100.0, y: 100.0});
        //let hatch_lines =
        LineHatch {}.generate(&rect, 10.0, 1.0);
    }

    #[test]
    fn test_experiment1_geos_clip_hatch() {
        let rect = Rect::<f64>::new(coord! {x: -100.0, y: -100.0}, coord! {x: 100.0, y: 100.0});
        let hatch_lines = LineHatch {}.generate(&rect, 5.0, 1.0);
        let poly = Polygon::<f64>::new(
            geo_types::LineString::<f64>::new(vec![
                coord! {x: 0.0, y: 20.0},
                coord! {x: 20.0, y: 0.0},
                coord! {x: 0.0, y: -20.0},
                coord! {x: -20.0, y: 0.0},
                coord! {x: 0.0, y: 20.0},
            ]),
            vec![],
        );
        let geo_perimeter: geos::Geometry = (&poly).try_into().expect("Invalid geometry");
        let hatch_lines: Vec<geo_types::LineString<f64>> =
            hatch_lines.iter().map(|x| x.clone()).collect();
        let geo_hatchlines: Vec<Geometry> = (&hatch_lines)
            .iter()
            .map(|hatch_line| (hatch_line).try_into().expect("Invalid hatch lines"))
            .collect();
        let geo_hatchlines =
            Geometry::create_geometry_collection(geo_hatchlines).expect("Got this far?");
        let _hatched_object = geo_perimeter
            .intersection(&geo_hatchlines)
            .expect("Got this far?");
    }

    #[test]
    fn test_trait_hatch_poly() {
        let poly = Polygon::<f64>::new(
            geo_types::LineString::<f64>::new(vec![
                coord! {x: 0.0, y: 20.0},
                coord! {x: 20.0, y: 0.0},
                coord! {x: 0.0, y: -20.0},
                coord! {x: -20.0, y: 0.0},
                coord! {x: 0.0, y: 20.0},
            ]),
            vec![],
        );
        let hatches = poly
            //.hatch(Hatches::line(), 0.0, 5.0, 0.0)
            .hatch(Arc::new(Box::new(LineHatch {})), 0.0, 5.0, 0.0)
            .expect("Failed to Ok the hatches.");
        //println!("Hatched object is: {:?}", hatches);
        assert!(hatches.0.len() == 7);
    }

    #[test]
    fn test_trait_hatch_poly_angle() {
        let poly = Polygon::<f64>::new(
            geo_types::LineString::<f64>::new(vec![
                coord! {x: 0.0, y: 20.0},
                coord! {x: 20.0, y: 0.0},
                coord! {x: 0.0, y: -20.0},
                coord! {x: -20.0, y: 0.0},
                coord! {x: 0.0, y: 20.0},
            ]),
            vec![],
        );
        let hatches = poly
            .hatch(Arc::new(Box::new(LineHatch {})), PI / 4.0, 5.0, 0.0)
            .expect("Failed to Ok the hatches.");
        // println!("Angle-Hatched object is: {:?}", hatches);
        assert_eq!(hatches.0.len(), 8);
    }

    #[test]
    fn test_trait_hatch_disjoint() {
        let poly1 = Polygon::<f64>::new(
            geo_types::LineString::<f64>::new(vec![
                coord! {x: 0.0, y: 20.0},
                coord! {x: 20.0, y: 0.0},
                coord! {x: 0.0, y: -20.0},
                coord! {x: -20.0, y: 0.0},
                coord! {x: 0.0, y: 20.0},
            ]),
            vec![],
        );
        let poly2 = Polygon::<f64>::new(
            geo_types::LineString::<f64>::new(vec![
                coord! {x:40.0, y: 20.0},
                coord! {x: 60.0, y: 0.0},
                coord! {x: 40.0, y: -20.0},
                coord! {x: 20.0, y: 0.0},
                coord! {x: 40.0, y: 20.0},
            ]),
            vec![],
        );
        let mpoly = MultiPolygon::<f64>::new(vec![poly1, poly2]);
        let _hatches = mpoly
            .hatch(Arc::new(Box::new(LineHatch {})), 0.0, 5.0, 0.0)
            .expect("Disjoint hatch failed");
        // println!("Disjoint hatch {:?}", hatches);
    }

    #[test]
    fn test_polygon_inset_disjoint() {
        let poly = Polygon::<f64>::new(
            geo_types::LineString::<f64>::new(vec![
                coord! {x: 0.0, y: 20.0},
                coord! {x: 19.0, y: 1.0},
                coord! {x: 21.0, y: 1.0},
                coord! {x:40.0, y: 20.0},
                coord! {x: 60.0, y: 0.0},
                coord! {x: 40.0, y: -20.0},
                coord! {x: 20.0, y: 0.0},
                coord! {x: 0.0, y: -20.0},
                coord! {x: -20.0, y: 0.0},
                coord! {x: 0.0, y: 20.0},
            ]),
            vec![],
        );
        let hatches = poly
            .hatch(Arc::new(Box::new(LineHatch {})), 0.0, 5.0, 2.0)
            .expect("Disjoint hatch failed");
        println!(
            "Got these {} disjointed after inset lines: {:?}",
            &hatches.0.len(),
            &hatches
        );
        assert!((&hatches).0.len() == 14);
    }

    #[test]
    fn test_polygon_inset_disjoint_angle() {
        let poly = Polygon::<f64>::new(
            geo_types::LineString::<f64>::new(vec![
                coord! {x: 0.0, y: 20.0},
                coord! {x: 19.0, y: 1.0},
                coord! {x: 21.0, y: 1.0},
                coord! {x:40.0, y: 20.0},
                coord! {x: 60.0, y: 0.0},
                coord! {x: 40.0, y: -20.0},
                coord! {x: 20.0, y: 0.0},
                coord! {x: 0.0, y: -20.0},
                coord! {x: -20.0, y: 0.0},
                coord! {x: 0.0, y: 20.0},
            ]),
            vec![],
        );
        let hatches = poly
            .hatch(Arc::new(Box::new(LineHatch {})), PI / 4.0, 5.0, 2.0)
            .expect("Disjoint hatch failed");
        // println!(
        //     "Got these {} angled then disjointed after inset lines: {:?}",
        //     &hatches.0.len(),
        //     &hatches
        // );
        assert!((&hatches).0.len() == 14);
    }
}
