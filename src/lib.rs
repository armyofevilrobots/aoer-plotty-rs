//! # AOER-PLOTTY-RS : ArmyOfEvilRobots pen-plotter art related tools and libraries
//!
//! [![Package][package-img]][package-url] [![Documentation][documentation-img]][documentation-url]
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
//!
//! # Changelog
//! * 0.1.6. Various changes:
//!   * Add [`geo_types::buffer::Buffer`] trait to offset polygons
//!   * Add [`geo_types::clip::LineClip`] trait to Clip geometry with
//!     OTHER geometry.
//!   * [`geo_types::svg::Arrangement`] trait has been extended to
//!     better support arbitrary transformations.
//!   * [`geo_types::svg::Arrangement`] trait has also added a margin
//!     option to fit geometry on a page with predefined margins.
//!   * [`geo_types::hatch::OutlineStroke`] is a utility Trait which
//!     takes a LineString/MultiLineString and applies a stroke, returning
//!     a MultiLineString containing lines which outline the stroked line
//!   * [`geo_types::hatch::OutlineFillStroke`] is the same as [`geo_types::hatch::OutlineStroke`]
//!     except it also fills the stroke with the given HatchPattern. Great for turning
//!     drawings made of thick lines into nicely filled polygons.
//!   * Tons more examples which are oxidized versions of the
//!     [`Generative Artistry tutorials`]
//! * 0.1.5. Add SVG generation features.
//! * 0.1.4. Add the [`geo_types::hatch::Hatch`]ing submodule.
//! * 0.1.3. Breaking change to GCode POST again; use an enum to define the
//!          the input geometry so that we can add new geometry source types,
//!          like svg2polyline polylines, or even multilayer geo with tool changes.
//! * 0.1.2. Mostly documentation improvements. Made MIT license explicit.
//! * 0.1.1. Breaking change to the GCode POST function to use geo_types in
//!          order to be consistent with everywhere else in the library.
//!          Also changed the Turtle/TurtleTrait to just be mutable.
//! * 0.1.0. Initial commit
//!
//! [documentation-img]: https://docs.rs/aoer-plotty-rs/badge.svg
//! [documentation-url]: https://docs.rs/aoer-plotty-rs
//! [package-img]: https://img.shields.io/crates/v/aoer-plotty-rs.svg
//! [package-url]: https://crates.io/crates/aoer-plotty-rs
//! [`Generative Artistry tutorials`]: https://generativeartistry.com/tutorials/

/// Extensions/Traits for geo_types geometry. Also includes some helper functions
/// for working with Nannou and geo_types.
pub mod geo_types;

/// Turtle graphics implementation, including integration with L-systems
pub mod turtle;

/// L-system implementation, with expansion/recursion
pub mod l_system;

/// gcode module provides a simple post-processor for line-based art to be converted
/// into GCode
pub mod gcode;

/// Make your life easy! Just import prelude::* and ignore all the warnings!
/// One stop shopping at the expense of a slightly more complex dependency graph.
pub mod prelude {
    pub use crate::geo_types::nannou::NannouDrawer;
    pub use crate::geo_types::PointDistance;
    pub use crate::l_system::LSystem;
    pub use crate::turtle::{Turtle, TurtleTrait};
    pub use crate::geo_types::hatch::*;
    pub use crate::geo_types::svg::*;
}
