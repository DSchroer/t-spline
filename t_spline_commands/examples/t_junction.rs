use std::error::Error;
use t_spline::{Command, TSpline};
use t_spline_io::obj_writer::ObjWriter;
use t_spline::tmesh::TMesh;
use t_spline_commands::tessellate::Tessellate;

fn main() -> Result<(), Box<dyn Error>> {
    let mut spline = TSpline::new_t_junction();

    spline.apply_mut(&mut |m: &mut TMesh<f64>| {
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
