use std::collections::HashMap;
use std::error::Error;
use std::rc::Rc;
use geo::rotate::RotatePoint;
use geo_types::{coord, Geometry, GeometryCollection, LineString, Point, point, Rect};
use crate::geo_types::shapes::arc_center;
use crate::geo_types::buffer::Buffer;
use crate::geo_types::boolean::BooleanOp;

pub enum CarlsonSmithTruchet {
    TLBR(bool),
    DIV(bool),
    DOTS(bool),
    PINWHEEL(bool),
    PLUS(bool),
    UNHAPPY(bool),
    HUGS(bool),
}

impl CarlsonSmithTruchet {

    pub fn full_set(invert: bool) -> HashMap<String, Rc<Geometry<f64>>>{
        let mut truchets = HashMap::new();
        for i in (0..360).step_by(90){
            truchets.insert(format!("TLBR{}", i),
                            Rc::new(
                                CarlsonSmithTruchet::TLBR(invert)
                                    .draw()
                                    .unwrap()
                                    .rotate_around_point(
                                        f64::from(i),
                                        point!{x: 0.0, y: 0.0})));
            truchets.insert(format!("DIV{}", i),
                            Rc::new(
                                CarlsonSmithTruchet::DIV(invert)
                                    .draw()
                                    .unwrap()
                                    .rotate_around_point(
                                        f64::from(i),
                                        point!{x: 0.0, y: 0.0})));
            truchets.insert(format!("UNHAPPY{}", i),
                            Rc::new(
                                CarlsonSmithTruchet::UNHAPPY(invert)
                                    .draw()
                                    .unwrap()
                                    .rotate_around_point(
                                        f64::from(i),
                                        point!{x: 0.0, y: 0.0})));
            truchets.insert(format!("HUGS{}", i),
                            Rc::new(
                                CarlsonSmithTruchet::HUGS(invert)
                                    .draw()
                                    .unwrap()
                                    .rotate_around_point(
                                        f64::from(i),
                                        point!{x: 0.0, y: 0.0})));
            // All teh dots are identical
            truchets.insert(format!("DOTS{}", i),
                            Rc::new(
                                CarlsonSmithTruchet::DOTS(invert)
                                    .draw()
                                    .unwrap()));
            truchets.insert(format!("PINWHEEL{}", i),
                            Rc::new(
                                CarlsonSmithTruchet::PINWHEEL(invert)
                                    .draw()
                                    .unwrap()));
            truchets.insert(format!("PLUS{}", i),
                            Rc::new(
                                CarlsonSmithTruchet::PLUS(invert)
                                    .draw()
                                    .unwrap()));



        }
        println!("Len of truchet set {}", truchets.len());
        truchets
    }

    /// Returns the Geometry for the given truchet, centered on 0.0, with a scale of 1.0
    pub fn draw(&self) -> Result<Geometry<f64>, Box<dyn Error>> {
        match self {
            CarlsonSmithTruchet::TLBR(false) => {
                let mut ac1 = Geometry::LineString(
                    arc_center(0.5, 0.5, 0.5, 180.0, 270.0))
                    .buffer(1.0f64 / 6.0f64)?; // Buffer is 1/2 of 1/3
                let ac2 = Geometry::LineString(
                    arc_center(-0.5, -0.5, 0.5, 90.0, 0.0))
                    .buffer(1.0f64 / 6.0f64)?; // Buffer is 1/2 of 1/3
                ac1.0.extend(ac2.0);
                Ok(Geometry::MultiPolygon(ac1))
            },
            CarlsonSmithTruchet::DIV(false) => {
                Ok(Geometry::GeometryCollection(
                    GeometryCollection::new_from(vec![
                        Geometry::MultiPolygon(Geometry::Point(Point::new(0.0, 0.5))
                            .buffer(1.0f64 / 6.0f64)
                            .unwrap()),
                        Geometry::MultiPolygon(Geometry::Point(Point::new(0.0, -0.5))
                            .buffer(1.0f64 / 6.0f64)
                            .unwrap()),
                        Geometry::MultiPolygon(Geometry::LineString(LineString::new(vec![
                            coord! {x: -0.5, y: 0.0},
                            coord! {x: 0.5, y: 0.0}]))
                            .buffer(1.0f64 / 6.0f64)
                            .unwrap()),
                    ]
                    )))
            },
            CarlsonSmithTruchet::DOTS(false) => {
                Ok(Geometry::GeometryCollection(
                    GeometryCollection::new_from(vec![
                        Geometry::MultiPolygon(Geometry::Point(Point::new(0.0, 0.5))
                            .buffer(1.0f64 / 6.0f64)
                            .unwrap()),
                        Geometry::MultiPolygon(Geometry::Point(Point::new(0.0, -0.5))
                            .buffer(1.0f64 / 6.0f64)
                            .unwrap()),
                        Geometry::MultiPolygon(Geometry::Point(Point::new(0.5, 0.0))
                            .buffer(1.0f64 / 6.0f64)
                            .unwrap()),
                        Geometry::MultiPolygon(Geometry::Point(Point::new(-0.5, -0.0))
                            .buffer(1.0f64 / 6.0f64)
                            .unwrap()),
                    ])))
            },
            CarlsonSmithTruchet::PINWHEEL(false) => {
                let center = Geometry::Rect(Rect::<f64>::new(
                    coord! {x:-0.5, y:-0.5},
                    coord! {x: 0.5, y: 0.5},
                ));
                let corners = CarlsonSmithTruchet::PINWHEEL(true).draw()?;
                let points = CarlsonSmithTruchet::DOTS(false).draw()?;
                let out = center
                    .union(&points)?
                    .difference(&corners)?;
                Ok(out)
            },
            CarlsonSmithTruchet::PINWHEEL(true) => {
                Ok(Geometry::GeometryCollection(
                    GeometryCollection::new_from(vec![
                        Geometry::MultiPolygon(Geometry::Point(Point::new(0.5, 0.5))
                            .buffer(1.0f64 / 3.0f64)
                            .unwrap()),
                        Geometry::MultiPolygon(Geometry::Point(Point::new(0.5, -0.5))
                            .buffer(1.0f64 / 3.0f64)
                            .unwrap()),
                        Geometry::MultiPolygon(Geometry::Point(Point::new(-0.5, 0.5))
                            .buffer(1.0f64 / 3.0f64)
                            .unwrap()),
                        Geometry::MultiPolygon(Geometry::Point(Point::new(-0.5, -0.5))
                            .buffer(1.0f64 / 3.0f64)
                            .unwrap()),
                    ])))
            },
            CarlsonSmithTruchet::PLUS(false) => {
                Ok(Geometry::MultiPolygon(Geometry::LineString(LineString::new(vec![
                    coord! {x: -0.5, y: 0.0},
                    coord! {x: 0.5, y: 0.0}]))
                    .buffer(1.0f64 / 6.0f64)?)
                    .union(&Geometry::MultiPolygon(Geometry::LineString(
                        LineString::new(vec![
                            coord!{x:0.0, y:-0.5},
                            coord!{x:0.0, y: 0.5}]))
                        .buffer(1.0f64 / 6.0f64)?))?)
            },
            CarlsonSmithTruchet::UNHAPPY(false) => {
                Ok(Geometry::GeometryCollection(
                    GeometryCollection::new_from(vec![
                        Geometry::MultiPolygon(Geometry::Point(Point::new(-0.5, 0.0))
                            .buffer(1.0f64 / 6.0f64)?),
                        Geometry::MultiPolygon(Geometry::Point(Point::new(0.0, -0.5))
                            .buffer(1.0f64 / 6.0f64)?),
                        Geometry::MultiPolygon(Geometry::LineString(
                            arc_center(0.5, 0.5, 0.5, 180.0, 270.0))
                            .buffer(1.0f64 / 6.0f64)?)
                    ])))

            },
            CarlsonSmithTruchet::HUGS(false) => {
                let center = Geometry::Rect(Rect::<f64>::new(
                    coord! {x:-0.5, y: 0.5},
                    coord! {x: 0.5, y: -1.0/6.0},
                ));
                let corners = CarlsonSmithTruchet::PINWHEEL(true).draw()?;
                let points = CarlsonSmithTruchet::DOTS(false).draw()?;
                let out = center
                    .union(&points)?
                    .difference(&corners)?;
                Ok(out)
            },
            CarlsonSmithTruchet::HUGS(true) => {
                Ok(Geometry::GeometryCollection(
                    GeometryCollection::new_from(vec![
                        Geometry::MultiPolygon(Geometry::Point(Point::new(0.5, 0.5))
                            .buffer(1.0f64 / 3.0f64)
                            .unwrap()),
                        Geometry::MultiPolygon(Geometry::Point(Point::new(0.5, -0.5))
                            .buffer(1.0f64 / 3.0f64)
                            .unwrap()),
                        Geometry::MultiPolygon(Geometry::Point(Point::new(-0.5, 0.5))
                            .buffer(1.0f64 / 3.0f64)
                            .unwrap()),
                        Geometry::MultiPolygon(Geometry::Point(Point::new(-0.5, -0.5))
                            .buffer(1.0f64 / 3.0f64)
                            .unwrap()),
                    ])))
            },
            CarlsonSmithTruchet::TLBR(true) => {
                // let dots = CarlsonSmithTruchet::DOTS(false).draw()?;
                let dots = CarlsonSmithTruchet::PINWHEEL(true).draw()?;
                let line = Geometry::MultiPolygon(
                    Geometry::LineString(
                        LineString(vec![
                            coord!{x: -0.5, y: 0.5},
                            coord!{x:0.5, y:-0.5}
                        ]))
                        .buffer(1.0f64 / 5.0f64)?
                );
                Ok(
                    dots.union(&line)?
                    .difference(&CarlsonSmithTruchet::TLBR(false)
                        .draw()?)?
                   )
            },
            CarlsonSmithTruchet::DIV(true) => {
                let center = Geometry::Rect(Rect::<f64>::new(
                    coord! {x:-0.5, y:-0.5},
                    coord! {x: 0.5, y: 0.5},
                ));
                let corners = CarlsonSmithTruchet::PINWHEEL(true).draw()?;
                Ok(
                    center.union(&corners)?
                        .difference(&CarlsonSmithTruchet::DIV(false).draw()?)?
                )
            },
            CarlsonSmithTruchet::DOTS(true) => {
                let center = Geometry::Rect(Rect::<f64>::new(
                    coord! {x:-0.5, y:-0.5},
                    coord! {x: 0.5, y: 0.5},
                ));
                let corners = CarlsonSmithTruchet::PINWHEEL(true).draw()?;
                Ok(
                    center.union(&corners)?
                        .difference(&CarlsonSmithTruchet::DOTS(false).draw()?)?
                )
            },
            CarlsonSmithTruchet::PLUS(true) => {
                let center = Geometry::Rect(Rect::<f64>::new(
                    coord! {x:-0.5, y:-0.5},
                    coord! {x: 0.5, y: 0.5},
                ));
                let corners = CarlsonSmithTruchet::PINWHEEL(true).draw()?;
                Ok(
                    center.union(&corners)?
                        .difference(&CarlsonSmithTruchet::PLUS(false).draw()?)?
                )

            },
            CarlsonSmithTruchet::UNHAPPY(true) => {
                let center = Geometry::Rect(Rect::<f64>::new(
                    coord! {x:-0.5, y:-0.5},
                    coord! {x: 0.5, y: 0.5},
                ));
                let corners = CarlsonSmithTruchet::PINWHEEL(true).draw()?;
                let ac1 = Geometry::LineString(
                    arc_center(0.5, 0.5, 0.5, 180.0, 270.0))
                    .buffer(1.0f64 / 6.0f64)?; // Buffer is 1/2 of 1/3
                let p1 = Geometry::MultiPolygon(Geometry::Point(Point::new(-0.5, 0.0))
                    .buffer(1.0f64 / 6.0f64)?);
                let p2 = Geometry::MultiPolygon(Geometry::Point(Point::new(0.0, -0.5))
                    .buffer(1.0f64 / 6.0f64)?);

                Ok(
                    center.union(&corners)?
                        .difference(&Geometry::MultiPolygon(ac1))?
                        .difference(&p1)?
                        .difference(&p2)?
                )
            }
        }
    }
}

#[cfg(test)]
pub mod tests {
    use crate::elements::CarlsonSmithTruchet;

    #[test]
    fn test_cst_all() {
        // Basically just testing that it runs.
        let _full_set = CarlsonSmithTruchet::full_set(false);
        let _full_set = CarlsonSmithTruchet::full_set(true);

    }
}


