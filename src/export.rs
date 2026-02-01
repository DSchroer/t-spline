use std::io;
use std::io::Write;
use cgmath::Point3;

pub fn write_ply(w: &mut impl Write, points: &[Point3<f64>]) -> Result<(), io::Error> {
    writeln!(w, r"ply
format ascii 1.0
element vertex {}
property float x
property float y
property float z
end_header", points.len())?;

    for point in points {
        writeln!(w, "{} {} {}", point.x, point.y, point.z)?;
    }

    Ok(())
}