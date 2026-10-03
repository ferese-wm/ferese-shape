use ferese_shape::{Outline, Shape};

#[test]
#[ignore = "manual release CPU timing fixture"]
fn corner_distance_timing() {
    for (label, radii) in [("uniform", [24.0; 4]), ("mixed", [0.0, 8.0, 24.0, 16.0])] {
        let outline = Outline::new([0.0, 0.0, 100.0, 80.0], radii, Shape::Continuous).unwrap();
        let start = std::time::Instant::now();
        let mut sum = 0.0;
        for _ in 0..30 {
            for y in -16..96 {
                for x in -16..116 {
                    sum += outline
                        .signed_distance(std::hint::black_box([x as f64 + 0.25, y as f64 + 0.5]));
                }
            }
        }
        std::hint::black_box(sum);
        eprintln!(
            "{label}: {:.3} ms/grid, checksum={sum:.8}",
            start.elapsed().as_secs_f64() * 1000.0 / 30.0
        );
    }
}
