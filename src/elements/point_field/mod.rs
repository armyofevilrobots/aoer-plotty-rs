pub mod halton;
pub use halton::*;
pub mod perlin;
pub use perlin::*;
use std::fmt::Debug;

pub trait PointField: Debug + Send + Sync + Iterator {}
