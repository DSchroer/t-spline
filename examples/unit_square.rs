use t_spline::export::write_ply;
use t_spline::models::{TMesh, TSpline};

fn main() {
    let mut mesh = TSpline::new_unit_square();

    mesh.perform(&mut |m: &mut TMesh| {
        m.vertices[0].geometry.z = 1.0;
        m.vertices[2].geometry.z = 1.0;
    }).unwrap();

    let points = mesh.tessellate(100);
    write_ply(&mut std::io::stdout(), &points).unwrap();
}
