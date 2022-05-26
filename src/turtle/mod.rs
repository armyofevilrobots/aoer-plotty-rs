use geo_types::{LineString, MultiLineString, Point};

pub struct Turtle {
    lines: Vec<Vec<Point<f64>>>,
    position: Point<f64>,
    start: Option<Point<f64>>,
    heading: f64,
    pen: bool,
}

pub fn degrees(deg: f64) -> f64 {
    std::f64::consts::PI * (deg / 180.0)
}


pub trait TurtleTrait {
    fn new() -> Turtle;
    fn fwd(self, distance: f64) -> Self;
    fn left(self, angle: f64) -> Self;
    fn right(self, angle: f64) -> Self;
    fn pen_up(self) -> Self;
    fn pen_down(self) -> Self;
    fn close(self) -> Self;
    fn to_multiline(self) -> MultiLineString<f64>;
}


impl TurtleTrait for Turtle {
    fn new() -> Self {
        Turtle {
            lines: vec![],
            position: Point::new(0.0f64, 0.0f64),
            start: None,
            heading: 0.0,
            pen: false,
        }
    }

    fn fwd(mut self, distance: f64) -> Self {
        let pos = self.position + Point::new(distance * self.heading.cos(),
                                             distance * self.heading.sin());
        if self.pen {
            self.lines.last_mut()
                .expect("Turtle closing without an active line!")
                .push(pos)
        }

        self.position = pos;
        self
    }

    fn left(mut self, angle: f64) -> Self {
        self.heading = (self.heading + angle) % 360.0f64;
        self
    }

    fn right(mut self, angle: f64) -> Self {
        self.heading = (self.heading - angle) % 360.0f64;
        self
    }

    fn pen_up(mut self) -> Self {
        self.pen = false;
        self.start = None;
        self
    }

    fn pen_down(mut self) -> Self {
        self.pen = true;
        self.start = Some(self.position.clone());
        self.lines.push(vec![self.position.clone()]);
        self
    }

    fn close(mut self) -> Self {
        match self.start {
            Some(start) => {
                if self.pen {
                    self.lines.last_mut()
                        .expect("Turtle closing without an active line!")
                        .push(self.start.expect("Turtle closing without a start point!").clone())
                }
                self.position = start.clone();
                self
            }
            None => self
        }
    }

    fn to_multiline(self) -> MultiLineString<f64> {
        // MultiLineString::new(vec![])
        self.lines.iter().map(|line| {
            LineString::from(line.clone())
        }).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geo_types::PointDistance;

    #[test]
    fn test_pendown() {
        let t = Turtle::new()
            .pen_down();
        assert_eq!(t.pen, true);
        let t = Turtle::new();
        assert_eq!(t.pen, false);
    }

    #[test]
    fn test_simple_box() {
        let t = Turtle::new()
            .pen_down()
            .fwd(100.0)
            .right(degrees(90.0))
            .fwd(100.0)
            .right(degrees(90.0))
            .fwd(100.0)
            .right(degrees(90.0))
            .close();
        assert!(t.lines[0][0]
            .distance(&Point::new(0.0f64, 0.0f64)) < 0.0001f64);
        assert!(t.lines[0][1]
            .distance(&Point::new(100.0f64, 0.0f64)) < 0.0001f64);
        assert!(t.lines[0][2]
            .distance(&Point::new(100.0f64, -100.0f64)) < 0.0001f64);
        assert!(t.lines[0][3]
            .distance(&Point::new(0.0f64, -100.0f64)) < 0.0001f64);
        assert!(t.lines[0][4]
            .distance(&Point::new(0.0f64, 0.0f64)) < 0.0001f64);
    }
}
