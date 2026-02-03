use std::error::Error;
use t_spline::export::PlyWriter;
use t_spline::models::TSpline;
use t_spline::commands::Tessellate;

fn main() -> Result<(), Box<dyn Error>> {
    let spline = TSpline::new_simple();
    let points = spline.command(&mut Tessellate{ resolution: 100 });

    PlyWriter::default()
        .with_point(&points)?
        // .with_geometry(&spline.mesh())?
        .write(&mut std::io::stdout())?;

    Ok(())
}
