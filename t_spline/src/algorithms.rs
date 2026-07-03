/*
 * Copyright (C) 2026 Dominick Schroer
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use crate::Numeric;
use crate::uv_mesh::LocalKnots;
use nalgebra::{Point3, Vector4};

/// Evaluates a univariate cubic B-spline basis function.
///
/// # Arguments
/// * `u` - The parameter value to evaluate.
/// * `knots` - A local knot vector of length 5: [u_i, u_{i+1}, u_{i+2}, u_{i+3}, u_{i+4}].
pub fn cubic_basis_function<T: Numeric>(u: T, knots: &[isize; 5]) -> T {
    let knots = |i| T::from_isize(knots[i]).unwrap();

    // 2. Initialize the 0th degree basis (step functions)
    // There are 4 intervals defined by 5 knots.
    let mut n = [T::zero(); 4];
    for i in 0..4 {
        // Standard half-open interval check [t_i, t_{i+1})
        if u >= knots(i) && u < knots(i + 1) {
            n[i] = T::one();
        }
    }

    // Handle right endpoint: u == last knot is excluded by half-open intervals.
    // Activate the last nonzero-span interval, treating the boundary as closed.
    if u == knots(4) {
        for i in (0..4).rev() {
            if knots(i) < knots(i + 1) {
                n[i] = T::one();
                break;
            }
        }
    }

    // 3. Iteratively calculate higher degrees up to degree 3
    for p in 1..=3 {
        // In each degree layer, we calculate (4 - p) basis functions
        for i in 0..(4 - p) {
            let mut val = T::zero();

            // Left term: ((u - u_i) / (u_{i+p} - u_i)) * N_{i, p-1}(u)
            let den1 = knots(i + p) - knots(i);
            if den1 != T::zero() {
                val += ((u - knots(i)) / den1) * n[i];
            }

            // Right term: ((u_{i+p+1} - u) / (u_{i+p+1} - u_{i+1})) * N_{i+1, p-1}(u)
            let den2 = knots(i + p + 1) - knots(i + 1);
            if den2 != T::zero() {
                val += ((knots(i + p + 1) - u) / den2) * n[i + 1];
            }

            n[i] = val;
        }
    }

    // The result N_{i,3} is now at n
    n[0]
}

pub fn subs<T: Numeric + 'static>(
    vertices: &[Vector4<T>],
    (s, t): (T, T),
    knot_cache: &[LocalKnots],
) -> Option<Point3<T>> {
    let mut point_sum: Point3<T> = Point3::origin();
    let mut weight_sum = T::zero();

    for (i, vertex) in vertices.iter().enumerate() {
        // 1. Evaluate the 1D basis functions for s and t
        let n_s = cubic_basis_function(s, &knot_cache[i].s_knots);
        let n_t = cubic_basis_function(t, &knot_cache[i].t_knots);

        // 2. The 2D basis function B_i(s, t) is the product of the 1D functions
        let b_i = n_s * n_t;

        // Skip calculations if this control point doesn't influence (s, t)
        if b_i > T::zero() {
            // 2. Multiply the basis function by the point's weight w_i
            let rational_weight = b_i * vertex.w;

            // 3. Accumulate the weighted point sum (Numerator of Eq. 1)
            point_sum.x += vertex.x * rational_weight;
            point_sum.y += vertex.y * rational_weight;
            point_sum.z += vertex.z * rational_weight;

            // 4. Accumulate the total weight (Denominator of Eq. 1)
            weight_sum += rational_weight;
        }
    }

    // 5. Divide by the sum of weights to get the final rational point
    if weight_sum > T::zero() {
        Some(Point3::new(
            point_sum.x / weight_sum,
            point_sum.y / weight_sum,
            point_sum.z / weight_sum,
        ))
    } else {
        // (s, t) is outside the defined domain of the entire surface
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TSpline;
    use crate::uv_mesh::UVMesh;
    use alloc::vec;

    #[test]
    pub fn it_can_find_points_on_a_square() {
        let mesh = TSpline::new_unit_square();
        let knots = mesh.local_knots();
        let points = vec![
            Vector4::new(0.0, 0.0, 0.0, 1.0),
            Vector4::new(1.0, 0.0, 0.0, 1.0),
            Vector4::new(1.0, 1.0, 0.0, 1.0),
            Vector4::new(0.0, 1.0, 0.0, 1.0),
        ];

        assert_eq!(
            Point3::new(0., 0., 0.),
            subs(&points, (0., 0.), &knots).unwrap()
        );
        assert_eq!(
            Point3::new(1., 0., 0.),
            subs(&points, (1., 0.), &knots).unwrap()
        );
        assert_eq!(
            Point3::new(0., 1., 0.),
            subs(&points, (0., 1.), &knots).unwrap()
        );
        assert_eq!(
            Point3::new(1., 1., 0.),
            subs(&points, (1., 1.), &knots).unwrap()
        );
    }

    #[test]
    fn test_cubic_basis_function_uniform_knots() {
        let knots = [0, 1, 2, 3, 4];

        assert_eq!(0.0, cubic_basis_function(0.0, &knots));
        assert_eq!(0.0, cubic_basis_function(4.0, &knots));

        assert!((cubic_basis_function(1.0_f64, &knots) - 1.0 / 6.0).abs() < 1e-6);
        assert!((cubic_basis_function(3.0_f64, &knots) - 1.0 / 6.0).abs() < 1e-6);
        assert!((cubic_basis_function(2.0_f64, &knots) - 2.0 / 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_cubic_basis_function_bezier_left_boundary() {
        let knots = [0, 0, 0, 0, 1];

        assert!((cubic_basis_function(0.0_f64, &knots) - 1.0).abs() < 1e-6);
        assert!((cubic_basis_function(0.5_f64, &knots) - 0.125).abs() < 1e-6);
        assert_eq!(0.0, cubic_basis_function(1.0, &knots));
    }
}
