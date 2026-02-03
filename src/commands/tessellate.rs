use crate::commands::Command;
use crate::models::Bounds;
use crate::models::{LocalKnots, TMesh};
use cgmath::Point3;
use cgmath::Vector4;
use rayon::prelude::*;

pub struct Tessellate {
    pub resolution: usize,
}

impl Command for Tessellate {
    type Result = Vec<Point3<f64>>;

    fn execute(&mut self, mesh: &TMesh) -> Self::Result {
        let Bounds {
            s: (s_min, s_max),
            t: (t_min, t_max),
        } = mesh.bounds();
        let knot_cache = mesh.knot_vectors();

        let denom = (self.resolution - 1) as f64;
        (0..self.resolution * self.resolution)
            .into_par_iter()
            .map(|i| {
                let u_i = i % self.resolution;
                let v_i = i / self.resolution;
                let s = if self.resolution > 1 {
                    s_min + (u_i as f64 / denom) * (s_max - s_min)
                } else {
                    s_min
                };

                let t = if self.resolution > 1 {
                    t_min + (v_i as f64 / denom) * (t_max - t_min)
                } else {
                    t_min
                };

                subs(mesh, s, t, &knot_cache)
            })
            .filter_map(|p| p)
            .collect()
    }
}

/// Evaluates a univariate cubic B-spline basis function.
///
/// # Arguments
/// * `u` - The parameter value to evaluate.
/// * `knots` - A local knot vector of length 5: [u_i, u_{i+1}, u_{i+2}, u_{i+3}, u_{i+4}].
pub fn cubic_basis_function(u: f64, knots: &[f64; 5]) -> f64 {
    // 1. Boundary check for the support [u_i, u_{i+4}]
    // Basis functions are non-zero only within their knot spans.
    if u < knots[0] || u > knots[4] {
        return 0.0;
    }

    // Clamp u to be strictly inside the support for the half-open interval logic,
    // effectively taking the limit from the left at the boundary.
    let u_eval = if u >= knots[4] { knots[4] - 1e-14 } else { u };

    // 2. Initialize the 0th degree basis (step functions)
    // There are 4 intervals defined by 5 knots.
    let mut n = [0.0; 4];
    for i in 0..4 {
        // Standard half-open interval check [t_i, t_{i+1})
        if u_eval >= knots[i] && u_eval < knots[i + 1] {
            n[i] = 1.0;
        }
    }

    // 3. Iteratively calculate higher degrees up to degree 3
    for p in 1..=3 {
        // In each degree layer, we calculate (4 - p) basis functions
        for i in 0..(4 - p) {
            let mut val = 0.0;

            // Left term: ((u - u_i) / (u_{i+p} - u_i)) * N_{i, p-1}(u)
            let den1 = knots[i + p] - knots[i];
            if den1.abs() > 1e-12 {
                val += ((u - knots[i]) / den1) * n[i];
            }

            // Right term: ((u_{i+p+1} - u) / (u_{i+p+1} - u_{i+1})) * N_{i+1, p-1}(u)
            let den2 = knots[i + p + 1] - knots[i + 1];
            if den2.abs() > 1e-12 {
                val += ((knots[i + p + 1] - u) / den2) * n[i + 1];
            }

            n[i] = val;
        }
    }

    // The result N_{i,3} is now at n
    n[0]
}

fn subs(mesh: &TMesh, s: f64, t: f64, knot_cache: &[LocalKnots]) -> Option<Point3<f64>> {
    let mut numerator = Vector4::new(0.0, 0.0, 0.0, 0.0);
    let mut denominator: f64 = 0.0;

    for (i, vert) in mesh.vertices.iter().enumerate() {
        let (s_knots, t_knots) = &knot_cache[i];

        // Quick AABB check in parameter space
        if s < s_knots[0] || s > s_knots[4] || t < t_knots[0] || t > t_knots[4] {
            continue;
        }

        let basis_s = cubic_basis_function(s, s_knots);
        let basis_t = cubic_basis_function(t, t_knots);
        let weight = vert.geometry.w;
        let basis = basis_s * basis_t * weight;

        if basis.abs() > 1e-12 {
            numerator += vert.geometry * basis; // geometry is pre-multiplied by weight usually?
            // Note: In rational splines, P_i usually stored as (wx, wy, wz, w).
            // If geometry is just (x,y,z,w), we multiply by basis.
            // Standard formula: sum(P_i * w_i * B_i) / sum(w_i * B_i)
            // If Vector4 is (x,y,z,w), then just add basis * Vector4.
        }
        denominator += basis_s * basis_t * weight; // Correct denominator accumulation
    }

    if denominator.abs() < 1e-9 {
        // Handle undefined regions or holes
        return None;
    }

    let result_homo = numerator / denominator;
    Some(Point3::new(result_homo.x, result_homo.y, result_homo.z))
}

#[cfg(test)]
mod tests {
    use crate::TSpline;
    use super::*;

    #[test]
    pub fn it_can_evaluate_points_on_square() {
        let square = TSpline::new_unit_square();
        let knots = square.mesh().knot_vectors();

        assert_eq!(Some(Point3::new(0., 0., 0.)), subs(square.mesh(), 0.0, 0.0, &knots));
        assert_eq!(Some(Point3::new(1., 0., 0.)),  subs(square.mesh(), 1.0, 0.0, &knots));
        assert_eq!(Some(Point3::new(0., 1., 0.)),  subs(square.mesh(), 0.0, 1.0, &knots));
        assert_eq!(Some(Point3::new(1., 1., 0.)),  subs(square.mesh(), 1.0, 1.0, &knots));
    }

    #[test]
    pub fn it_can_tessellate_a_square() {
        let square = TSpline::new_unit_square();
        let points = square.apply(&mut Tessellate{ resolution: 2 });

        assert_eq!(4, points.len());

        assert_eq!(Point3::new(0., 0., 0.), points[0]);
        assert_eq!(Point3::new(1., 0., 0.), points[1]);
        assert_eq!(Point3::new(0., 1., 0.), points[2]);
        assert_eq!(Point3::new(1., 1., 0.), points[3]);
    }

    #[test]
    pub fn it_can_evaluate_center() {
        let square = TSpline::new_unit_square();
        let knots = square.mesh().knot_vectors();
        let center =  subs(square.mesh(), 0.5, 0.5, &knots).unwrap();

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
        let square = TSpline::new_unit_square();
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
        let t_mesh = TSpline::new_t_junction();
        let knots = t_mesh.mesh().knot_vectors();

        // Just verify it doesn't panic and returns a point
        let p =  subs(t_mesh.mesh(), 0.0, 0.0, &knots).unwrap();
        assert!((p.z - 0.0).abs() < 1e-9);
    }
}
