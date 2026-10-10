# Changelog

## 0.1.0

Initial release of the corner geometry shared by Ferese and its Iced fork.

- Circular and measured three-cubic continuous corners, with independent radii
  for each corner and circular blending when shoulders would overlap.
- Signed-distance queries, accumulated inward and outward insets, uniform
  scaling and translation, and a one-physical-pixel coverage ramp.
- Adaptive polygon sampling of the reference contour and its insets.
- WGSL distance and coverage functions generated from the CPU profile constants.
- Corner-distance searches that skip distant curves using symmetry and
  conservative bounds.

Tests compare geometry against independent references, including mixed radii,
fractional scales, circles, pills, insets and polygon paths. The README records
the measured profile's join discontinuities and the test tolerances.
