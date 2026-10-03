#![doc = include_str!("../README.md")]

mod path;
mod profile;

pub use profile::{CIRCULAR_BLEND, CONTINUOUS, Cubic, EXTENT, Point};

/// WGSL distance and coverage functions, with controls generated from the CPU profile.
pub const WGSL: &str = include_str!(concat!(env!("OUT_DIR"), "/shape.wgsl"));

/// Shape is independent of the numerical corner radii.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u32)]
pub enum Shape {
    #[default]
    Circular = 0,
    Continuous = 1,
}

/// An inset retains its original outline instead of rebuilding a smaller curve.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Outline {
    bounds: [f64; 4],
    radii: [f64; 4],
    shape: Shape,
    inset: f64,
}

impl Outline {
    /// Radii are top-left, top-right, bottom-right, bottom-left.
    /// Each radius is clamped to half the shorter dimension, as in Ferese.
    pub fn new(bounds: [f64; 4], radii: [f64; 4], shape: Shape) -> Option<Self> {
        if !bounds.into_iter().chain(radii).all(f64::is_finite)
            || bounds[2] <= 0.
            || bounds[3] <= 0.
            || !(bounds[0] + bounds[2]).is_finite()
            || !(bounds[1] + bounds[3]).is_finite()
        {
            return None;
        }

        let limit = bounds[2].min(bounds[3]) * 0.5;
        Some(Self {
            bounds,
            radii: radii.map(|r| r.clamp(0., limit)),
            shape,
            inset: 0.,
        })
    }

    pub fn bounds(self) -> [f64; 4] {
        self.bounds
    }

    pub fn radii(self) -> [f64; 4] {
        self.radii
    }

    pub fn shape(self) -> Shape {
        self.shape
    }

    pub fn inset_distance(self) -> f64 {
        self.inset
    }

    /// Positive distances inset; negative distances expand the same outline.
    pub fn inset(self, distance: f64) -> Option<Self> {
        let inset = self.inset + distance;
        inset.is_finite().then_some(Self { inset, ..self })
    }

    /// Translate and uniformly scale the reference outline and accumulated inset.
    pub fn transformed(self, translation: Point, scale: f64) -> Option<Self> {
        if !scale.is_finite() || scale <= 0. {
            return None;
        }

        let bounds = [
            self.bounds[0] * scale + translation[0],
            self.bounds[1] * scale + translation[1],
            self.bounds[2] * scale,
            self.bounds[3] * scale,
        ];
        Self::new(bounds, self.radii.map(|r| r * scale), self.shape)?.inset(self.inset * scale)
    }

    /// Negative inside. Insets add their accumulated distance to the distance
    /// from the original boundary. The query point must have finite coordinates.
    pub fn signed_distance(self, point: Point) -> f64 {
        let p = [point[0] - self.bounds[0], point[1] - self.bounds[1]];
        let [_, _, width, height] = self.bounds;
        let limit = width.min(height) * 0.5;
        let radius = self.radii[0];

        if self.radii.iter().all(|r| *r == radius)
            && (self.shape == Shape::Circular || radius == 0. || radius >= limit)
        {
            let q = [
                (p[0] - width * 0.5).abs() - width * 0.5 + radius,
                (p[1] - height * 0.5).abs() - height * 0.5 + radius,
            ];
            return q[0].max(0.).hypot(q[1].max(0.)) + q[0].max(q[1]).min(0.) - radius + self.inset;
        }

        let extents = self.radii.map(|r| corner_extent(r, limit, self.shape));
        let edge_inset = [
            width * 0.5 - (p[0] - width * 0.5).abs(),
            height * 0.5 - (p[1] - height * 0.5).abs(),
        ];

        // Outside every corner box, the nearest contour is a straight edge.
        if edge_inset[0].max(edge_inset[1]) >= extents.into_iter().fold(0., f64::max) {
            return -edge_inset[0].min(edge_inset[1]) + self.inset;
        }

        let lines = [
            [[extents[0], 0.], [width - extents[1], 0.]],
            [[width, extents[1]], [width, height - extents[2]]],
            [[width - extents[2], height], [extents[3], height]],
            [[0., height - extents[3]], [0., extents[0]]],
        ];
        let mut distance = lines
            .into_iter()
            .map(|[a, b]| line_distance(p, a, b))
            .fold(f64::INFINITY, f64::min);
        let queries = [
            p,
            [width - p[0], p[1]],
            [width - p[0], height - p[1]],
            [p[0], height - p[1]],
        ];
        let mut inside = p[0] >= 0. && p[1] >= 0. && p[0] <= width && p[1] <= height;

        for (index, query) in queries.into_iter().enumerate() {
            let radius = self.radii[index];

            if radius == 0. {
                continue;
            }

            let extent = extents[index];
            let circular = self.shape == Shape::Circular || radius >= limit;

            if inside && query[0] < extent && query[1] < extent {
                inside = if circular {
                    (query[0] - radius).hypot(query[1] - radius) <= radius
                } else {
                    corner_contains([query[0] / radius, query[1] / radius], blend(radius, limit))
                };
            }

            // A conservative corner-box bound avoids solving distant cubics.
            let lower =
                outside_interval(query[0], extent).hypot(outside_interval(query[1], extent));

            if lower >= distance {
                continue;
            }

            let candidate = if circular {
                if query[0] <= radius && query[1] <= radius {
                    ((query[0] - radius).hypot(query[1] - radius) - radius).abs()
                } else {
                    length(sub(query, [0., radius])).min(length(sub(query, [radius, 0.])))
                }
            } else {
                let q = [query[0] / radius, query[1] / radius];
                corner_distance(q, blend(radius, limit), distance / radius) * radius
            };
            distance = distance.min(candidate);
        }

        if inside {
            -distance + self.inset
        } else {
            distance + self.inset
        }
    }
}

pub fn edge_coverage(physical_distance: f64) -> f64 {
    (0.5 - physical_distance).clamp(0., 1.)
}

pub fn blend(radius: f64, limit: f64) -> f64 {
    if radius <= 0. {
        return 0.;
    }

    ((limit / radius - 1.) / (EXTENT - 1.)).clamp(0., 1.)
}

pub fn corner_extent(radius: f64, limit: f64, shape: Shape) -> f64 {
    if shape == Shape::Circular {
        return radius;
    }

    radius * (1. + (EXTENT - 1.) * blend(radius, limit))
}

pub fn controls(blend: f64) -> [Cubic; 3] {
    std::array::from_fn(|segment| {
        std::array::from_fn(|point| {
            let a = CIRCULAR_BLEND[segment][point];
            let b = CONTINUOUS[segment][point];
            [a[0] + (b[0] - a[0]) * blend, a[1] + (b[1] - a[1]) * blend]
        })
    })
}

fn sub(a: Point, b: Point) -> Point {
    [a[0] - b[0], a[1] - b[1]]
}

fn length(p: Point) -> f64 {
    p[0].hypot(p[1])
}

fn dot(a: Point, b: Point) -> f64 {
    a[0] * b[0] + a[1] * b[1]
}

fn outside_interval(value: f64, extent: f64) -> f64 {
    (-value).max(value - extent).max(0.)
}

fn line_distance(p: Point, a: Point, b: Point) -> f64 {
    let delta = sub(b, a);
    let squared = dot(delta, delta);

    if squared == 0. {
        return length(sub(p, a));
    }

    let t = (dot(sub(p, a), delta) / squared).clamp(0., 1.);
    length(sub(p, [a[0] + delta[0] * t, a[1] + delta[1] * t]))
}

fn sample(c: Cubic, t: f64) -> (Point, Point, Point) {
    let position: [[f64; 3]; 2] = std::array::from_fn(|i| {
        let first = 3. * (c[1][i] - c[0][i]);
        let second = 3. * (c[2][i] - 2. * c[1][i] + c[0][i]);
        let third = c[3][i] - 3. * c[2][i] + 3. * c[1][i] - c[0][i];
        [
            c[0][i] + t * (first + t * (second + t * third)),
            first + t * (2. * second + 3. * t * third),
            2. * second + 6. * t * third,
        ]
    });
    (
        [position[0][0], position[1][0]],
        [position[0][1], position[1][1]],
        [position[0][2], position[1][2]],
    )
}

fn corner_distance(p: Point, blend: f64, mut nearest: f64) -> f64 {
    // Reflection across the diagonal preserves this profile. For x <= y,
    // the reflected shoulder is at least as close as the opposite shoulder.
    let p = [p[0].min(p[1]), p[0].max(p[1])];
    for c in controls(blend).into_iter().take(2) {
        // A Bezier lies in its control hull. The hull's bounding box gives
        // a lower bound without solving the curve.
        if control_box_distance_squared(c, p) < nearest * nearest {
            nearest = nearest.min(nearest_distance(c, p));
        }
    }
    nearest
}

fn control_box_distance_squared(c: Cubic, p: Point) -> f64 {
    let low: Point =
        std::array::from_fn(|axis| c.into_iter().map(|p| p[axis]).fold(f64::INFINITY, f64::min));
    let high: Point = std::array::from_fn(|axis| {
        c.into_iter()
            .map(|p| p[axis])
            .fold(f64::NEG_INFINITY, f64::max)
    });
    let delta = [
        (low[0] - p[0]).max(p[0] - high[0]).max(0.),
        (low[1] - p[1]).max(p[1] - high[1]).max(0.),
    ];
    dot(delta, delta)
}

fn nearest_distance(c: Cubic, p: Point) -> f64 {
    [0., 1.]
        .into_iter()
        .map(|mut t| {
            for _ in 0..8 {
                let (point, tangent, acceleration) = sample(c, t);
                let error = sub(point, p);
                let speed = dot(tangent, tangent);
                let denominator = (speed + dot(error, acceleration)).max(speed * 0.25);
                let next =
                    (t - (dot(error, tangent) / denominator).clamp(-0.25, 0.25)).clamp(0., 1.);
                if next == t {
                    break;
                }
                t = next;
            }

            length(sub(sample(c, t).0, p))
        })
        .fold(f64::INFINITY, f64::min)
}

fn corner_contains(p: Point, blend: f64) -> bool {
    for c in controls(blend) {
        if p[0] > c[3][0] {
            continue;
        }

        // Both blended profiles are monotone in x and y. Queries above
        // or below this segment's endpoint box need no inversion.
        if p[1] >= c[0][1] {
            return true;
        }
        if p[1] < c[3][1] {
            return false;
        }

        let (mut low, mut high) = (0., 1.);

        for _ in 0..32 {
            let t = (low + high) * 0.5;

            if sample(c, t).0[0] < p[0] {
                low = t;
            } else {
                high = t;
            }
        }

        return p[1] >= sample(c, (low + high) * 0.5).0[1];
    }

    true
}

#[cfg(test)]
mod optimization_tests {
    use super::*;

    #[test]
    fn pruned_search_matches_all_segments_including_deep_interiors() {
        for blend in [0., 0.01, 0.25, 0.5, 0.75, 0.99, 1.] {
            let curves = controls(blend);
            for y in -40..81 {
                for x in -40..81 {
                    let p = [x as f64 / 20., y as f64 / 20.];
                    let reference = curves
                        .into_iter()
                        .map(|c| nearest_distance(c, p))
                        .fold(f64::INFINITY, f64::min);
                    for bound in [f64::INFINITY, 0.1, 0.75, 2.] {
                        let actual = corner_distance(p, blend, bound);
                        assert!(
                            (actual - reference.min(bound)).abs() < 1e-12,
                            "blend={blend} p={p:?} bound={bound}: {actual} vs {reference}"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn endpoint_containment_shortcuts_match_inverse_search() {
        for blend in [0., 0.25, 0.5, 0.75, 1.] {
            for y in 0..81 {
                for x in 0..81 {
                    let p = [x as f64 / 50., y as f64 / 50.];
                    let reference = controls(blend)
                        .into_iter()
                        .find(|c| p[0] <= c[3][0])
                        .map_or(true, |c| {
                            let (mut low, mut high) = (0., 1.);
                            for _ in 0..32 {
                                let t = (low + high) * 0.5;
                                if sample(c, t).0[0] < p[0] {
                                    low = t;
                                } else {
                                    high = t;
                                }
                            }
                            p[1] >= sample(c, (low + high) * 0.5).0[1]
                        });
                    assert_eq!(
                        corner_contains(p, blend),
                        reference,
                        "blend={blend} p={p:?}"
                    );
                }
            }
        }
    }
}
