use crate::geo_types::ToGeos;
use geo_types::Geometry;
use geos::Geom;
use std::error::Error;

/// Boolean operations trait. Used to give boolean caps to geo_types
/// shapes. Basically just a wrapper on geos boolean ops.
pub trait BooleanOp
where
    Self: Sized,
{
    /// Subtract other from self
    fn difference(&self, other: &Self) -> Result<Self, Box<dyn Error>>;

    /// The combination of both other and self
    fn union(&self, other: &Self) -> Result<Self, Box<dyn Error>>;

    /// Returns only the portion of self that overlaps other
    fn intersection(&self, other: &Self) -> Result<Self, Box<dyn Error>>;

    /// Unary union; faster method of unioning a whole geometry collection
    fn unary_union(&self) -> Result<Self, Box<dyn Error>>;
}

impl BooleanOp for Geometry<f64> {
    fn difference(&self, other: &Self) -> Result<Self, Box<dyn Error>> {
        let geos_self = self.to_geos()?;
        let geos_other = other.to_geos()?;
        Ok(Geometry::try_from(geos_self.difference(&geos_other)?)?)
    }

    fn union(&self, other: &Self) -> Result<Self, Box<dyn Error>> {
        let geos_self = self.to_geos()?;
        let geos_other = other.to_geos()?;
        Ok(Geometry::try_from(geos_self.union(&geos_other)?)?)
    }

    fn intersection(&self, other: &Self) -> Result<Self, Box<dyn Error>> {
        let geos_self = self.to_geos()?;
        let geos_other = other.to_geos()?;
        Ok(Geometry::try_from(geos_self.intersection(&geos_other)?)?)
    }

    fn unary_union(&self) -> Result<Self, Box<dyn Error>> {
        let geos_self = self.to_geos()?;
        Ok(Geometry::try_from(geos_self.unary_union()?)?)
    }
}
