use contour::ContourBuilder;
use geo::{Coord, MultiLineString, MultiPolygon, Rect, Scale, Translate};
use noise::{NoiseFn, Perlin};

pub struct ContourField {
    xy_step: f64,
    seed: u32,
    bounds: Option<Rect<f64>>,
    thresholds: Vec<f64>,
    // contour_builder: Option<ContourBuilder>,
    perlin: Perlin,
    perlin_scale: f64,
}

impl ContourField {
    fn values(&self) -> (Vec<f64>, usize, usize) {
        let bounds = self
            .bounds
            .expect("Failed to initialize ContourField bounds.");
        let (minx, miny) = bounds.min().x_y();
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
                    self.perlin
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
            seed: Default::default(),
            bounds: Default::default(),
            thresholds: vec![],
            // contour_builder: None,
            perlin: Perlin::new(),
            perlin_scale: 1.,
        }
    }
}

pub struct ContourFieldBuilder {
    field: ContourField,
}

impl ContourFieldBuilder {
    pub fn new() -> ContourFieldBuilder {
        ContourFieldBuilder {
            field: ContourField::default(),
        }
    }

    pub fn seed(self, seed: u32) -> Self {
        Self {
            field: ContourField { seed, ..self.field },
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

    pub fn build(mut self) -> ContourField {
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
