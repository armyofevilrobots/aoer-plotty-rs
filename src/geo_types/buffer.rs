use std::error::Error;
use geo_types::{Geometry, LineString, MultiLineString, MultiPolygon, Polygon};
use geos::Geom;
use crate::geo_types::flatten::FlattenPolygons;

// TODO: Take this out :\
fn flatten_gt_geom_to_multipolygon(source_gt_geometry: &Geometry<f64>) -> Result<MultiPolygon<f64>, Box<dyn Error>> {
    match source_gt_geometry {
        geo_types::Geometry::Polygon(_poly) => Ok(MultiPolygon::<f64>::new(vec![_poly.clone()])),
        geo_types::Geometry::MultiPolygon(_polys) => Ok(MultiPolygon::<f64>::new(_polys.0.clone())),
        geo_types::Geometry::GeometryCollection(gc) => {
            let foo: Vec<Vec<Polygon<f64>>> = gc.iter().map(|g| match g {
                geo_types::Geometry::Polygon(poly) => vec![poly.clone()],
                geo_types::Geometry::MultiPolygon(polys) => polys.0.clone(),
                geo_types::Geometry::GeometryCollection(gc) => {
                    flatten_gt_geom_to_multipolygon(
                        &geo_types::Geometry::GeometryCollection(gc.clone()))
                        .unwrap_or(MultiPolygon::new(vec![]))
                        .0
                },
                _ => vec![] // Some part didn't buffer <shrug/>
            }).collect();

            Ok(MultiPolygon::new(foo.iter().map(|polys| polys.clone()).flatten().collect())) // .flatten().collect()))

        },
        _ => Ok(MultiPolygon::new(vec![])),
    }
}



/// #Buffer
///
/// Given a geo_types geometry enum type, either inset (negative distance) or
/// outset (positive distance) it by the distance amount, and return a new MultiPolygon
/// which contains the offset version of the geometry. Supports a variety of input types
/// including [`geo_types::Geometry`]::Point in case you want to create circles ;)
pub trait Buffer{
   fn buffer(&self, distance: f64) -> Result<MultiPolygon<f64>, Box<dyn Error>>;
}

impl Buffer for Geometry<f64>{
    fn buffer(&self, distance: f64) -> Result<MultiPolygon<f64>, Box<dyn Error>>{
        let geo_self: geos::Geometry = match self {
            Geometry::Point(p) => geos::Geometry::try_from(p),
            Geometry::LineString(line) => geos::Geometry::try_from(line),
            Geometry::Polygon(poly) => geos::Geometry::try_from(poly),
            Geometry::MultiPolygon(polys) => geos::Geometry::try_from(polys),
            Geometry::MultiLineString(mls) => {
                geos::Geometry::create_multiline_string(mls.0
                    .clone()
                    .iter()
                    .map(|line| {
                        geos::Geometry::try_from(line)
                            .unwrap_or(geos::Geometry::create_empty_line_string().unwrap())
                    })
                    .collect())
            },
            _ => Err(geos::Error::InvalidGeometry("Wrong type of geometry".into()))
        }?;
        let buffered_self = geo_self.buffer(distance, 4)?;

        let gt_out: geo_types::Geometry<f64> = geo_types::Geometry::try_from(buffered_self)?;
        // flatten_gt_geom_to_multipolygon(&gt_out)
        gt_out.flatten_polys()
    }

}