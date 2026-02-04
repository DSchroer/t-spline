use fixed::types::I10F22;
use std::error::Error;
use num_traits::One;
use t_spline::{Command, TSpline};
use t_spline::tmesh::TMesh;
use t_spline_commands::tessellate::Tessellate;
use t_spline_io::obj_writer::ObjWriter;


fn main() -> Result<(), Box<dyn Error>> {
    let mut spline: TSpline<I10F22> = TSpline::new_unit_square();
    spline.apply_mut(&mut |m: &mut TMesh<I10F22>| m.vertices[0].geometry.z = I10F22::one());

    let points = Tessellate { resolution: 10 }.apply(&spline);

    ObjWriter::default()
        .with_points("Surface", &points)?
        .with_control_surface("Control", &spline.mesh())?
        .write(&mut std::io::stdout())?;

    Ok(())
}
