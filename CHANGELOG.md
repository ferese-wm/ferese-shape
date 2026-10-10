# Changelog

## 0.1.1

This release corrects the README and crate description.

## 0.1.0

Initial release.

- Circular and measured three-cubic continuous corners, with independent radii
  for each corner and circular blending when shoulders would overlap.
- Signed-distance queries, accumulated inward and outward insets, uniform
  scaling and translation, and a one-physical-pixel coverage ramp.
- Adaptive polygon sampling of the reference contour and its insets.
- WGSL distance and coverage functions generated from the CPU profile constants.
- Corner-distance searches that skip distant curves using symmetry and
  conservative bounds.
