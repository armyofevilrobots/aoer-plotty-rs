use contour::ContourBuilder;
use geo::{Coord, MultiLineString, MultiPolygon, Rect, Scale, Translate};
use noise::{NoiseFn, Perlin};

/// A ContourField is an area filled with perlin generated contour lines.
/// The noise is clamped from 0.0 .. 1.0, and you pass thresholds in as
/// a list of "heights" in the noise where you want a band.
/// The ContourBuilder from the contour crate provides the contours.
pub struct ContourField {
    xy_step: f64,
    _seed: u32,
    bounds: Option<Rect<f64>>,
    thresholds: Vec<f64>,
    // contour_builder: Option<ContourBuilder>,
    // perlin: Perlin,
    noise: Box<dyn NoiseFn<[f64; 2]>>,
    perlin_scale: f64,
}

impl ContourField {
    /// Get the values array for building the contour.
    fn values(&self) -> (Vec<f64>, usize, usize) {
        let bounds = self
            .bounds
            .expect("Failed to initialize ContourField bounds.");
        let (minx, miny) = bounds.min().x_y();
        #[allow(unused_assignments)]
        let (mut x, mut y) = (minx, miny);
        let (xmax, ymax) = bounds.max().x_y();
        let mut values: Vec<f64> = Vec::new();
        let (mut xc, mut yc) = (0usize, 0usize);

        while y < ymax {
            yc += 1;
            xc = 0;
            x = minx;
            while x < xmax {
                xc += 1;
                // println!("{},{}:{},{}", x, y, xc, yc);
                values.push(
                    self.noise
                        .get([x * self.perlin_scale, y * self.perlin_scale])
                        / 2.
                        + 0.5,
                );
                x += self.xy_step;
            }
            y += self.xy_step;
        }
        (values, xc, yc)
    }

    /// The lines which follow a particular "elevation"
    /// in the noise map. returns a Vec of MultiLineStrings.
    /// Nice if you want lines, like for a hatch or something, that
    /// won't natively occlude when used in a drawing
    /// context.
    pub fn isolines(&mut self) -> Vec<MultiLineString> {
        let (values, xc, yc) = self.values();
        let cb = ContourBuilder::new(xc, yc, true);

        let mut lines = cb
            .lines(&values, &self.thresholds)
            .expect("Failed to unwrap contours.");
        lines
            .iter_mut()
            .map(|layer| {
                let layer_lines = layer.geometry().clone();
                layer_lines
                    .scale_around_point(self.xy_step, self.xy_step, Coord { x: 0., y: 0. })
                    .translate(self.bounds.unwrap().min().x, self.bounds.unwrap().min().y)
            })
            .collect::<Vec<MultiLineString<f64>>>()
    }

    /// IsoBands; polygons that enclose the entire area between two threshold/height values
    /// These are useful, but a naive assumption of a map is generally thinking about
    /// contours instead.
    pub fn isobands(&mut self) -> Vec<MultiPolygon> {
        let (values, xc, yc) = self.values();
        let cb = ContourBuilder::new(xc, yc, true);

        let mut bands = cb
            .isobands(&values, &self.thresholds)
            .expect("Failed to unwrap contours.");
        bands
            .iter_mut()
            .map(|layer| {
                let layer_polys = layer.clone().into_inner();
                layer_polys
                    .0
                    .scale_around_point(self.xy_step, self.xy_step, Coord { x: 0., y: 0. })
                    .translate(self.bounds.unwrap().min().x, self.bounds.unwrap().min().y)
            })
            .collect::<Vec<MultiPolygon<f64>>>()
    }

    /// These are polygons enclosing the entire area which lies
    /// above a particular "elevation".
    pub fn contours(&mut self) -> Vec<MultiPolygon> {
        let (values, xc, yc) = self.values();
        let cb = ContourBuilder::new(xc, yc, true);

        let mut contours = cb
            .contours(&values, &self.thresholds)
            .expect("Failed to unwrap contours.");
        contours
            .iter_mut()
            .map(|layer| {
                let layer_polys = layer.clone().into_inner();
                layer_polys
                    .0
                    .scale_around_point(self.xy_step, self.xy_step, Coord { x: 0., y: 0. })
                    .translate(self.bounds.unwrap().min().x, self.bounds.unwrap().min().y)
            })
            .collect::<Vec<MultiPolygon<f64>>>()
    }
}

impl Default for ContourField {
    fn default() -> Self {
        Self {
            xy_step: 1.,
            _seed: Default::default(),
            bounds: Default::default(),
            thresholds: vec![],
            noise: Box::new(Perlin::new()),
            perlin_scale: 1.,
        }
    }
}

/// The ContourFieldBuilder should be how you build every contour field.
/// It allows you to specify the bounds of your field, as well as seed,
/// noise function, etc. The noise function is especially neat, since
/// the rust Noise crate provides an amazing variety of noise functions.
/// Just the combinations/permutations of noise functions make for some
/// very interesting sketches.
pub struct ContourFieldBuilder {
    field: ContourField,
}

impl ContourFieldBuilder {
    pub fn new() -> ContourFieldBuilder {
        ContourFieldBuilder {
            field: ContourField::default(),
        }
    }

    pub fn noise(self, noise: Box<dyn NoiseFn<[f64; 2]>>) -> Self {
        Self {
            field: ContourField {
                noise,
                ..self.field
            },
        }
    }

    pub fn seed(self, seed: u32) -> Self {
        Self {
            field: ContourField {
                _seed: seed,
                ..self.field
            },
        }
    }

    pub fn bounds(self, rect: Rect) -> Self {
        Self {
            field: ContourField {
                bounds: Some(rect),
                ..self.field
            },
        }
    }

    pub fn xy_step(self, xy_step: f64) -> Self {
        Self {
            field: ContourField {
                xy_step,
                ..self.field
            },
        }
    }

    pub fn perlin_scale(self, perlin_scale: f64) -> Self {
        Self {
            field: ContourField {
                perlin_scale,
                ..self.field
            },
        }
    }

    pub fn thresholds(self, thresholds: Vec<f64>) -> Self {
        Self {
            field: ContourField {
                thresholds,
                ..self.field
            },
        }
    }

    pub fn build(self) -> ContourField {
        if self.field.bounds.is_none() {
            panic!("Failed to initialize contour builder bounds.");
        }
        // let rows = (self.field.bounds.unwrap().height() / self.field.xy_step).ceil() as usize;
        // let cols = (self.field.bounds.unwrap().width() / self.field.xy_step).ceil() as usize;
        // println!("XROWS YCOLS ARE {},{}", cols, rows);
        // self.field.contour_builder = Some(ContourBuilder::new(cols, rows, true));
        self.field
    }
}

#[cfg(test)]
pub mod test {
    use geo::Coord;

    use super::*;

    #[test]
    pub fn test_build() {
        let mut cf = ContourFieldBuilder::new()
            .bounds(Rect::new(Coord { x: 0., y: 0. }, Coord { x: 20., y: 20. }))
            .xy_step(1.)
            .seed(2)
            .perlin_scale(0.1)
            .thresholds(vec![0., 0.2, 0.4, 0.6, 0.8])
            .build();
        println!("LAYERS:\n{:?}", &mut cf.contours());
    }
}
