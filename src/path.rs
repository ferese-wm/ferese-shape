use crate::{Outline, Point};

impl Outline {
    /// Samples the zero-distance contour as a closed polygon.
    /// Tolerance is in the outline's units; divide a physical-pixel tolerance by scale.
    /// Insets use the reference distance field, never offset cubic control points.
    /// Returns an empty polygon for a collapsed contour, or None for invalid accuracy
    /// or a contour that exceeds the subdivision budget.
    pub fn polygon(self, tolerance: f64) -> Option<Vec<Point>> {
        if !tolerance.is_finite() || tolerance <= 0.0 {
            return None;
        }

        let [x, y, width, height] = self.bounds();
        let center = [x + width * 0.5, y + height * 0.5];

        if self.signed_distance(center) >= 0.0 {
            return Some(Vec::new());
        }

        let reach = width.hypot(height) + self.inset_distance().abs() * 2.0 + tolerance;

        if !reach.is_finite() {
            return None;
        }

        let sample = |angle: f64| {
            let (sin, cos) = angle.sin_cos();
            let ray = [cos, sin];
            let mut inside = 0.0;
            let mut outside = reach;

            for _ in 0..48 {
                let distance = (inside + outside) * 0.5;
                let point = [center[0] + ray[0] * distance, center[1] + ray[1] * distance];

                if self.signed_distance(point) <= 0.0 {
                    inside = distance;
                } else {
                    outside = distance;
                }
            }

            let distance = (inside + outside) * 0.5;
            [center[0] + ray[0] * distance, center[1] + ray[1] * distance]
        };
        let mut points = vec![sample(0.0)];

        for segment in 0..16 {
            let start = segment as f64 * std::f64::consts::TAU / 16.0;
            let end = (segment + 1) as f64 * std::f64::consts::TAU / 16.0;
            let a = *points.last()?;
            let b = sample(end);
            subdivide(self, &sample, start, end, a, b, tolerance, 0, &mut points)?;
        }

        points.pop();
        Some(points)
    }
}

#[allow(clippy::too_many_arguments)]
fn subdivide(
    outline: Outline,
    sample: &impl Fn(f64) -> Point,
    start: f64,
    end: f64,
    a: Point,
    b: Point,
    tolerance: f64,
    depth: usize,
    points: &mut Vec<Point>,
) -> Option<()> {
    // Quarter samples catch the small tangent changes at the measured cubic joins.
    let deviation = [0.25, 0.5, 0.75].map(|t| {
        let point = [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t];
        outline.signed_distance(point).abs()
    });

    if points.len() >= 65_536 {
        return None;
    }

    if deviation.into_iter().all(|error| error <= tolerance * 0.5) {
        points.push(b);
        return Some(());
    }

    if depth >= 24 {
        return None;
    }

    let middle = (start + end) * 0.5;
    let point = sample(middle);
    subdivide(
        outline,
        sample,
        start,
        middle,
        a,
        point,
        tolerance,
        depth + 1,
        points,
    )?;
    subdivide(
        outline,
        sample,
        middle,
        end,
        point,
        b,
        tolerance,
        depth + 1,
        points,
    )
}
