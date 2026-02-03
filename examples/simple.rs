use std::error::Error;
use t_spline::TSpline;
use t_spline::commands::{Command, Tessellate};
use t_spline::export::{ObjWriter};

fn main() -> Result<(), Box<dyn Error>> {
    let spline = TSpline::new_simple();
    let points = Tessellate { resolution: 100 }.apply(&spline);

    ObjWriter::default()
        .with_points("Surface", &points)?
        .with_control_surface("Control", &spline.mesh())?
        .write(&mut std::io::stdout())?;

    Ok(())
}
