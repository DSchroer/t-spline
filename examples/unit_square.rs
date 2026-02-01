use t_spline::export::write_ply;
use t_spline::models::{TMesh, TSpline};

fn main() {
    let mut mesh = TMesh::new_unit_square();
    mesh.vertices[0].geometry.z = 1.0;
    mesh.vertices[2].geometry.z = 1.0;

    let points = TSpline::new(mesh).tessellate(100);
    write_ply(&mut std::io::stdout(), &points).unwrap();
}
