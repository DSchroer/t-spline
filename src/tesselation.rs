use cgmath::Point3;
use cgmath::Vector4;
use rayon::prelude::*;
use crate::models::{Bounds, TSpline};

/// Evaluates a univariate cubic B-spline basis function.
///
/// # Arguments
/// * `u` - The parameter value to evaluate.
/// * `knots` - A local knot vector of length 5: [u_i, u_{i+1}, u_{i+2}, u_{i+3}, u_{i+4}].
pub fn cubic_basis_function(u: f64, knots: &[f64; 5]) -> f64 {
    // 1. Boundary check for the support [u_i, u_{i+4}]
    // Basis functions are non-zero only within their knot spans.
    if u < knots[0] || u > knots[1] {
        return 0.0;
    }

    // 2. Initialize the 0th degree basis (step functions)
    // There are 4 intervals defined by 5 knots.
    let mut n = [0.0; 4];
    for i in 0..4 {
        // Handle the right-most boundary to ensure the interval is closed at the end
        if u >= knots[i] && (u < knots[i + 1] || (u == knots[1] && i == 3)) {
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

impl TSpline {
    pub fn subs(&self, s: f64, t: f64) -> Point3<f64> {
        let mut numerator = Vector4::new(0.0, 0.0, 0.0, 0.0);
        let mut denominator: f64 = 0.0;

        for (i, vert) in self.mesh().vertices.iter().enumerate() {
            let (s_knots, t_knots) = &self.knot_cache()[i];

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
            return Point3::new(0.0, 0.0, 0.0);
        }

        let result_homo = numerator / denominator;
        Point3::new(result_homo.x, result_homo.y, result_homo.z)
    }
}

impl TSpline {
    pub fn tessellate(&self, resolution: usize) -> Vec<Point3<f64>> {
        let Bounds{
            s: (s_min, s_max),
            t: (t_min, t_max),
        } = self.bounds();

        (0..resolution * resolution).into_par_iter().map(|i| {
            let u_i = i % resolution;
            let v_i = i / resolution;
            let s = s_min + (u_i as f64 / resolution as f64) * (s_max - s_min);
            let t = t_min + (v_i as f64 / resolution as f64) * (t_max - t_min);

            self.subs(s, t)
        }).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn it_can_evaluate_points_on_square() {
        let square = TSpline::new_unit_square();

        assert_eq!(Point3::new(0., 0., 0.), square.subs(0.0, 0.0));
        assert_eq!(Point3::new(1., 0., 0.), square.subs(1.0, 0.0));
    }

    // #[test]
    // pub fn it_can_tessellate_square() {
    //     let square = TSpline::new_unit_square();
    //
    //     let points = square.tessellate(10);
    //
    //     println!("{:?}", points);
    // }
}