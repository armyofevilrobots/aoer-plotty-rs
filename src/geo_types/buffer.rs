use std::error::Error;
use geo_types::{Geometry, MultiPolygon, Polygon};
use geos::Geom;

pub trait Buffer{
   fn buffer(&self, distance: f64) -> Result<MultiPolygon<f64>, Box<dyn Error>>;
}

impl Buffer for Geometry<f64>{
    fn buffer(&self, distance: f64) -> Result<MultiPolygon<f64>, Box<dyn Error>>{
        let geo_self: geos::Geometry = match self {
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
        let buffered_self = geo_self.buffer(distance, 8)?;

        let gt_out: geo_types::Geometry<f64> = geo_types::Geometry::try_from(buffered_self)?;

        match gt_out {
            geo_types::Geometry::Polygon(_poly) => Ok(MultiPolygon::<f64>::new(vec![_poly.clone()])),
            geo_types::Geometry::MultiPolygon(_polys) => Ok(MultiPolygon::<f64>::new(_polys.0.clone())),
            geo_types::Geometry::GeometryCollection(gc) => {
                let foo: Vec<Vec<Polygon<f64>>> = gc.iter().map(|g| match g {
                    geo_types::Geometry::Polygon(poly) => vec![poly.clone()],
                    geo_types::Geometry::MultiPolygon(polys) => polys.0.clone(),
                    _ => vec![] // Some part didn't buffer <shrug/>
                }).collect();

                // let tmp = foo.iter().map(|poly|poly).flatten().collect();
                Ok(MultiPolygon::new(foo.iter().map(|polys| polys.clone()).flatten().collect())) // .flatten().collect()))

            },
            _ => Ok(MultiPolygon::new(vec![])),
        }
    }

}