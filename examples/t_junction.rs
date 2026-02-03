use std::error::Error;
use t_spline::TSpline;
use t_spline::commands::{Command, Tessellate};
use t_spline::export::{ObjWriter};
use t_spline::tmesh::TMesh;

fn main() -> Result<(), Box<dyn Error>> {
    let mut spline = TSpline::new_t_junction();

    spline.apply_mut(&mut |m: &mut TMesh| {
        let j = m.vertices.iter_mut().find(|v| v.is_t_junction).unwrap();

        j.geometry.z = 0.5;
    });

    let points = Tessellate { resolution: 100 }.apply(&spline);

    ObjWriter::default()
        .with_control_surface("Control", &spline.mesh())?
        .with_points("Surface", &points)?
        .write(&mut std::io::stdout())?;
    Ok(())
}
