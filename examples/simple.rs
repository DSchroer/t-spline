use std::error::Error;
use t_spline::export::write_ply;
use t_spline::models::{TSpline};

fn main() -> Result<(), Box<dyn Error>> {
    let mesh = TSpline::new_simple();

    let points = mesh.tessellate(100);
    write_ply(&mut std::io::stdout(), &points)?;
    Ok(())
}
