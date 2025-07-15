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
* 0.2.3. Add support for hatch scale, so that it isn't always just
         the pen width used to define the space between lines.
* 0.2.2. Bugfixens! The regular_poly_native function was duplicating
         points, resulting in invalid geometries.
* 0.2.1. Optimizations and plotters:
  * Add optimization to toolpaths/line-ordering so that we don't waste
    valuable time traversing empty space when plotting.
  * Add simple serial-plotter support
* 0.2.0. We made it!!!!
  * Typography module. It's buggy and ugly, but we can now place simple
    text on the sketches.
  * Performance improvements; particularly on complex overlapping
    geometries
  * Truchet tiles (Carlson Smith)
  * Example of using a UI in Nannou to customize a sketch
* 0.1.11. Added the first "element" (reusable sketch component) in the form
         of the [`elements::CarlsonSmithTruchet`], which provides tileable
         and scalable truchets which make for some very interesting patterns.
         Think of them as the "goto 10" tiles on steroids.
         Also added a to_geos trait which makes it easy to convert
         back and forth from geo_types without fancy and unpredictable
         From/Into magic.
         Also added a [`geo_types::shapes`] module which provides some
         additional primitives (arc, polygons, circles).
         Added the [`geo_types::boolean::BooleanOp`] trait to allow for
         boolean operations directly against geo_types.
* 0.1.10. Getting close to having to do a 0.2 release. Added the 'flatten'
         method to [`context::Context`] so that you can merge all your
         pen strokes that live on the same layer. Good for merging
         overlapping polygons. Layer is defined as "exact same color, pen,
         and fill configuration"
* 0.1.9. Add masking to context: You can now mask the drawable area with
         any [`geo_types::Geometry`] variant and only areas under the mask
         will actually render. Also changed some performance and accuracy
         related optimizations so that clipped items look clean.
         Also added the final Generative Artistry examples. I'll miss
         implementing those :(
* 0.1.8. Add a bunch of new features to Context, including regular polygons
         and tesselated polys (stars of various point counts). Circles are
         now somewhat simpler as well.
* 0.1.7. Another big change. Added the Context drawing library, which is HUGE,
  and contains way too much functionality to discuss here.
* 0.1.6. Various changes:
  * Add [`geo_types::buffer::Buffer`] trait to offset polygons
  * Add [`geo_types::clip::LineClip`] trait to Clip geometry with
    OTHER geometry.
  * [`geo_types::svg::Arrangement`] trait has been extended to
    better support arbitrary transformations.
  * [`geo_types::svg::Arrangement`] trait has also added a margin
    option to fit geometry on a page with predefined margins.
  * [`geo_types::hatch::OutlineStroke`] is a utility Trait which
    takes a LineString/MultiLineString and applies a stroke, returning
    a MultiLineString containing lines which outline the stroked line
  * [`geo_types::hatch::OutlineFillStroke`] is the same as [`geo_types::hatch::OutlineStroke`]
    except it also fills the stroke with the given HatchPattern. Great for turning
    drawings made of thick lines into nicely filled polygons.
  * Tons more examples which are oxidized versions of the
    [`Generative Artistry tutorials`]
* 0.1.5. Add SVG generation features.
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
[`Generative Artistry tutorials`]: https://generativeartistry.com/tutorials/

License: MIT
