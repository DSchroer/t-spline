use t_spline::export::write_ply;
use t_spline::models::{TMesh, TSpline};

fn main() {
    let mesh = TMesh::new_t_junction();

    let points = TSpline::new(mesh).tessellate(100);
    write_ply(&mut std::io::stdout(), &points).unwrap();
}
