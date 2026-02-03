use std::error::Error;
use t_spline::TSpline;
use t_spline::commands::{Command, Tessellate};
use t_spline::export::PlyWriter;

fn main() -> Result<(), Box<dyn Error>> {
    let spline = TSpline::new_simple();
    let points = Tessellate { resolution: 100 }.apply(&spline);

    PlyWriter::default()
        .with_point(&points)?
        // .with_geometry(&spline.mesh())?
        .write(&mut std::io::stdout())?;

    Ok(())
}
