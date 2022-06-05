use geo_types::{Point, CoordNum, Polygon, MultiPolygon, MultiLineString, Rect, coord, GeometryCollection};
use geo::bounding_rect::BoundingRect;
use geo::GeoFloat;
use geo::rotate::Rotate;
use geos::from_geo;
use geos::to_geo;
use geos::{Geom, Geometry};
use geos::GeometryTypes::{GeometryCollection as GGeometryCollection, LineString};
use num_traits::real::Real;
use std::error::Error;
use std::fmt::{Display, Formatter};
use num_traits::FloatErrorKind::Invalid;


#[derive(Debug)]
pub enum InvalidHatchGeometry {
    CouldNotGenerateHatch,
    InvalidBoundary,
    InvalidResultGeometry
}


impl Display for InvalidHatchGeometry {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match *self{
            InvalidHatchGeometry::CouldNotGenerateHatch=>f.write_str("Could not generate hatch"),
            InvalidHatchGeometry::InvalidBoundary=>f.write_str("Could not process boundary geometry"),
            InvalidHatchGeometry::InvalidResultGeometry=>f.write_str("Processed hatch, but result geometry was invalid"),
        }
    }
}

impl Error for InvalidHatchGeometry{
}

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
/// on their interiors.
pub trait Hatch<P>
    where P: HatchPattern<f64>,
          {
    fn hatch(&self, pattern: P, angle: f64, scale: f64, inset: f64) -> Result<MultiLineString<f64>, InvalidHatchGeometry>;
}

struct LineHatch {}

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

fn gt_flatten_mlines(geo: geo_types::Geometry<f64>, mut existing: MultiLineString<f64>) -> MultiLineString<f64>{
    match geo{
        geo_types::Geometry::GeometryCollection::<f64>(new_geo) => {
            for geo in new_geo{
                existing = gt_flatten_mlines(geo, existing)
            }
            existing
        },
        geo_types::Geometry::MultiLineString::<f64>(ms)=> {
            for line in ms{
                existing.0.push(line);
            }
            existing
        },
        geo_types::Geometry::LineString::<f64>(ls)=>{
            existing.0.push(ls);
            existing
        },
        _ => {
            existing
        }
    }
}

impl<P> Hatch<P> for Polygon<f64>
    where P: HatchPattern<f64>
{
    fn hatch(&self, pattern: P, angle: f64, scale: f64, inset: f64) -> Result<MultiLineString<f64>, InvalidHatchGeometry> {
        let bbox = self.bounding_rect().ok_or(InvalidHatchGeometry::CouldNotGenerateHatch)?
            .to_polygon().rotate_around_centroid(angle).bounding_rect()
            .ok_or(InvalidHatchGeometry::CouldNotGenerateHatch)?;
        let hatches = pattern.generate(&bbox, scale);
        let geo_perimeter: geos::Geometry = self.try_into().or(Err(InvalidHatchGeometry::InvalidBoundary))?;
        let hatch_lines: Vec<geo_types::LineString<f64>> = hatches.iter().map(|x| x.to_owned()).collect();
        let geo_hatchlines: Vec<Geometry> = hatch_lines.iter()
            .map(|hatch_line|{
                hatch_line.clone().try_into().expect("Invalid hatch lines")}).collect();
        let geo_hatchlines = Geometry::create_geometry_collection(geo_hatchlines).or(Err(InvalidHatchGeometry::CouldNotGenerateHatch))?;
        let hatched_object = geo_perimeter.intersection(&geo_hatchlines).or(Err(InvalidHatchGeometry::CouldNotGenerateHatch))?;
        let out: geo_types::Geometry<f64> = hatched_object.try_into().or(Err(InvalidHatchGeometry::InvalidResultGeometry))?;
        let out = gt_flatten_mlines(out, MultiLineString::new(vec![]));
        Ok(out)
    }
}


#[cfg(test)]
mod test {
    use geos::Geometry;
    use geos::GeometryTypes::{GeometryCollection, LineString};
    use super::*;

    #[test]
    fn test_box_hatch() {
        let rect = Rect::<f64>::new(coord! {x: 0.0, y: 0.0}, coord! {x: 100.0, y: 100.0});
        let hatch_lines = LineHatch {}.generate(&rect, 10.0);
        println!("LINES HATCHED: {:?}", hatch_lines);
    }

    #[test]
    fn test_experiment1_geos_clip_hatch() {
        let rect = Rect::<f64>::new(coord! {x: -100.0, y: -100.0}, coord! {x: 100.0, y: 100.0});
        let hatch_lines = LineHatch {}.generate(&rect, 5.0);
        let poly = Polygon::<f64>::new(
            geo_types::LineString::<f64>::new(vec![
                coord!{x: 0.0, y: 20.0},
                coord!{x: 20.0, y: 0.0},
                coord!{x: 0.0, y: -20.0},
                coord!{x: -20.0, y: 0.0},
                coord!{x: 0.0, y: 20.0},
            ]),
            vec![]);
        let geo_perimeter: geos::Geometry = (&poly).try_into().expect("Invalid geometry");
        let hatch_lines: Vec<geo_types::LineString<f64>> = hatch_lines.iter().map(|x| x.clone()).collect();
        let geo_hatchlines: Vec<Geometry> = (&hatch_lines).iter()
            .map(|hatch_line|
                (hatch_line).try_into().expect("Invalid hatch lines")).collect();
        let geo_hatchlines = Geometry::create_geometry_collection(geo_hatchlines).expect("Got this far?");
        let hatched_object = geo_perimeter.intersection(&geo_hatchlines).expect("Got this far?");
        // println!("Hatched object is: {}", hatched_object.to_wkt().expect("As a string!"))
    }

    #[test]
    fn test_trait_hatch_poly() {
        let rect = Rect::<f64>::new(coord! {x: -100.0, y: -100.0}, coord! {x: 100.0, y: 100.0});
        let hatch_lines = LineHatch {}.generate(&rect, 5.0);
        let poly = Polygon::<f64>::new(
            geo_types::LineString::<f64>::new(vec![
                coord!{x: 0.0, y: 20.0},
                coord!{x: 20.0, y: 0.0},
                coord!{x: 0.0, y: -20.0},
                coord!{x: -20.0, y: 0.0},
                coord!{x: 0.0, y: 20.0},
            ]),
            vec![]);
        let hatches = poly.hatch(LineHatch{}, 0.0, 5.0, 0.0 );
        println!("Hatched object is: {:?}", hatches)
    }

}