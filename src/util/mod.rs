pub mod halton;
pub use halton::HaltonSequence;

pub trait AnythingToGeo {
    fn to_geo(&self) -> geo::Geometry;
}
