// All distances and coordinates are in the caller's units.
struct ShapeSample {
    point: vec2<f32>,
    tangent: vec2<f32>,
    acceleration: vec2<f32>,
}

fn shape_controls(segment: u32, blend: f32) -> mat4x2<f32> {
    let a = SHAPE_CIRCLE[segment];
    let b = SHAPE_CONTINUOUS[segment];
    return mat4x2<f32>(mix(a[0], b[0], blend), mix(a[1], b[1], blend),
                      mix(a[2], b[2], blend), mix(a[3], b[3], blend));
}

fn shape_sample(c: mat4x2<f32>, t: f32) -> ShapeSample {
    let first = 3.0 * (c[1] - c[0]);
    let second = 3.0 * (c[2] - 2.0 * c[1] + c[0]);
    let third = c[3] - 3.0 * c[2] + 3.0 * c[1] - c[0];
    return ShapeSample(c[0] + t * (first + t * (second + t * third)),
                       first + t * (2.0 * second + 3.0 * t * third),
                       2.0 * second + 6.0 * t * third);
}

fn shape_nearest(c: mat4x2<f32>, p: vec2<f32>) -> f32 {
    var distance = 1.0e30;

    for (var seed = 0u; seed < 2u; seed++) {
        var t = f32(seed);

        for (var iteration = 0u; iteration < 8u; iteration++) {
            let sample = shape_sample(c, t);
            let error = sample.point - p;
            let speed = dot(sample.tangent, sample.tangent);
            let denominator = max(speed + dot(error, sample.acceleration), speed * 0.25);
            let next = clamp(t - clamp(dot(error, sample.tangent) / denominator, -0.25, 0.25), 0.0, 1.0);
            if next == t {
                break;
            }
            t = next;
        }

        distance = min(distance, length(shape_sample(c, t).point - p));
    }

    return distance;
}

fn shape_contains(p: vec2<f32>, blend: f32) -> bool {
    for (var segment = 0u; segment < 3u; segment++) {
        let c = shape_controls(segment, blend);

        if p.x > c[3].x {
            continue;
        }

        // The blended profiles are monotone; endpoint bounds settle most
        // containment queries without the inverse-curve search.
        if p.y >= c[0].y {
            return true;
        }
        if p.y < c[3].y {
            return false;
        }

        var low = 0.0;
        var high = 1.0;

        for (var iteration = 0u; iteration < 24u; iteration++) {
            let t = (low + high) * 0.5;

            if shape_sample(c, t).point.x < p.x {
                low = t;
            } else {
                high = t;
            }
        }

        return p.y >= shape_sample(c, (low + high) * 0.5).point.y;
    }

    return true;
}

fn shape_line_distance(p: vec2<f32>, a: vec2<f32>, b: vec2<f32>) -> f32 {
    let edge = b - a;
    let squared = dot(edge, edge);

    if squared == 0.0 {
        return length(p - a);
    }

    let t = clamp(dot(p - a, edge) / squared, 0.0, 1.0);
    return length(p - a - t * edge);
}

// Radii: top-left, top-right, bottom-right, bottom-left. Kind: 0 circular, 1 squircle.
fn shape_distance(point: vec2<f32>, bounds: vec4<f32>, radii: vec4<f32>, kind: u32, inset: f32) -> f32 {
    let p = point - bounds.xy;
    let size = bounds.zw;
    let limit = min(size.x, size.y) * 0.5;
    let r = clamp(radii, vec4(0.0), vec4(limit));

    if all(r == vec4(r.x)) && (kind == 0u || r.x == 0.0 || r.x >= limit) {
        let q = abs(p - size * 0.5) - size * 0.5 + r.x;
        return length(max(q, vec2(0.0))) + min(max(q.x, q.y), 0.0) - r.x + inset;
    }

    var blends = vec4(0.0);
    var extents = r;

    if kind == 1u {
        blends = clamp((vec4(limit) / max(r, vec4(1.0e-30)) - 1.0) / (SHAPE_EXTENT - 1.0), vec4(0.0), vec4(1.0));
        extents = r * (1.0 + (SHAPE_EXTENT - 1.0) * blends);
    }

    let edge_inset = size * 0.5 - abs(p - size * 0.5);

    if max(edge_inset.x, edge_inset.y) >= max(max(extents.x, extents.y), max(extents.z, extents.w)) {
        return -min(edge_inset.x, edge_inset.y) + inset;
    }

    var distance = shape_line_distance(p, vec2(extents.x, 0.0), vec2(size.x - extents.y, 0.0));
    distance = min(distance, shape_line_distance(p, vec2(size.x, extents.y), vec2(size.x, size.y - extents.z)));
    distance = min(distance, shape_line_distance(p, vec2(size.x - extents.z, size.y), vec2(extents.w, size.y)));
    distance = min(distance, shape_line_distance(p, vec2(0.0, size.y - extents.w), vec2(0.0, extents.x)));
    let queries = array<vec2<f32>, 4>(p, vec2(size.x - p.x, p.y), size - p, vec2(p.x, size.y - p.y));
    var inside = all(p >= vec2(0.0)) && all(p <= size);

    for (var corner = 0u; corner < 4u; corner++) {
        let radius = r[corner];

        if radius == 0.0 {
            continue;
        }

        let q = queries[corner];
        let extent = extents[corner];
        let circular = kind == 0u || radius >= limit;

        if inside && all(q < vec2(extent)) {
            if circular {
                inside = length(q - radius) <= radius;
            } else {
                inside = shape_contains(q / radius, blends[corner]);
            }
        }

        let lower = length(max(max(-q, q - extent), vec2(0.0)));

        if lower >= distance {
            continue;
        }

        var candidate = 1.0e30;

        if circular {
            if all(q <= vec2(radius)) {
                candidate = abs(length(q - radius) - radius);
            } else {
                candidate = min(length(q - vec2(0.0, radius)), length(q - vec2(radius, 0.0)));
            }
        } else {
            // Fold the symmetric corner and omit the opposite shoulder.
            let normalized = vec2(min(q.x, q.y), max(q.x, q.y)) / radius;
            var nearest = distance / radius;
            for (var segment = 0u; segment < 2u; segment++) {
                let c = shape_controls(segment, blends[corner]);
                let low = min(min(c[0], c[1]), min(c[2], c[3]));
                let high = max(max(c[0], c[1]), max(c[2], c[3]));
                let delta = max(max(low - normalized, normalized - high), vec2(0.0));
                if dot(delta, delta) < nearest * nearest {
                    nearest = min(nearest, shape_nearest(c, normalized));
                }
            }
            candidate = nearest * radius;
        }

        distance = min(distance, candidate);
    }

    return select(distance, -distance, inside) + inset;
}

fn shape_coverage(distance: f32) -> f32 {
    return clamp(0.5 - distance, 0.0, 1.0);
}
