use std::fmt::Debug;
use geo_types::{CoordNum, LineString, MultiLineString, Polygon};
use geos::{Geom, Geometry as GeosGeometry};
use num_traits::real::Real;

pub trait Clip{
    fn clip(&self, other: &Self) -> MultiLineString<f64>;
}

impl Clip for LineString<f64> {
    fn clip(&self, other: &Self) -> MultiLineString<f64> {
        let geo_perim_front: geos::Geometry = Polygon::new(self.clone(), vec![])
            .try_into()
            .expect("Invalid geometry");
        let geo_clipped_lines: Vec<geos::Geometry> = vec![other.clone().try_into().expect("Couldn't convert back line")];
        let geo_clipped_lines_collection = geos::Geometry::create_geometry_collection(geo_clipped_lines).expect("Got this far?");
        // let _clipped_object = geo_clipped_lines_collection.difference(&geo_perim_front).expect("Got this far?");
        let _clipped_object = geo_perim_front.difference(&geo_clipped_lines_collection).expect("Got this far?");
        println!("CLipped object is: {}", _clipped_object.to_wkt().expect("As a string!"));
        let geo_out: geo_types::Geometry<f64> = _clipped_object.try_into().expect("Could not convert back to lines");
        let out = match geo_out{
            geo_types::Geometry::MultiLineString(mls) => mls,
            geo_types::Geometry::GeometryCollection(gc) =>{
                let foo = gc.iter().map(|g| match g {
                    geo_types::Geometry::MultiLineString(mls) => mls.clone(),
                    geo_types::Geometry::LineString(ls) => MultiLineString(vec![ls.clone()]),
                    _ => MultiLineString(vec![])
                }).collect::<Vec<MultiLineString<f64>>>();
                println!("Foo is: {:?}", foo);
                MultiLineString::new(foo.iter().map(|mls| mls.0.clone()).flatten().collect())
            },
            _ => MultiLineString::new(vec![])
        };
        out
    }
}


#[cfg(test)]
mod test{
    use std::f64::consts::PI;
    use geo_types::{coord, Rect, Polygon, LineString};
    use geos::{Geom, Geometry};
    use super::*;

    #[test]
    fn test_clip_simple() {
        let joydivfront = LineString::<f64>::new(vec![
            coord! {x: 0.0, y: 0.0},
            coord! {x: 10.0, y: 0.0},
            coord! {x: 20.0, y: 10.0},
            coord! {x: 30.0, y: 100.0},
            coord! {x: 40.0, y: 10.0},
            coord! {x: 50.0, y: 0.0},
            coord! {x: 60.0, y: 0.0},
        ]);
        let joydivback = LineString::<f64>::new(vec![
            coord! {x: 0.0, y: 10.0},
            coord! {x: 10.0, y: 10.0},
            coord! {x: 20.0, y: 10.0},
            coord! {x: 30.0, y: 40.0},
            coord! {x: 40.0, y: 10.0},
            coord! {x: 50.0, y: 10.0},
            coord! {x: 60.0, y: 10.0},
        ]);
        let clipped = joydivback.clip(&joydivfront);
        println!("Clipped: {:?}", clipped);
    }

    #[test]
    fn test_experiment1_geos_clip_line() {
        let joydivfront = LineString::<f64>::new(vec![
            coord! {x: 0.0, y: 0.0},
            coord! {x: 10.0, y: 0.0},
            coord! {x: 20.0, y: 10.0},
            coord! {x: 30.0, y: 100.0},
            coord! {x: 40.0, y: 10.0},
            coord! {x: 50.0, y: 0.0},
            coord! {x: 60.0, y: 0.0},
        ]);
        let joydivback = LineString::<f64>::new(vec![
            coord! {x: 0.0, y: 10.0},
            coord! {x: 10.0, y: 10.0},
            coord! {x: 20.0, y: 10.0},
            coord! {x: 30.0, y: 40.0},
            coord! {x: 40.0, y: 10.0},
            coord! {x: 50.0, y: 10.0},
            coord! {x: 60.0, y: 10.0},
        ]);



        let geo_perim_front: geos::Geometry = Polygon::new(joydivfront.clone(), vec![])
            .try_into()
            .expect("Invalid geometry");
        // let hatch_lines: Vec<geo_types::LineString<f64>> = hatch_lines.iter().map(|x| x.clone()).collect();
        let geo_clipped_lines: Vec<Geometry> = vec![joydivback.clone().try_into().expect("Couldn't convert back line")];
        println!("geo_clipped_lines is {:?}", &joydivback);
            // (&hatch_lines).iter()
            // .map(|hatch_line|
            //     (hatch_line).try_into().expect("Invalid hatch lines")).collect();
        let geo_clipped_lines_collection = Geometry::create_geometry_collection(geo_clipped_lines).expect("Got this far?");
        // let _clipped_object = geo_perim_front.difference(&geo_clipped_lines_collection).expect("Got this far?");
        let _clipped_object = geo_clipped_lines_collection.difference(&geo_perim_front).expect("Got this far?");
        println!("CLipped object is: {}", _clipped_object.to_wkt().expect("As a string!"))
    }

}
