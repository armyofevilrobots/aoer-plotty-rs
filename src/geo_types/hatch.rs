use geo_types::{CoordNum, Polygon, MultiPolygon, MultiLineString, Rect, coord};
use geo::bounding_rect::BoundingRect;
// use geo::GeoFloat;
use geo::rotate::Rotate;
// use geos::from_geo;
// use geos::to_geo;
use geos::{Geom, Geometry};
// use geos::GeometryTypes::{GeometryCollection as GGeometryCollection, LineString};
use num_traits::real::Real;
use std::error::Error;
use std::fmt::{Display, Formatter};
use geo_offset::Offset;
use embed_doc_image::embed_doc_image;


/// #InvalidHatchGeometry
/// A bunch of excuses that the hatching traits will throw ;)
/// CouldNotGenerateHatch is just the total failure of the system.
/// InvalidBoundary means that we couldn't create a container boundary for the hatchlines.
/// InvalidResultGeometry means we calculated SOMETHING, but it's irrevocably broken.
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
            InvalidHatchGeometry::CouldNotGenerateHatch =>
                f.write_str("Could not generate hatch"),
            InvalidHatchGeometry::InvalidBoundary =>
                f.write_str("Could not process boundary geometry"),
            InvalidHatchGeometry::InvalidResultGeometry =>
                f.write_str("Processed hatch, but result geometry was invalid"),
        }
    }
}

impl Error for InvalidHatchGeometry {}

/// # HatchPattern
/// Returns a MultiLineString which draws a hatch pattern which fills the entire
/// bbox area. Set up as a trait so the developer can add new patterns at their
/// leisure.
pub trait HatchPattern<T>
    where T: CoordNum, T: Real {
    fn generate(&self, bbox: &Rect<T>, scale: T) -> MultiLineString<T>;
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
/// use geo_types::Polygon;
/// use aoer_plotty_rs::geo_types::hatch::LineHatch;
/// let geoms = Polygon::<f64>::try_from_wkt_str("POLYGON ((350 100, 450 450, 150 400, 100 200, 350 100), (200 300, 350 350, 300 200, 200 300))")
///     .expect("Failed to load box");
/// let hatch = geoms.hatch(LineHatch{}, 45.0, 5.0, 2.5).expect("Got some hatches in here failin'");
/// ```
/// ![hatch-example-1][hatch-example-1]
#[embed_doc_image("hatch-example-1", "images/hatch-demo-1.png")]

pub trait Hatch<P>
    where P: HatchPattern<f64>,
{
    fn hatch(&self, pattern: P, angle: f64, scale: f64, inset: f64)
             -> Result<MultiLineString<f64>, InvalidHatchGeometry>;
}

/// The basic built in parallel LineHatch.
#[derive(Clone)]
pub struct LineHatch {}

impl<T> HatchPattern<T> for LineHatch
    where T: CoordNum,
          T: Real,
          T: std::ops::AddAssign {
    fn generate(&self, bbox: &Rect<T>, scale: T) -> MultiLineString<T> {
        let min = bbox.min();
        let max = bbox.max();
        let mut y = min.y;
        let mut count = 0u32;
        // MultiLineString::<T>::new(
        let mut lines: Vec<geo_types::LineString<T>> = vec![];
        while y < max.y {
            if count % 2 == 0 {
                lines.push(geo_types::LineString::<T>::new(vec![
                    coord! {x: min.x, y: y},
                    coord! {x: max.x, y: y},
                ]));
            } else {
                lines.push(geo_types::LineString::<T>::new(vec![
                    coord! {x: max.x, y: y},
                    coord! {x: min.x, y: y},
                ]));
            }
            y += scale;
            count += 1;
        };
        MultiLineString::<T>::new(lines)
    }
}

/// Internal helper function for flattening a ton of Geometry which contains LineStrings, into
/// a single MultiLineString for drawing on whatever output device we want.
fn gt_flatten_mlines(geo: geo_types::Geometry<f64>, mut existing: MultiLineString<f64>)
                     -> MultiLineString<f64> {
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
        _ => {
            existing
        }
    }
}

impl<P> Hatch<P> for MultiPolygon<f64>
    where P: HatchPattern<f64>,
          P: Clone {
    fn hatch(&self, pattern: P, angle: f64, scale: f64, inset: f64)
             -> Result<MultiLineString<f64>, InvalidHatchGeometry> {
        let mpolys = if inset != 0.0 {
            self.offset(-inset).or(Err(InvalidHatchGeometry::InvalidBoundary))?
        } else {
            self.to_owned()
        };
        let mut mlines = MultiLineString::<f64>::new(vec![]);
        for poly in mpolys {
            let mut tmplines = poly.hatch(pattern.clone(),
                                          angle, scale, 0.0)?;
            mlines.0.append(&mut tmplines.0);
        }
        Ok(mlines)
    }
}

impl<P> Hatch<P> for Polygon<f64>
    where P: HatchPattern<f64>,
          P: Clone {
    fn hatch(&self, pattern: P, angle: f64, scale: f64, inset: f64)
             -> Result<MultiLineString<f64>, InvalidHatchGeometry> {
        let perimeter = if inset != 0.0 {
            let mpolys = self.offset(-inset)
                .or(Err(InvalidHatchGeometry::InvalidBoundary))?;
            return mpolys.hatch(pattern.clone(), angle, scale, 0.0);
        } else {
            self
        };
        let bbox = perimeter.bounding_rect()
            .ok_or(InvalidHatchGeometry::CouldNotGenerateHatch)?
            .to_polygon().rotate_around_centroid(angle).bounding_rect()
            .ok_or(InvalidHatchGeometry::CouldNotGenerateHatch)?;
        let geo_perimeter: geos::Geometry = self.try_into()
            .or(Err(InvalidHatchGeometry::InvalidBoundary))?;
        let hatch_lines: Vec<geo_types::LineString<f64>> = pattern
            .generate(&bbox, scale)
            .rotate_around_centroid(angle)
            .iter().map(|x| x.to_owned())
            .collect();
        let geo_hatchlines = Geometry::create_geometry_collection(
            hatch_lines.iter().map(|hatch_line| {
                hatch_line.clone()
                    .try_into()
                    .expect("Invalid hatch lines")
            })
                .collect())
            .or(Err(InvalidHatchGeometry::CouldNotGenerateHatch))?;
        let hatched_object = geo_perimeter
            .intersection(&geo_hatchlines)
            .or(Err(InvalidHatchGeometry::CouldNotGenerateHatch))?;
        let out: geo_types::Geometry<f64> = hatched_object
            .try_into()
            .or(Err(InvalidHatchGeometry::InvalidResultGeometry))?;
        let out = gt_flatten_mlines(out,
                                    MultiLineString::new(vec![]));
        Ok(out)
    }
}


#[cfg(test)]
mod test {
    use std::f64::consts::PI;
    use geos::Geometry;
    use super::*;

    #[test]
    fn test_box_hatch() {
        let rect = Rect::<f64>::new(
            coord! {x: 0.0, y: 0.0},
            coord! {x: 100.0, y: 100.0});
        let hatch_lines = LineHatch {}.generate(&rect, 10.0);
        println!("LINES HATCHED: {:?}", hatch_lines);
    }

    #[test]
    fn test_experiment1_geos_clip_hatch() {
        let rect = Rect::<f64>::new(
            coord! {x: -100.0, y: -100.0},
            coord! {x: 100.0, y: 100.0});
        let hatch_lines = LineHatch {}.generate(&rect, 5.0);
        let poly = Polygon::<f64>::new(
            geo_types::LineString::<f64>::new(vec![
                coord! {x: 0.0, y: 20.0},
                coord! {x: 20.0, y: 0.0},
                coord! {x: 0.0, y: -20.0},
                coord! {x: -20.0, y: 0.0},
                coord! {x: 0.0, y: 20.0},
            ]),
            vec![]);
        let geo_perimeter: geos::Geometry = (&poly).try_into().expect("Invalid geometry");
        let hatch_lines: Vec<geo_types::LineString<f64>> = hatch_lines.iter().map(|x| x.clone()).collect();
        let geo_hatchlines: Vec<Geometry> = (&hatch_lines).iter()
            .map(|hatch_line|
                (hatch_line).try_into().expect("Invalid hatch lines")).collect();
        let geo_hatchlines = Geometry::create_geometry_collection(geo_hatchlines).expect("Got this far?");
        let _hatched_object = geo_perimeter.intersection(&geo_hatchlines).expect("Got this far?");
        // println!("Hatched object is: {}", _hatched_object.to_wkt().expect("As a string!"))
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
            vec![]);
        let hatches = poly.hatch(LineHatch {}, 0.0, 5.0, 0.0).expect("Failed to Ok the hatches.");
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
            vec![]);
        let hatches = poly.hatch(LineHatch {}, PI / 4.0, 5.0, 0.0).expect("Failed to Ok the hatches.");
        println!("Angle-Hatched object is: {:?}", hatches);
        // assert!(hatches.0.len() == 7);
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
            vec![]);
        let poly2 = Polygon::<f64>::new(
            geo_types::LineString::<f64>::new(vec![
                coord! {x:40.0, y: 20.0},
                coord! {x: 60.0, y: 0.0},
                coord! {x: 40.0, y: -20.0},
                coord! {x: 20.0, y: 0.0},
                coord! {x: 40.0, y: 20.0},
            ]),
            vec![]);
        let mpoly = MultiPolygon::<f64>::new(vec![poly1, poly2]);
        let _hatches = mpoly.hatch(LineHatch {}, 0.0, 5.0, 0.0)
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
            vec![]);
        let hatches = poly.hatch(LineHatch {},
                                 0.0, 5.0, 2.0)
            .expect("Disjoint hatch failed");
        // println!("Got these {} disjointed after inset lines: {:?}", &hatches.0.len(), &hatches);
        assert!((&hatches).0.len() == 12);
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
            vec![]);
        let hatches = poly.hatch(LineHatch {},
                                 PI / 4.0, 5.0, 2.0)
            .expect("Disjoint hatch failed");
        // println!("Got these {} angled then disjointed after inset lines: {:?}", &hatches.0.len(), &hatches);
        assert!((&hatches).0.len() == 12);
    }
}