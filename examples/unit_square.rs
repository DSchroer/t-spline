use std::error::Error;
use t_spline::export::PlyWriter;
use t_spline::models::{TMesh, TSpline};

fn main() -> Result<(), Box<dyn Error>> {
    let mut spline = TSpline::new_unit_square();

    spline.perform(&mut |m: &mut TMesh| {
        m.vertices[0].geometry.z = 1.0;
        m.vertices[2].geometry.z = 0.5;
    })?;

    let points = spline.tessellate(100);

    PlyWriter::default()
        .with_point(&points)?
        // .with_geometry(spline.mesh())?
        .write(&mut std::io::stdout())?;

    Ok(())
}
