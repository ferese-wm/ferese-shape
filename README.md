# ferese-shape

`ferese-shape` is a Rust library for rounded rectangles and squircles. Choose
`Shape::Circular` for rounded rectangles, circles and pills, or
`Shape::Continuous` for a measured three-cubic squircle profile. The radius is
independent of that choice, so setting it to zero gives a square corner in
either mode.

## Use

To use the crate, you'll need Rust 1.85 or newer and this dependency in your
`Cargo.toml`:

```toml
[dependencies]
ferese-shape = "0.1.0"
```

The example below calculates fill and border coverage; the
[API reference](https://docs.rs/ferese-shape) covers the public types and functions.

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

Bounds, radii, query points and inset distances use the caller's units, with
calculations performed in `f64`. Bounds, radii and query points must be finite,
and width and height must be positive. Negative radii become zero, and each
radius is capped at half the shorter dimension.

Because a squircle's shoulder can extend up to `1.52866498 × radius`, the profile
blends toward circular corners as the shoulders approach half the shorter
dimension. This keeps the shoulders from overlapping, with exact arcs for
circles and pills at the maximum radius.

The squircle control points come from this
[measured UIKit approximation](https://liamrosenfeld.com/posts/apple_icon_quest/).
Adjacent segments meet at the same point but have about a 2.45° tangent jump and
a curvature jump, so the name `Continuous` does not imply mathematical tangent
or curvature continuity.

## Insets

Calling `Outline::inset(distance)` changes the contour without rebuilding the
shape. The original bounds, radii and profile stay intact as the method adds the
distance to their signed-distance field. Positive values pull the edge inward,
negative values push it outward, and repeated calls add to the stored offset.

For a border of width `w`, subtract the coverage at `d + w` from the outer
coverage, where `d` is the outer signed distance. The inset follows that original
distance field, which is why rebuilding a smaller squircle with a smaller radius
does not give the same contour. A sufficiently deep inset can disappear entirely.

`transformed(translation, scale)` scales the original outline and its accumulated
inset by a positive uniform factor, then translates the result. When using
`edge_coverage`, work in physical pixels by transforming both the outline and
your query points into those units.

`polygon(tolerance)` samples the reference contour, including any inset, into a
closed polygon with the tolerance expressed in the outline's units. For a
tolerance of 0.25 physical pixels, either transform the outline to physical
pixels first or divide the tolerance by the output scale. A collapsed contour
produces an empty polygon, while an invalid tolerance or an exhausted
subdivision budget produces `None`.

## GPU geometry

`WGSL` exports `shape_distance(point, bounds, radii, kind, inset)` and
`shape_coverage(distance)` for evaluating the outlines on the GPU. Set `kind` to
`0` for circular corners or `1` for the squircle profile.

The shader shares its control points and coordinate units with the CPU geometry;
your renderer handles the transforms, colors, batching and clipping around these
functions. Edge coverage expects distances in physical pixels on both the CPU
and GPU.

[Changelog](CHANGELOG.md) · [MIT license](LICENSE)
