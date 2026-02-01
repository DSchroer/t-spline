use std::error::Error;
use t_spline::export::write_ply;
use t_spline::models::{TMesh, TSpline};

fn main() -> Result<(), Box<dyn Error>> {
    let mut mesh = TSpline::new_unit_square();

    mesh.perform(&mut |m: &mut TMesh| {
        m.vertices[0].geometry.z = 1.0;
        m.vertices[2].geometry.z = 1.0;
    })?;

    let points = mesh.tessellate(100);
    write_ply(&mut std::io::stdout(), &points)?;
    Ok(())
}
