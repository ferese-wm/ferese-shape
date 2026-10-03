use ferese_shape::{Outline, Shape};

#[test]
fn polygon_insets_stay_within_a_quarter_physical_pixel() {
    for shape in [Shape::Circular, Shape::Continuous] {
        for size in [[100.0, 80.0], [20.0, 20.0], [760.0, 500.0]] {
            for radii in [
                [0.0; 4],
                [0.375; 4],
                [8.25; 4],
                [30.0; 4],
                [0.0, 20.0, 7.0, 15.0],
            ] {
                for scale in [1.0, 1.25, 1.5] {
                    for inset in [-3.0, 0.0, 0.75, 4.0, 12.0, 32.0] {
                        let outline = Outline::new([1.25, 2.5, size[0], size[1]], radii, shape)
                            .unwrap()
                            .inset(inset)
                            .unwrap()
                            .transformed([0.0; 2], scale)
                            .unwrap();
                        let points = outline.polygon(0.25).unwrap();

                        if points.is_empty() {
                            assert!(
                                outline.signed_distance([
                                    (1.25 + size[0] * 0.5) * scale,
                                    (2.5 + size[1] * 0.5) * scale
                                ]) >= 0.0
                            );
                            continue;
                        }

                        for (a, b) in points.iter().zip(points.iter().cycle().skip(1)) {
                            for step in 0..=100 {
                                let t = step as f64 / 100.0;
                                let point = [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t];
                                let distance = outline.signed_distance(point).abs();
                                assert!(
                                    distance <= 0.25,
                                    "shape={shape:?}, size={size:?}, radii={radii:?}, scale={scale}, inset={inset}, deviation={distance}"
                                );
                            }
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn invalid_accuracy_is_rejected_and_collapsed_contours_are_empty() {
    let outline = Outline::new([0.0, 0.0, 100.0, 80.0], [20.0; 4], Shape::Continuous).unwrap();
    assert!(outline.polygon(0.0).is_none());
    assert!(outline.polygon(f64::NAN).is_none());
    assert!(
        outline
            .inset(41.0)
            .unwrap()
            .polygon(0.25)
            .unwrap()
            .is_empty()
    );
}
