use fixed::types::I10F22;
use std::error::Error;
use t_spline::{Command, TSpline};
use t_spline_commands::tessellate::Tessellate;
use t_spline_io::obj_writer::ObjWriter;

fn main() -> Result<(), Box<dyn Error>> {
    let spline: TSpline<I10F22> = TSpline::new_unit_square();
    let points = Tessellate { resolution: 100 }.apply(&spline);

    ObjWriter::default()
        .with_points("Surface", &points)?
        .with_control_surface("Control", &spline.mesh())?
        .write(&mut std::io::stdout())?;

    Ok(())
}
