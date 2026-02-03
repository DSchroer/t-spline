use num_traits::{Bounded, FromPrimitive, Num, NumAssign, Signed};
use rayon::prelude::*;
use std::fmt::Debug;
use t_spline::tmesh::bounds::Bounds;
use t_spline::tmesh::ids::VertID;
use t_spline::tmesh::{LocalKnots, TMesh};
use t_spline::{Command, Point3};

pub struct Tessellate {
    pub resolution: usize,
}

impl<
    T: Num + Copy + PartialOrd + Signed + NumAssign + Debug + Send + Sync + Bounded + FromPrimitive,
> Command<T> for Tessellate
{
    type Result = Vec<Point3<T>>;

    fn execute(&mut self, mesh: &TMesh<T>) -> Self::Result {
        let mut bounds = Bounds::default();
        bounds.add_mesh(mesh);

        let knot_cache: Vec<_> = knot_vectors(mesh);

        (0..self.resolution * self.resolution)
            .into_par_iter()
            .map(|i| mesh.subs(bounds.interpolate(i, self.resolution), &knot_cache))
            .filter_map(|p| p)
            .collect()
    }
}

fn knot_vectors<T: Num + FromPrimitive + Copy + Bounded + Signed + PartialOrd + Send + Sync>(
    mesh: &TMesh<T>,
) -> Vec<LocalKnots<T>> {
    (0..mesh.vertices.len())
        .into_par_iter()
        .map(|v| mesh.infer_local_knots(VertID(v)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use t_spline::TSpline;
    use t_spline::tmesh::ids::FaceID;

    #[test]
    pub fn it_can_evaluate_points_on_square() {
        let square = TSpline::new_unit_square();
        let knots = knot_vectors(square.mesh());

        assert_eq!(
            Some(Point3::new(0., 0., 0.)),
            square.mesh().subs((0.0, 0.0), &knots)
        );
        assert_eq!(
            Some(Point3::new(1., 0., 0.)),
            square.mesh().subs((1.0, 0.0), &knots)
        );
        assert_eq!(
            Some(Point3::new(0., 1., 0.)),
            square.mesh().subs((0.0, 1.0), &knots)
        );
        assert_eq!(
            Some(Point3::new(1., 1., 0.)),
            square.mesh().subs((1.0, 1.0), &knots)
        );
    }

    #[test]
    pub fn it_can_tessellate_a_square() {
        let square = TSpline::new_unit_square();
        let points = square.apply(&mut Tessellate { resolution: 2 });

        assert_eq!(4, points.len());

        assert_eq!(Point3::new(0., 0., 0.), points[0]);
        assert_eq!(Point3::new(1., 0., 0.), points[1]);
        assert_eq!(Point3::new(0., 1., 0.), points[2]);
        assert_eq!(Point3::new(1., 1., 0.), points[3]);
    }

    #[test]
    pub fn it_can_evaluate_center() {
        let square = TSpline::new_unit_square();
        let knots = knot_vectors(square.mesh());
        let center = square.mesh().subs((0.5, 0.5), &knots).unwrap();

        // Check components with epsilon tolerance
        let expected = Point3::new(0.5, 0.5, 0.0);
        let diff = center - expected;
        assert!(
            diff.x.abs() < 1e-9 && diff.y.abs() < 1e-9 && diff.z.abs() < 1e-9,
            "Center mismatch: expected {:?}, got {:?}",
            expected,
            center
        );
    }

    #[test]
    pub fn it_can_tessellate_square() {
        let square = TSpline::<f64>::new_unit_square();
        let resolution = 10;
        let points = square.apply(&mut Tessellate { resolution });

        assert_eq!(points.len(), resolution * resolution);

        // Verify bounds of tessellated points
        for p in points {
            assert!(p.x >= -1e-9 && p.x <= 1.0 + 1e-9);
            assert!(p.y >= -1e-9 && p.y <= 1.0 + 1e-9);
            assert!((p.z - 0.0).abs() < 1e-9);
        }
    }

    #[test]
    pub fn it_can_create_and_evaluate_t_junction_mesh() {
        let mut t_mesh = TSpline::new_t_junction();

        // lift center t-junction,
        // spline should still be symmetrical
        t_mesh.apply_mut(&mut |m: &mut TMesh<f64>| {
            let j = m.vertices.iter_mut().find(|v| v.is_t_junction).unwrap();
            j.geometry.z = 0.5;
        });

        let knots = knot_vectors(t_mesh.mesh());

        let mut f1_bounds = Bounds::default();
        f1_bounds.add_face(&t_mesh.mesh(), FaceID(1));
        assert_eq!(1., f1_bounds.area());
        let f1_center = t_mesh.mesh().subs(f1_bounds.center(), &knots).unwrap();

        let mut f2_bounds = Bounds::default();
        f2_bounds.add_face(&t_mesh.mesh(), FaceID(2));
        assert_eq!(1., f2_bounds.area());
        let f2_center = t_mesh.mesh().subs(f2_bounds.center(), &knots).unwrap();

        assert_eq!(f1_center.z, f2_center.z, "t-spline is not symmetrical");
    }
}
