use std::error::Error;
use t_spline::TSpline;
use t_spline::commands::{Command, Tessellate};
use t_spline::export::{ObjWriter};
use t_spline::tmesh::TMesh;

fn main() -> Result<(), Box<dyn Error>> {
    let mut spline = TSpline::new_unit_square();

    spline.apply_mut(&mut |m: &mut TMesh| {
        m.vertices[0].geometry.z = 1.0;
        m.vertices[2].geometry.z = 0.5;
    });

    let points = Tessellate { resolution: 100 }.apply(&mut spline);

    ObjWriter::default()
        .with_points("Surface", &points)?
        .with_control_surface("Control", &spline.mesh())?
        .write(&mut std::io::stdout())?;

    Ok(())
}
