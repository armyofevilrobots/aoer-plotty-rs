use crate::geo_types::buffer::Buffer;
use embed_doc_image::embed_doc_image;
use geo::bounding_rect::BoundingRect;
use geo::rotate::Rotate;
use geo::Simplify;
use geo_offset::Offset;
use geo_types::{LineString, MultiLineString, MultiPolygon, Polygon, Rect};
use geos::{Geom, Geometry};
use rayon::iter::ParallelIterator;
use rayon::prelude::IntoParallelRefIterator;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::sync::Arc;

pub mod gototen;
pub use gototen::GotoTenHatch;
pub mod spiral;
pub use spiral::{SpiralDirection, SpiralHatch};
pub mod fasthex;
pub use fasthex::FastHexHatch;
pub mod radius;
pub use radius::RadiusHatch;
pub mod circles;
pub use circles::CircleHatch;
pub mod crosshatch;
pub use crosshatch::CrossHatch;
pub mod line;
pub use line::LineHatch;
pub mod truchet;
pub use truchet::*;
/// There are a variety of hatches documented below, but a quick glance at the image below
/// should help you select which you want to use. If nothing fits, you can either implement
/// your own `impl HatchPattern`, or you can use the truchet hatch to repeat a square pattern
/// to generate your hatch.
///
///

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
    use geo::coord;
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
