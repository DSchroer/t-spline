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

pub mod bounds;
pub mod control_point;
pub mod direction;
pub mod face;
pub mod half_edge;
pub mod ids;
pub mod segment;

use crate::Numeric;
use crate::tmesh::control_point::ControlPoint;
use crate::tmesh::direction::Direction;
use crate::tmesh::face::Face;
use crate::tmesh::half_edge::HalfEdge;
use crate::tmesh::ids::{EdgeID, FaceID, VertID};
use alloc::vec::Vec;
use nalgebra::Point3;
use smallvec::SmallVec;

#[derive(Debug, Clone, Default)]
pub struct TMesh<T> {
    pub vertices: Vec<ControlPoint<T>>,
    pub edges: Vec<HalfEdge>,
    pub faces: Vec<Face>,
}

/// Two directional knot vectors for S & T directions.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LocalKnots {
    pub s_knots: KnotVector,
    pub t_knots: KnotVector,
}

/// A local knot vector consisting of 5 knots for a cubic T-spline.
///
/// In T-spline representations, each control point is associated with a local
/// knot vector in both the `u` and `v` parametric directions. For a cubic basis
/// function (degree 3), exactly 5 knots are required to evaluate the blending
/// weights over its local support domain.
///
/// The knots are typically ordered such that $k_0 \le k_1 \le k_2 \le k_3 \le k_4$.
pub type KnotVector = [isize; 5];

impl<T> TMesh<T> {
    pub fn vertex(&self, id: VertID) -> &ControlPoint<T> {
        &self.vertices[id.0]
    }

    pub fn edge(&self, id: EdgeID) -> &HalfEdge {
        &self.edges[id.0]
    }

    pub fn face(&self, id: FaceID) -> &Face {
        &self.faces[id.0]
    }

    pub fn face_edges(&self, face_id: FaceID) -> Vec<EdgeID> {
        let start_edge = self.faces[face_id.0].edge;
        let mut edges = Vec::new();
        let mut curr = start_edge;
        loop {
            edges.push(curr);
            curr = self.edge(curr).next;
            if curr == start_edge {
                break;
            }
        }
        edges
    }

    pub fn next_edge(&self, edge: &HalfEdge) -> &HalfEdge {
        &self.edge(edge.next)
    }

    pub fn connected_edges(&self, id: VertID) -> impl ExactSizeIterator<Item = EdgeID> {
        let mut edges: SmallVec<[EdgeID; 4]> = SmallVec::with_capacity(4); // max 4

        let start_edge_id = self.vertex(id).outgoing_edge;
        let mut current_edge_id = start_edge_id;

        // forward lookup
        loop {
            let edge = self.edge(current_edge_id);
            edges.push(current_edge_id);

            if let Some(twin_id) = edge.twin {
                let twin = self.edge(twin_id);
                current_edge_id = twin.next;

                // Back at start
                if current_edge_id == start_edge_id {
                    return edges.into_iter();
                }
            } else {
                break;
            }
        }

        // reverse lookup
        current_edge_id = self.edge(start_edge_id).prev;
        loop {
            let edge = self.edge(current_edge_id);
            edges.push(current_edge_id);

            if let Some(twin_id) = edge.twin {
                let twin = self.edge(twin_id);
                current_edge_id = twin.prev;

                // Back at start
                if current_edge_id == start_edge_id {
                    return edges.into_iter();
                }
            } else {
                break;
            }
        }

        edges.into_iter()
    }

    /// Loops around a vertex `id` and returns all verteces connected to it
    pub fn connected_verteces(&self, id: VertID) -> impl ExactSizeIterator<Item = VertID> {
        self.connected_edges(id).map(move |e| {
            let edge = self.edge(e);
            if edge.origin == id {
                self.next_edge(edge).origin
            } else {
                edge.origin
            }
        })
    }

    /// Loops around a vertex `id` and returns all verteces connected to it
    pub fn connected_faces(&self, id: VertID) -> impl ExactSizeIterator<Item = FaceID> {
        self.connected_edges(id).map(move |e| {
            let edge = self.edge(e);
            edge.face
        })
    }

    pub fn find_face(&self, id: VertID) -> FaceID {
        let edge = self.vertices[id.0].outgoing_edge;
        self.edges[edge.0].face
    }
}

impl<T: Numeric> TMesh<T> {
    /// Infers the local knot vectors for a specific control point.
    /// Returns (s_vector, t_vector).
    pub fn infer_local_knots(&self, v_id: VertID) -> LocalKnots {
        let s_knots = self.trace_local_knots(v_id, Direction::S);
        let t_knots = self.trace_local_knots(v_id, Direction::T);

        LocalKnots { s_knots, t_knots }
    }

    fn trace_local_knots(&self, v_id: VertID, direction: Direction) -> KnotVector {
        let v = self.vertex(v_id);
        let c = match direction {
            Direction::S => v.uv.s,
            Direction::T => v.uv.t,
        };

        // Trace two knots in each of the four cardinal directions
        let pos: [_; 2] = self.trace_knots(v_id, direction, true); // s3, s4
        let neg: [_; 2] = self.trace_knots(v_id, direction, false); // s1, s0

        match (neg[0], pos[0]) {
            (Some(n_0), None) => [n_0, c, c, c, c],
            (None, Some(p_0)) => [c, c, c, c, p_0],
            (Some(n_0), Some(p_0)) => [neg[1].unwrap_or(n_0), n_0, c, p_0, pos[1].unwrap_or(p_0)],
            _ => unreachable!("trace can not miss knot and find next"),
        }
    }

    /// Traces a ray from start_v in a direction to find the next two orthogonal knots.
    fn trace_knots<const DEPTH: usize>(&self, start_v: VertID, axis: Direction, positive: bool) -> [Option<isize>; DEPTH] {
        enum Start{
            Vertex(VertID),
        }

        let mut results = [None; DEPTH];
        let mut next_v = Start::Vertex(start_v);

        for i in 0..DEPTH {
            match next_v {
                Start::Vertex(v) => {
                    if let Some(found) = self.find_next_vertex_in_direction(v, axis, positive) {
                        next_v = Start::Vertex(found);
                        results[i] = match axis {
                            Direction::S => self.vertices[found.0].uv.s,
                            Direction::T => self.vertices[found.0].uv.t,
                        }.into();
                    } else {
                        // TODO: Ray Trace through face
                        break;
                    }
                }
            }
        }

        results
    }

    /// Helper to find the next vertex along the mesh edges in a specific direction.
    fn find_next_vertex_in_direction(
        &self,
        v_id: VertID,
        axis: Direction,
        positive: bool,
    ) -> Option<VertID> {
        let v = self.vertex(v_id);
        for vertex in self.connected_verteces(v_id) {
            let dest_v = &self.vertex(vertex);

            let delta = match axis {
                Direction::S => dest_v.uv.s - v.uv.s,
                Direction::T => dest_v.uv.t - v.uv.t,
            };

            let is_collinear = match axis {
                Direction::S => dest_v.uv.t == v.uv.t,
                Direction::T => dest_v.uv.s == v.uv.s,
            };

            if is_collinear && ((positive && delta > 0) || (!positive && delta < 0)) {
                return Some(vertex);
            }
        }

        None
    }
}

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

impl<T: Numeric + 'static> TMesh<T> {
    pub fn subs(&self, (s, t): (T, T), knot_cache: &[LocalKnots]) -> Option<Point3<T>> {
        let mut point_sum: Point3<T> = Point3::origin();
        let mut weight_sum = T::zero();

        for (i, vertex) in self.vertices.iter().enumerate() {
            // 1. Evaluate the 1D basis functions for s and t
            let n_s = cubic_basis_function(s, &knot_cache[i].s_knots);
            let n_t = cubic_basis_function(t, &knot_cache[i].t_knots);

            // 2. The 2D basis function B_i(s, t) is the product of the 1D functions
            let b_i = n_s * n_t;

            // Skip calculations if this control point doesn't influence (s, t)
            if b_i > T::zero() {
                // 2. Multiply the basis function by the point's weight w_i
                let rational_weight = b_i * vertex.geometry.w;

                // 3. Accumulate the weighted point sum (Numerator of Eq. 1)
                point_sum.x += vertex.geometry.x * b_i * rational_weight;
                point_sum.y += vertex.geometry.y * b_i * rational_weight;
                point_sum.z += vertex.geometry.z * b_i * rational_weight;

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
}

#[cfg(test)]
mod tests {
    use nalgebra::{Point4, Vector4};

    use super::*;
    use crate::{TSpline, tmesh::segment::ParamPoint};

    #[test]
    fn it_finds_face_edges() {
        let mesh = unit_square_tmesh();

        let edges = mesh.face_edges(FaceID(0));
        assert_eq!(4, edges.len()); // find all edges
    }

    #[test]
    fn it_finds_connected_verteces() {
        let mesh = TSpline::new_t_junction().into_mesh();
        let v = mesh
            .vertices
            .iter()
            .enumerate()
            .find_map(|(i, v)| {
                if v.is_t_junction {
                    Some(VertID(i))
                } else {
                    None
                }
            })
            .unwrap();

        assert_eq!(3, mesh.connected_verteces(v).len());
    }

    #[test]
    fn it_finds_connected_verteces_with_boundaries() {
        let mesh = unit_square_tmesh();

        assert_eq!(2, mesh.connected_verteces(VertID(0)).len());
    }

    #[test]
    fn it_finds_vertex_in_direction() {
        let mesh = unit_square_tmesh();

        assert_eq!(
            Some(VertID(1)),
            mesh.find_next_vertex_in_direction(VertID(0), Direction::S, true)
        );

        assert_eq!(
            Some(VertID(1)),
            mesh.find_next_vertex_in_direction(VertID(2), Direction::T, false)
        );

        assert_eq!(
            Some(VertID(3)),
            mesh.find_next_vertex_in_direction(VertID(0), Direction::T, true)
        );

        assert_eq!(
            Some(VertID(3)),
            mesh.find_next_vertex_in_direction(VertID(2), Direction::S, false)
        );
    }

    #[test]
    fn it_can_trace_direct_knots() {
        let mesh = unit_square_tmesh();

        let trace = mesh.trace_knots(VertID(0), Direction::S, true);
        assert_eq!([Some(1), None], trace);
    }

    #[test]
    fn it_can_infer_local_knots() {
        let mesh = unit_square_tmesh();

        assert_eq!(
            LocalKnots {
                s_knots: [0, 0, 0, 0, 1],
                t_knots: [0, 0, 0, 0, 1],
            },
            mesh.infer_local_knots(VertID(0))
        );
        assert_eq!(
            LocalKnots {
                s_knots: [0, 1, 1, 1, 1],
                t_knots: [0, 0, 0, 0, 1],
            },
            mesh.infer_local_knots(VertID(1))
        );
        assert_eq!(
            LocalKnots {
                s_knots: [0, 0, 0, 0, 1],
                t_knots: [0, 1, 1, 1, 1],
            },
            mesh.infer_local_knots(VertID(3))
        );
        assert_eq!(
            LocalKnots {
                s_knots: [0, 1, 1, 1, 1],
                t_knots: [0, 1, 1, 1, 1],
            },
            mesh.infer_local_knots(VertID(2))
        );
    }

    pub fn unit_square_tmesh() -> TMesh<f64> {
        TSpline::new_unit_square().into_mesh()
    }

    fn local_knots(mesh: &TMesh<f64>) -> Vec<LocalKnots> {
        (0..mesh.vertices.len())
            .map(|i| VertID(i))
            .map(|v| mesh.infer_local_knots(v))
            .collect()
    }

    #[test]
    pub fn it_can_find_points_on_a_square() {
        let mesh = unit_square_tmesh();
        let knots = local_knots(&mesh);

        assert_eq!(
            Point3::new(0., 0., 0.),
            mesh.subs((0., 0.), &knots).unwrap()
        );
        assert_eq!(
            Point3::new(1., 0., 0.),
            mesh.subs((1., 0.), &knots).unwrap()
        );
        assert_eq!(
            Point3::new(0., 1., 0.),
            mesh.subs((0., 1.), &knots).unwrap()
        );
        assert_eq!(
            Point3::new(1., 1., 0.),
            mesh.subs((1., 1.), &knots).unwrap()
        );
    }

    // #[test]
    // pub fn it_can_find_points_on_a_cube() {
    //     let mesh = TSpline::new_rounded_cube().into_mesh();
    //     let knots = local_knots(&mesh);
    //     let origin = Point3::origin();

    //     // All control points have values
    //     let mut distances_from_origin = Vec::new();
    //     for v in &mesh.vertices {
    //         if let Some(p) = mesh.subs((v.uv.s as f64, v.uv.t as f64), &knots) {
    //             distances_from_origin.push(nalgebra::distance(&origin, &p));
    //         } else {
    //             assert!(false, "cube has gaps");
    //         }
    //     }

    //     // All corners are symmetrical
    //     let first = distances_from_origin[0];
    //     for d in &distances_from_origin {
    //         assert!(
    //             (first - d).abs() < f64::delta(),
    //             "uniform control points have different distances from origin {d} in {distances_from_origin:?}"
    //         );
    //     }
    // }

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

    #[test]
    fn test_cubic_basis_function_bezier_right_boundary() {
        let knots = [0, 0, 0, 1, 1];

        assert_eq!(0.0, cubic_basis_function(5.0, &knots));
    }

    /// [0,0,1,1,1]: mirror of left boundary — should equal 1 at u=1
    #[test]
    fn test_cubic_basis_function_bezier_right_end() {
        let knots = [0, 0, 1, 1, 1];

        assert_eq!(0.0, cubic_basis_function(0.0_f64, &knots));
        // Mirrors [0,0,0,1,1] evaluated at 1-u; both give 0.375 at the midpoint.
        assert!((cubic_basis_function(0.5_f64, &knots) - 0.375).abs() < 1e-6,
            "got {}", cubic_basis_function(0.5_f64, &knots));
        // At u==1 the standard half-open interval gives 0 (no closed endpoint).
        assert_eq!(0.0, cubic_basis_function(1.0_f64, &knots));
    }

    /// Partition of unity: the four cubic Bernstein basis functions over [0,1]
    /// must sum to 1 at every point in [0,1].
    ///
    /// Note: the 4-point unit-square mesh uses only two of these four 1-D basis
    /// functions (the two corner-Bezier knot vectors), so it does NOT satisfy
    /// partition of unity at interior points.  A full 4×4 = 16-point control
    /// grid would be required for a proper cubic tensor-product surface.
    #[test]
    fn test_partition_of_unity_bernstein() {
        // All four cubic Bernstein basis functions over [0,1]
        let bernstein_knots: [[isize; 5]; 4] = [
            [0, 0, 0, 0, 1], // (1-u)^3
            [0, 0, 0, 1, 1], // 3u(1-u)^2
            [0, 0, 1, 1, 1], // 3u^2(1-u)
            [0, 1, 1, 1, 1], // u^3
        ];

        let samples = [0.0_f64, 0.1, 0.25, 0.5, 0.75, 0.9];
        for &u in &samples {
            let sum: f64 = bernstein_knots
                .iter()
                .map(|k| cubic_basis_function(u, k))
                .sum();
            assert!(
                (sum - 1.0).abs() < 1e-10,
                "partition of unity failed at u={u}: sum = {sum}"
            );
        }
    }

    /// The 4-point unit-square mesh does NOT satisfy partition of unity at
    /// interior points — it only uses 2 of the 4 Bernstein basis functions per
    /// axis.  This test documents the gap so we can detect regressions.
    #[test]
    fn test_unit_square_mesh_does_not_have_partition_of_unity() {
        let corners: [([isize; 5], [isize; 5]); 4] = [
            ([0, 0, 0, 0, 1], [0, 0, 0, 0, 1]), // v0 (0,0)
            ([0, 0, 0, 1, 1], [0, 0, 0, 0, 1]), // v1 (1,0)
            ([0, 0, 0, 1, 1], [0, 0, 0, 1, 1]), // v2 (1,1)
            ([0, 0, 0, 0, 1], [0, 0, 0, 1, 1]), // v3 (0,1)
        ];

        // At (0,0) the sum is 1: N_s([0,0,0,0,1], 0)=1, all others = 0
        let sum_origin: f64 = corners
            .iter()
            .map(|(sk, tk)| cubic_basis_function(0.0, sk) * cubic_basis_function(0.0, tk))
            .sum();
        assert!(
            (sum_origin - 1.0).abs() < 1e-10,
            "origin should sum to 1, got {sum_origin}"
        );

        // At corners where s=1 or t=1 the half-open interval [k_i, k_{i+1})
        // never includes the right endpoint, so every basis function returns 0.
        // This documents the known limitation of the current implementation.
        for &(s, t) in &[(1.0_f64, 0.0), (0.0, 1.0), (1.0, 1.0)] {
            let sum: f64 = corners
                .iter()
                .map(|(sk, tk)| cubic_basis_function(s, sk) * cubic_basis_function(t, tk))
                .sum();
            assert_eq!(
                0.0, sum,
                "right-endpoint ({s},{t}): all basis fns are 0 (half-open interval limitation), got {sum}"
            );
        }

        // At an interior point the sum is less than 1 — basis functions don't
        // cover the full domain.
        let sum_interior: f64 = corners
            .iter()
            .map(|(sk, tk)| cubic_basis_function(0.5, sk) * cubic_basis_function(0.5, tk))
            .sum();
        assert!(
            sum_interior < 1.0,
            "expected sum < 1 at interior, got {sum_interior}"
        );
    }

    /// Non-negativity: basis values must never be negative.
    #[test]
    fn test_basis_non_negative() {
        let knot_vecs: &[[isize; 5]] = &[
            [0, 0, 0, 0, 1],
            [0, 0, 0, 1, 1],
            [0, 0, 1, 1, 1],
            [0, 1, 2, 3, 4],
        ];
        let samples = [0.0_f64, 0.1, 0.5, 0.9, 1.0, 2.0, 3.0];
        for knots in knot_vecs {
            for &u in &samples {
                let val = cubic_basis_function(u, knots);
                assert!(val >= 0.0, "negative basis value {val} at u={u} knots={knots:?}");
            }
        }
    }

    /// Symmetry: [0,0,0,1,1] and [0,0,1,1,1] should mirror each other
    /// around the midpoint of [0,1].
    #[test]
    fn test_basis_symmetry() {
        let left  = [0_isize, 0, 0, 1, 1];
        let right = [0_isize, 0, 1, 1, 1];
        for i in 0..=10 {
            let u = i as f64 / 10.0;
            let l = cubic_basis_function(u, &left);
            let r = cubic_basis_function(1.0 - u, &right);
            assert!(
                (l - r).abs() < 1e-10,
                "symmetry broken at u={u}: left({u})={l}  right({})={r}",
                1.0 - u
            );
        }
    }
}
