# aoer-plotty-rs

## AOER-PLOTTY-RS : ArmyOfEvilRobots pen-plotter art related tools and libraries

[![Package][package-img]][package-url] [![Documentation][documentation-img]][documentation-url]

This library contains a variety of tools used to make pen-plotter based art.
While it focuses on a combination of nannou and geo/geo_types for now, it
will likely expand into other areas as I find the need for my own creations.
Based (extremely roughly) on [`shapely`] for Python for the geographic
functions, with a splash of [`VSK`].

*CAUTION: This isn't even Alpha quality yet, and I am poking away at it on
my days off. May spontaneously explode, might take your plotter with it.*

[`shapely`]: https://github.com/shapely/shapely
[`vsk`]: https://vsketch.readthedocs.io/en/latest/index.html

## Changelog
* 0.1.x. Add SVG generation features. In progress
* 0.1.4. Add the [`geo_types::hatch::Hatch`]ing submodule.
* 0.1.3. Breaking change to GCode POST again; use an enum to define the
         the input geometry so that we can add new geometry source types,
         like svg2polyline polylines, or even multilayer geo with tool changes.
* 0.1.2. Mostly documentation improvements. Made MIT license explicit.
* 0.1.1. Breaking change to the GCode POST function to use geo_types in
         order to be consistent with everywhere else in the library.
         Also changed the Turtle/TurtleTrait to just be mutable.
* 0.1.0. Initial commit

[documentation-img]: https://docs.rs/aoer-plotty-rs/badge.svg
[documentation-url]: https://docs.rs/aoer-plotty-rs
[package-img]: https://img.shields.io/crates/v/aoer-plotty-rs.svg
[package-url]: https://crates.io/crates/aoer-plotty-rs

License: MIT
