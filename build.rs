#[path = "src/profile.rs"]
mod profile;

use std::fmt::Write;

fn main() {
    println!("cargo:rerun-if-changed=src/profile.rs");
    println!("cargo:rerun-if-changed=src/shape.wgsl");
    let mut shader = format!("const SHAPE_EXTENT: f32 = {:.11};\n", profile::EXTENT);

    for (name, curves) in [
        ("SHAPE_CONTINUOUS", profile::CONTINUOUS),
        ("SHAPE_CIRCLE", profile::CIRCULAR_BLEND),
    ] {
        writeln!(
            &mut shader,
            "const {name}: array<mat4x2<f32>, 3> = array<mat4x2<f32>, 3>("
        )
        .unwrap();

        for points in curves {
            shader.push_str("    mat4x2<f32>(");

            for point in points {
                write!(
                    &mut shader,
                    "vec2<f32>({:.11}, {:.11}),",
                    point[0], point[1]
                )
                .unwrap();
            }

            shader.push_str("),\n");
        }

        shader.push_str(");\n");
    }

    shader.push_str(include_str!("src/shape.wgsl"));
    let path = std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap()).join("shape.wgsl");
    std::fs::write(path, shader).unwrap();
}
