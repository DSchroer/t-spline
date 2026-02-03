use std::error::Error;
use t_spline::commands::{Command, Tessellate};
use t_spline::export::PlyWriter;
use t_spline::models::{TMesh, TSpline};

fn main() -> Result<(), Box<dyn Error>> {
    let mut spline = TSpline::new_t_junction();

    spline.apply_mut(&mut |m: &mut TMesh| {
        let j = m.vertices.iter_mut().find(|v| v.is_t_junction).unwrap();

        j.geometry.z = 0.5;
    });

    let points = Tessellate{ resolution: 100}.apply(&spline);

    PlyWriter::default()
        .with_point(&points)?
        // .with_geometry(&spline.mesh())?
        .write(&mut std::io::stdout())?;
    Ok(())
}
