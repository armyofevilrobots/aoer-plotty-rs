use geo_types::{Polygon, MultiPolygon, MultiLineString, Rect, coord, LineString};
use geo::bounding_rect::BoundingRect;
use geo::rotate::Rotate;
use geos::{Geom, Geometry};
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::rc::Rc;
use geo_offset::Offset;
use embed_doc_image::embed_doc_image;
use crate::geo_types::buffer::Buffer;


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
    fn outline_fill_stroke_with_hatch(&self, stroke_weight: f64, pen_width: f64, pattern: &dyn HatchPattern, angle: f64)
                                      -> Result<MultiLineString<f64>, Box<dyn Error>>;
}

impl OutlineFillStroke for MultiLineString<f64> {
    fn outline_fill_stroke_with_hatch(&self, stroke_weight: f64, pen_width: f64, pattern: &dyn HatchPattern, angle: f64)
                                      -> Result<MultiLineString<f64>, Box<dyn Error>> {
        let polys = self.outline_stroke(stroke_weight)?;
        let mut lines_list: MultiLineString<f64> = MultiLineString::new(
            polys
                .0.iter().map(|p| p.exterior().clone()).collect());
        for poly in &polys {
            for interior in poly.interiors() {
                lines_list.0.push(interior.clone())
            }
        }

        lines_list.0.append(
            &mut polys.hatch(pattern, angle, pen_width, pen_width * 0.5)?.0);
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
pub trait HatchPattern{
    fn generate(&self, bbox: &Rect<f64>, scale: f64) -> MultiLineString<f64>;
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
/// use wkt::{TryFromWkt, Wkt, geo_types_from_wkt} ;
/// use aoer_plotty_rs::geo_types::hatch::{LineHatch, Hatch};
/// use std::str::FromStr;
/// let geoms: Polygon::<f64> = Polygon::try_from(
///     Wkt::<f64>::from_str("POLYGON ((350 100, 450 450, 150 400, 100 200, 350 100), (200 300, 350 350, 300 200, 200 300))")
///         .expect("Failed to load WKT"))
///     .expect("Failed to load box");
/// let hatch = geoms.hatch(&LineHatch{}, 45.0, 5.0, 2.5).expect("Got some hatches in here failin'");
/// ```
/// ![hatch-example-1][hatch-example-1]
#[embed_doc_image("hatch-example-1", "images/hatch-demo-1.png")]
pub trait Hatch
{
    fn hatch(&self, pattern: &dyn HatchPattern, angle: f64, scale: f64, inset: f64)
             -> Result<MultiLineString<f64>, InvalidHatchGeometry>;
}

/// The no-hatch option
#[derive(Debug, Clone)]
pub struct NoHatch {}
impl NoHatch {
    pub fn gen() -> Rc<NoHatch> {
        Rc::new(Self {})
    }
}

impl HatchPattern for NoHatch {
    fn generate(&self, _bbox: &Rect<f64>, _scale: f64) -> MultiLineString<f64> {
        MultiLineString::new(vec![])
    }
}

/// The basic built in parallel LineHatch.
#[derive(Debug, Clone)]
pub struct LineHatch {}
impl LineHatch {
    pub fn gen() -> Rc<LineHatch> {
        Rc::new(Self {})
    }
}

impl HatchPattern for LineHatch {
    fn generate(&self, bbox: &Rect<f64>, scale: f64) -> MultiLineString<f64> {
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
        };
        MultiLineString::<f64>::new(lines)
    }
}


#[derive(Debug, Clone)]
pub struct CrossHatch {}
impl CrossHatch {
    pub fn gen() -> Rc<CrossHatch> {
        Rc::new(Self {})
    }
}



impl HatchPattern for CrossHatch {
    fn generate(&self, bbox: &Rect<f64>, scale: f64) -> MultiLineString<f64> {
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
        };
        let mut x = min.x;
        let mut count = 0u32;
        while x < max.x {
            if count % 2 == 0 {
                lines.push(geo_types::LineString::<f64>::new(vec![
                    coord! {x: x, y: min.y},
                    coord! {x: x, y: max.y},
                ]));
            } else {
                lines.push(geo_types::LineString::<f64>::new(vec![
                    coord! {x: x, y: max.y},
                    coord! {x: x, y: max.y},
                ]));
            }
            x += scale;
            count += 1;
        };
        MultiLineString::<f64>::new(lines)
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

impl Hatch for MultiPolygon<f64> {
    fn hatch(&self, pattern: &dyn HatchPattern, angle: f64, scale: f64, inset: f64)
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

impl Hatch for Polygon<f64> {
    fn hatch(&self, pattern: &dyn HatchPattern, angle: f64, scale: f64, inset: f64)
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
    use super::Hatch;


    #[test]
    fn test_box_hatch() {
        let rect = Rect::<f64>::new(
            coord! {x: 0.0, y: 0.0},
            coord! {x: 100.0, y: 100.0});
        //let hatch_lines =
        LineHatch {}.generate(&rect, 10.0);
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
        let hatches = poly.hatch(&LineHatch {}, 0.0, 5.0, 0.0).expect("Failed to Ok the hatches.");
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
        let hatches = poly.hatch(&LineHatch {}, PI / 4.0, 5.0, 0.0).expect("Failed to Ok the hatches.");
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
        let _hatches = mpoly.hatch(&LineHatch {}, 0.0, 5.0, 0.0)
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
        let hatches = poly.hatch(&LineHatch {},
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
        let hatch = LineHatch {};
        let hatches = poly.hatch(&hatch,
                                 PI / 4.0, 5.0, 2.0)
            .expect("Disjoint hatch failed");
        // println!("Got these {} angled then disjointed after inset lines: {:?}", &hatches.0.len(), &hatches);
        assert!((&hatches).0.len() == 12);
    }
}