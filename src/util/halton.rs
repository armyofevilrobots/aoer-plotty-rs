/// Halton sequence
///

pub struct HaltonSequence {
    numer: usize,
    denom: usize,
    x: usize,
    y: usize,
    base: usize,
}

impl Default for HaltonSequence {
    fn default() -> Self {
        Self {
            numer: 0,
            denom: 1,
            x: 0,
            y: 0,
            base: 2,
        }
    }
}

impl HaltonSequence {
    pub fn with_base(base: usize) -> HaltonSequence {
        HaltonSequence {
            base,
            ..HaltonSequence::default()
        }
    }
}

impl Iterator for HaltonSequence {
    type Item = f64;

    fn next(&mut self) -> Option<Self::Item> {
        self.x = self.denom - self.numer;
        if self.x == 1 {
            self.numer = 1;
            self.denom *= self.base;
        } else {
            self.y = self.denom / self.base;
            while self.x <= self.y {
                self.y /= self.base;
            }
            self.numer = (self.base + 1) * self.y - self.x;
        }
        Some(self.numer as f64 / self.denom as f64)
    }
}

#[cfg(test)]
pub mod test {
    use super::HaltonSequence;

    #[test]
    fn test_halton_base2() {
        let seq = HaltonSequence::with_base(2)
            .skip(0)
            .take(10)
            .collect::<Vec<f64>>();
        println!("{:?}", seq);
        assert!(seq == vec![0.5, 0.25, 0.75, 0.125, 0.625, 0.375, 0.875, 0.0625, 0.5625, 0.3125]);
    }
}
