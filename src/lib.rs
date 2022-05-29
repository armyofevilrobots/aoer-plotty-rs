//! ArmyOfEvilRobots pen-plotter art related tools and libraries
//!
//! This library contains a variety of tools used to make pen-plotter based art.
//! While it focuses on a combination of nannou and geo/geo_types for now, it
//! will likely expand into other areas as I find the need for my own creations.
//! Based (extremely roughly) on [`shapely`] for Python for the geographic
//! functions, with a splash of [`VSK`].
//!
//! *CAUTION: This isn't even Alpha quality yet, and I am poking away at it on
//! my days off. May spontaneously explode, might take your plotter with it.*
//!
//! [`shapely`]: https://github.com/shapely/shapely
//! [`vsk`]: https://vsketch.readthedocs.io/en/latest/index.html

/// Extensions/Traits for geo_types geometry. Also includes some helper functions
/// for working with Nannou and geo_types.
pub mod geo_types;

/// Turtle graphics implementation, including integration with L-systems
pub mod turtle;

/// L-system implementation, with expansion/recursion
pub mod l_system;

/// Make your life easy! Just import prelude::* and ignore all the warnings!
/// One stop shopping at the expense of a slightly more complex dependency graph.
pub mod prelude{

    pub use crate::geo_types::PointDistance;
    pub use crate::geo_types::nannou::NannouDrawer;
    pub use crate::l_system::LSystem;
    pub use crate::turtle::{Turtle, TurtleTrait};
}
