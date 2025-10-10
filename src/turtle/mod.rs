use geo_types::{LineString, MultiLineString, Point, Polygon};

/// # Turtle Module
///
/// This provides logo-style turtle features, which are useful for things like
/// lindemeyer systems, etc.
#[derive(Clone)]
pub struct Turtle {
    stack: Vec<Turtle>,
    lines: Vec<Vec<Point<f64>>>,
    position: Point<f64>,
    start: Option<Point<f64>>,
    heading: f64,
    pen: bool,
}

/// Helper function to convert degrees to radians
pub fn degrees(deg: f64) -> f64 {
    std::f64::consts::PI * (deg / 180.0)
}

/// TurtleTrait provides turtle related functions for the Turtle struct.
///
/// Provides 2D turtle actions, and a stack-based history for drawing
/// 2D graphics.
///
/// # Example
///
/// ```
/// use geo_types::MultiLineString;
/// use aoer_plotty_rs::turtle::{Turtle, TurtleTrait, degrees};
/// let mline_string: MultiLineString<f64> = Turtle::new()
///     .pen_down()
///     .fwd(100.0)
///     .right(degrees(90.0))
///     .fwd(100.0)
///     .right(degrees(90.0))
///     .fwd(100.0)
///     .right(degrees(90.0))
///     .fwd(100.0)
///     .right(degrees(90.0))
///     .to_multiline();
/// ```

pub trait TurtleTrait {
    fn new() -> Turtle;

    /// #fwd
    ///
    /// Move forward @distance units (negative values are allowed)
    fn fwd(self, distance: f64) -> Self;

    /// #left
    ///
    /// Turn left @angle radians
    fn left(self, angle: f64) -> Self;

    /// #right
    ///
    /// Turn right @angle radians
    fn right(self, angle: f64) -> Self;

    /// #pen_up
    ///
    /// Lift the pen and discard the closing state
    fn pen_up(self) -> Self;

    /// #pen_down
    ///
    /// Put the pen down so that we start drawing, and store the "start position" state
    /// for later closing the drawing.
    fn pen_down(self) -> Self;

    /// #close
    ///
    /// Automatically close the current line. Great for automagically closing Polygons.
    fn close(self) -> Self;

    /// #push
    ///
    /// Pushes the current state onto the stack, including heading, position.
    fn push(self) -> Self;

    /// #pop
    ///
    /// Pops the state back to the previously [`crate::turtle::TurtleTrait::push`]'d state, but retains the line state
    /// so that we can backtrack and continue drawing.
    fn pop(self) -> Self;

    /// # walk_lpath (Walk L-system Path)
    ///
    /// Used to take an existing expanded l-system path (see [`crate::l_system::LSystem`]) for more
    /// information on the expansion syntax. Walks the L-system, performing movements, stack
    /// push/pop, and turns.
    fn walk_lpath(self, lpath: &String, angle: f64, distance: f64) -> Self;

    /// # to_multiline
    ///
    /// Takes the lines recorded in the Turtle state and returns a [`geo_types::MultiLineString`]
    fn to_multiline(&mut self) -> MultiLineString<f64>;

    /// # to_polygon
    ///
    /// Returns a [`geo_types::Polygon`] from the Turtle state. May return an error for turtles
    /// which have self-intersecting lines, or zero-volume polygons.
    fn to_polygon(&mut self) -> Result<Polygon<f64>, geo_types::Error>;
    // fn to_multipolygon(self) -> Result<MultiPolygon<f64>, geo_types::Error>;
}

impl TurtleTrait for Turtle {
    fn new() -> Self {
        Turtle {
            stack: vec![],
            lines: vec![],
            position: Point::new(0.0f64, 0.0f64),
            start: None,
            heading: 0.0,
            pen: false,
        }
    }

    fn fwd(mut self, distance: f64) -> Self {
        let pos = self.position
            + Point::new(distance * self.heading.cos(), distance * self.heading.sin());
        if self.pen {
            self.lines
                .last_mut()
                .expect("Turtle closing without an active line!")
                .push(pos)
        }

        self.position = pos;
        self
    }

    fn left(mut self, angle: f64) -> Self {
        self.heading = self.heading + angle;
        self
    }

    fn right(mut self, angle: f64) -> Self {
        self.heading = self.heading - angle;
        self
    }

    fn pen_up(mut self) -> Self {
        self.pen = false;
        self.start = None;
        self
    }

    fn pen_down(mut self) -> Self {
        if self.pen {
            self
        } else {
            self.pen = true;
            self.start = Some(self.position.clone());
            self.lines.push(vec![self.position.clone()]);
            self
        }
    }

    fn close(mut self) -> Self {
        match self.start {
            Some(start) => {
                if self.pen {
                    self.lines
                        .last_mut()
                        .expect("Turtle closing without an active line!")
                        .push(
                            self.start
                                .expect("Turtle closing without a start point!")
                                .clone(),
                        )
                }
                self.position = start.clone();
                self
            }
            None => self,
        }
    }

    fn push(mut self) -> Self {
        self.stack.push(self.clone());
        self
    }

    fn pop(mut self) -> Self {
        match self.stack.pop() {
            Some(t) => Turtle {
                lines: self.lines,
                ..t
            },
            None => self,
        }
    }

    fn to_multiline(&mut self) -> MultiLineString<f64> {
        // MultiLineString::new(vec![])
        self.lines
            .iter()
            .map(|line| LineString::from(line.clone()))
            .collect()
    }

    fn to_polygon(&mut self) -> Result<Polygon<f64>, geo_types::Error> {
        match self.lines.len() {
            1 => Ok(Polygon::new(
                LineString::from(self.lines[0].clone()),
                vec![],
            )),
            _ => Err(geo_types::Error::MismatchedGeometry {
                expected: "Single linestring",
                found: "Multiple or zero linestrings",
            }),
        }
    }

    // fn to_multipolygon(self) -> Result<MultiPolygon<f64>, geo_types::Error> {
    //
    // }
    fn walk_lpath(mut self, lpath: &String, angle: f64, distance: f64) -> Self {
        for c in lpath.chars() {
            self = match c {
                '[' => self.push(),
                ']' => self.pop(),
                '-' => self.left(angle),
                '+' => self.right(angle),
                _ => self.fwd(distance),
            }
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use geo_types::Point;

    use crate::geo_types::PointDistance;
    use crate::l_system::LSystem;

    use super::{Turtle, TurtleTrait, degrees};

    #[test]
    fn test_walk_lsystem() {
        let t = Turtle::new().pen_down();
        let system = LSystem {
            axiom: "A".to_string(),
            rules: HashMap::from([('A', "A-B".to_string()), ('B', "A".to_string())]),
        };
        let expanded = system.expand(2);
        let t = t.walk_lpath(&expanded, degrees(90.0), 10.0);
        let last_point = t.lines.last().unwrap().last().unwrap();
        assert!(last_point.x().abs() <= 0.0001f64);
        assert!((&last_point.y() - 10.0f64).abs() <= 0.0001);
    }

    #[test]
    fn test_stack() {
        let t = Turtle::new();
        let result = t.push().fwd(100.0).right(degrees(90.0)).fwd(100.0).pop();
        assert!(result.position == Point::new(0.0f64, 0.0f64));
    }

    #[test]
    fn test_pendown() {
        let t = Turtle::new().pen_down();
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
        assert!(t.lines[0][0].distance(&Point::new(0.0f64, 0.0f64)) < 0.0001f64);
        assert!(t.lines[0][1].distance(&Point::new(100.0f64, 0.0f64)) < 0.0001f64);
        assert!(t.lines[0][2].distance(&Point::new(100.0f64, -100.0f64)) < 0.0001f64);
        assert!(t.lines[0][3].distance(&Point::new(0.0f64, -100.0f64)) < 0.0001f64);
        assert!(t.lines[0][4].distance(&Point::new(0.0f64, 0.0f64)) < 0.0001f64);
    }
}
