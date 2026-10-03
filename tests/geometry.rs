use ferese_shape::{Outline, Shape, edge_coverage};

// Independent fixture and polygon oracle; no production controls or Newton solver.
const PROFILE: [[[f64; 2]; 4]; 3] = [
    [
        [0., 1.52866498],
        [0., 1.08849296],
        [0., 0.86840694],
        [0.07491139, 0.63149379],
    ],
    [
        [0.07491139, 0.63149379],
        [0.16905956, 0.37282383],
        [0.37282383, 0.16905956],
        [0.63149379, 0.07491139],
    ],
    [
        [0.63149379, 0.07491139],
        [0.86840694, 0.],
        [1.08849296, 0.],
        [1.52866498, 0.],
    ],
];
const CIRCLE: [[[f64; 2]; 4]; 3] = [
    [
        [0., 1.],
        [0., 0.8686781289],
        [0.0258657631, 0.7386421566],
        [0.0761204675, 0.6173165676],
    ],
    [
        [0.0761204675, 0.6173165676],
        [0.1776144241, 0.3722884810],
        [0.3722884810, 0.1776144241],
        [0.6173165676, 0.0761204675],
    ],
    [
        [0.6173165676, 0.0761204675],
        [0.7386421566, 0.0258657631],
        [0.8686781289, 0.],
        [1., 0.],
    ],
];

fn curve(mut points: [[f64; 2]; 4], t: f64) -> [f64; 2] {
    for count in (1..4).rev() {
        for i in 0..count {
            points[i] =
                std::array::from_fn(|axis| points[i][axis] * (1. - t) + points[i + 1][axis] * t);
        }
    }

    points[0]
}

fn polygon(bounds: [f64; 4], radii: [f64; 4], shape: Shape) -> Vec<[f64; 2]> {
    let limit = bounds[2].min(bounds[3]) / 2.;
    let mut path = Vec::new();

    for (corner, radius) in radii.into_iter().enumerate() {
        let r = radius.clamp(0., limit);
        let mut points = Vec::new();

        if shape == Shape::Circular || r >= limit || r == 0. {
            for step in 0..=256 {
                let angle = std::f64::consts::PI + std::f64::consts::FRAC_PI_2 * step as f64 / 256.;
                points.push([r * (1. + angle.cos()), r * (1. + angle.sin())]);
            }
        } else {
            let blend = ((limit / r - 1.) / 0.52866498).clamp(0., 1.);

            for segment in 0..3 {
                let controls = std::array::from_fn(|index| {
                    std::array::from_fn(|axis| {
                        r * (CIRCLE[segment][index][axis] * (1. - blend)
                            + PROFILE[segment][index][axis] * blend)
                    })
                });

                for step in 0..=128 {
                    points.push(curve(controls, step as f64 / 128.));
                }
            }
        }

        if corner == 1 || corner == 3 {
            points.reverse();
        }

        path.extend(points.into_iter().map(|p| {
            [
                bounds[0]
                    + if corner == 1 || corner == 2 {
                        bounds[2] - p[0]
                    } else {
                        p[0]
                    },
                bounds[1] + if corner >= 2 { bounds[3] - p[1] } else { p[1] },
            ]
        }));
    }

    path
}

fn reference(point: [f64; 2], path: &[[f64; 2]]) -> f64 {
    let mut distance = f64::INFINITY;
    let mut inside = false;

    for (&a, &b) in path.iter().zip(path.iter().cycle().skip(1)) {
        let edge = [b[0] - a[0], b[1] - a[1]];
        let squared = edge[0] * edge[0] + edge[1] * edge[1];
        let t = if squared == 0. {
            0.
        } else {
            (((point[0] - a[0]) * edge[0] + (point[1] - a[1]) * edge[1]) / squared).clamp(0., 1.)
        };
        distance =
            distance.min((point[0] - a[0] - t * edge[0]).hypot(point[1] - a[1] - t * edge[1]));

        if (a[1] > point[1]) != (b[1] > point[1])
            && point[0] < a[0] + (point[1] - a[1]) * edge[0] / edge[1]
        {
            inside = !inside;
        }
    }

    if inside { -distance } else { distance }
}

#[test]
fn distance_matches_independent_curves_and_winding() {
    for shape in [Shape::Circular, Shape::Continuous] {
        for radii in [
            [0.; 4],
            [0.375; 4],
            [8.25; 4],
            [24.; 4],
            [29.5; 4],
            [0., 29.5, 3., 18.],
            [28., 2., 21., 0.],
        ] {
            for scale in [1., 1.25, 1.5] {
                let bounds = [1.25 * scale, 2.5 * scale, 61.5 * scale, 59. * scale];
                let radii = radii.map(|r| r * scale);
                let outline = Outline::new(bounds, radii, shape).unwrap();
                let oracle = polygon(bounds, radii, shape);

                for y in (0..95).step_by(3) {
                    for x in (0..98).step_by(3) {
                        let p = [x as f64 + 0.5, y as f64 + 0.5];
                        let expected = reference(p, &oracle);
                        let actual = outline.signed_distance(p);
                        assert!(
                            (actual - expected).abs() < 0.01,
                            "{shape:?} radii={radii:?} p={p:?}: {actual} != {expected}"
                        );

                        for inset in [0., 0.75, 2.25, 12., 32.] {
                            let alpha =
                                edge_coverage(outline.inset(inset).unwrap().signed_distance(p));
                            assert!((alpha - edge_coverage(expected + inset)).abs() <= 2. / 255.);
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn insets_keep_the_reference_curve_and_accumulate() {
    let outline = Outline::new([2.5, 3.25, 100., 80.], [12.; 4], Shape::Continuous).unwrap();
    let child = outline.inset(6.).unwrap().inset(3.).unwrap();
    assert_eq!(child.bounds(), outline.bounds());
    assert_eq!(child.radii(), outline.radii());
    assert_eq!(child.inset_distance(), 9.);

    for p in [[2.5, 3.25], [20., 10.], [50., 40.]] {
        assert!((child.signed_distance(p) - outline.signed_distance(p) - 9.).abs() < 1e-12);
    }
}

#[test]
fn mixed_radii_deep_insets_match_the_whole_boundary() {
    for shape in [Shape::Circular, Shape::Continuous] {
        let outline = Outline::new([0., 0., 100., 100.], [0., 50., 5., 28.], shape).unwrap();
        let path = polygon(outline.bounds(), outline.radii(), shape);

        for p in [[45., 45.], [60., 40.], [32., 64.], [50., 50.]] {
            let expected = reference(p, &path);

            for inset in [0., 10., 44., 55.] {
                let actual = outline.inset(inset).unwrap().signed_distance(p);
                assert!((actual - expected - inset).abs() < 0.001);
            }
        }
    }
}

#[test]
fn rebuilding_a_smaller_curve_is_not_a_constant_distance_inset() {
    let outer = Outline::new([0., 0., 100., 80.], [20.; 4], Shape::Continuous).unwrap();
    let inset = outer.inset(8.).unwrap();
    let rebuilt = Outline::new([8., 8., 84., 64.], [12.; 4], Shape::Continuous).unwrap();
    let p = [20., 20.];
    let oracle = reference(p, &polygon(outer.bounds(), outer.radii(), outer.shape()));
    assert!((inset.signed_distance(p) - oracle - 8.).abs() < 0.001);
    assert!((inset.signed_distance(p) - rebuilt.signed_distance(p)).abs() > 0.1);
}

#[test]
fn exact_pills_and_circles_do_not_use_a_cubic_approximation() {
    for shape in [Shape::Circular, Shape::Continuous] {
        let circle = Outline::new([0., 0., 40., 40.], [200.; 4], shape).unwrap();

        for p in [[0., 0.], [20., 20.], [2., 15.], [21., 43.]] {
            assert!(
                (circle.signed_distance(p) - ((p[0] - 20.).hypot(p[1] - 20.) - 20.)).abs() < 1e-12
            );
        }

        let pill = Outline::new([0., 0., 120., 20.], [10.; 4], shape).unwrap();
        assert_eq!(pill.signed_distance([60., 10.]), -10.);
        assert_eq!(pill.signed_distance([110., 0.]), 0.);
    }
}

#[test]
fn reference_and_inset_scale_together() {
    let outline = Outline::new([1.25, 2.5, 80., 48.], [12., 3., 6., 0.], Shape::Continuous)
        .unwrap()
        .inset(5.)
        .unwrap();

    for scale in [1.25, 1.5, 2.] {
        let transformed = outline.transformed([7., 9.], scale).unwrap();

        for p in [[0., 0.], [8., 9.], [45., 28.], [78., 45.]] {
            let q = [p[0] * scale + 7., p[1] * scale + 9.];
            assert!(
                (transformed.signed_distance(q) - scale * outline.signed_distance(p)).abs() < 1e-9
            );
        }
    }
}

#[test]
fn border_coverage_is_the_outer_minus_inner_contour() {
    let outline = Outline::new([0., 0., 80., 50.], [12.; 4], Shape::Continuous).unwrap();

    for width in [0., 0.25, 1., 12., 30.] {
        for y in 0..50 {
            for x in 0..80 {
                let p = [x as f64 + 0.5, y as f64 + 0.5];
                let outer = edge_coverage(outline.signed_distance(p));
                let inner = edge_coverage(outline.inset(width).unwrap().signed_distance(p));
                assert!(inner <= outer);
                assert!((inner + (outer - inner) - outer).abs() < 1e-12);

                if width == 30. {
                    assert_eq!(inner, 0.);
                }
            }
        }
    }
}

#[test]
fn invalid_geometry_never_enters_a_renderer() {
    for bounds in [
        [0., 0., 0., 10.],
        [0., 0., 10., -1.],
        [f64::NAN, 0., 10., 10.],
    ] {
        assert!(Outline::new(bounds, [2.; 4], Shape::Continuous).is_none());
    }

    assert!(Outline::new([0., 0., 10., 10.], [f64::INFINITY; 4], Shape::Circular).is_none());
    let outline = Outline::new([0., 0., 10., 10.], [-1.; 4], Shape::Continuous).unwrap();
    assert_eq!(outline.radii(), [0.; 4]);
    assert!(outline.inset(f64::NAN).is_none());
    assert!(outline.transformed([0., 0.], 0.).is_none());
}

fn endpoint_derivatives(points: [[f64; 2]; 4], end: bool) -> ([f64; 2], [f64; 2]) {
    if end {
        (
            std::array::from_fn(|i| 3. * (points[3][i] - points[2][i])),
            std::array::from_fn(|i| 6. * (points[3][i] - 2. * points[2][i] + points[1][i])),
        )
    } else {
        (
            std::array::from_fn(|i| 3. * (points[1][i] - points[0][i])),
            std::array::from_fn(|i| 6. * (points[2][i] - 2. * points[1][i] + points[0][i])),
        )
    }
}

fn curvature(tangent: [f64; 2], acceleration: [f64; 2]) -> f64 {
    (tangent[0] * acceleration[1] - tangent[1] * acceleration[0]).abs()
        / tangent[0].hypot(tangent[1]).powi(3)
}

#[test]
fn measured_profile_preserves_its_documented_join_limits() {
    let actual = ferese_shape::controls(1.);

    for (segment, fixture) in actual.iter().zip(PROFILE) {
        for (point, expected) in segment.iter().zip(fixture) {
            for axis in 0..2 {
                assert!((point[axis] - expected[axis]).abs() < 1e-15);
            }
        }
    }

    let (start, a0) = endpoint_derivatives(actual[0], false);
    let (end, a1) = endpoint_derivatives(actual[2], true);
    assert_eq!(start[0], 0.);
    assert_eq!(end[1], 0.);
    assert_eq!(curvature(start, a0), 0.);
    assert_eq!(curvature(end, a1), 0.);

    for pair in actual.windows(2) {
        assert_eq!(pair[0][3], pair[1][0]);
        let (before, a0) = endpoint_derivatives(pair[0], true);
        let (after, a1) = endpoint_derivatives(pair[1], false);
        let cosine = (before[0] * after[0] + before[1] * after[1])
            / before[0].hypot(before[1])
            / after[0].hypot(after[1]);
        let angle = cosine.clamp(-1., 1.).acos().to_degrees();
        assert!((angle - 2.453_166_93).abs() < 1e-6);
        assert!((curvature(before, a0) - curvature(after, a1)).abs() > 0.35);
    }
}
