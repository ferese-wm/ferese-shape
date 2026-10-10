# ferese-shape

`ferese-shape` is a Rust library for rounded rectangles and squircles.

`Shape::Circular` gives rounded rectangles, circles and pills.
`Shape::Continuous` uses a measured three-cubic squircle profile. Shape
and radius are separate choices; zero radius gives a square corner in either mode.

## Use

Requires Rust 1.85 or newer. Add the crate to your `Cargo.toml`:

```toml
[dependencies]
ferese-shape = "0.1.0"
```

The [API reference](https://docs.rs/ferese-shape) documents the public types and
functions. This example calculates fill and border coverage:

```rust
use ferese_shape::{Outline, Shape, edge_coverage};

let outer = Outline::new(
    [0.0, 0.0, 100.0, 80.0], // x, y, width, height
    [12.0; 4],              // top-left, top-right, bottom-right, bottom-left
    Shape::Continuous,
).unwrap();
let inner = outer.inset(2.0).unwrap();
let point = [0.5, 20.5];

// Use physical pixels for a one-pixel antialiasing ramp.
let fill = edge_coverage(inner.signed_distance(point));
let border = edge_coverage(outer.signed_distance(point)) - fill;
assert!(border >= 0.0);
```

## Geometry

Bounds, radii, query points and inset distances use the caller's units. Calculations
use `f64`. Bounds and radii must be finite, and width and height must be positive.
Negative radii become zero; each radius is capped at half the shorter dimension.
Query points must be finite.

The squircle shoulder extends up to `1.52866498 × radius`. As shoulders approach
half the shorter dimension, the profile blends toward circular corners to avoid
overlap. At the maximum radius, circles and pills use exact arcs.

The squircle control points follow this
[measured UIKit approximation](https://liamrosenfeld.com/posts/apple_icon_quest/).
They meet at the same positions, but the internal joins have about a 2.45° tangent
jump and a curvature jump. `Continuous` names the profile; it does not promise
mathematical tangent or curvature continuity.

## Insets

`Outline::inset(distance)` keeps the original bounds, radii and profile. It adds
the distance to the signed distance from that reference outline. Positive values
move inward; negative values move outward. Repeated insets accumulate.

A border of width `w` uses the outer coverage minus the coverage at `d + w`, where
`d` is the outer signed distance. Rebuilding a smaller squircle with a smaller
radius does not give the same contour. A sufficiently deep inset can disappear.

`transformed(translation, scale)` applies a positive uniform scale to the outline
and its accumulated inset, then translates it. For pixel coverage, transform the
outline and query points to physical pixels before calling `edge_coverage`.

`polygon(tolerance)` samples the reference contour, including its inset, as a
closed polygon. It returns an empty polygon for a collapsed contour. Invalid
tolerances or exhausted subdivision budgets return `None`. For a tolerance of
0.25 physical pixels, either transform the outline to physical pixels first or
divide the tolerance by the output scale.

## GPU geometry

`WGSL` exports `shape_distance(point, bounds, radii, kind, inset)` and
`shape_coverage(distance)`. Kind `0` selects circular corners; `1` selects the
squircle profile. Coordinates use the same units as the CPU outline. Use physical
pixels when evaluating edge coverage.

The shader uses the same control points as the CPU geometry. Your renderer
supplies transforms, colors, batching, and clipping.

[Changelog](CHANGELOG.md) · [MIT license](LICENSE)
