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

pub mod direction;
pub mod half_edge;
pub mod ids;
pub mod uv_point;

use crate::Numeric;
use crate::line::Line;
use crate::uv_mesh::direction::Direction;
use crate::uv_mesh::half_edge::HalfEdge;
use crate::uv_mesh::ids::{EdgeID, VertID};
use crate::uv_mesh::uv_point::{UVCoord, UVPoint};
use alloc::collections::BTreeSet;
use alloc::vec::Vec;
use smallvec::SmallVec;
use thiserror::Error;

const INVALID_MESH: &'static str = "invalid mesh, validate to avoid panics";

/// A local knot vector consisting of 5 knots for a cubic T-spline.
///
/// In T-spline representations, each control point is associated with a local
/// knot vector in both the `u` and `v` parametric directions. For a cubic basis
/// function (degree 3), exactly 5 knots are required to evaluate the blending
/// weights over its local support domain.
///
/// The knots are typically ordered such that $k_0 \le k_1 \le k_2 \le k_3 \le k_4$.
pub type KnotVector = [isize; 5];

/// Two directional knot vectors for S & T directions.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LocalKnots {
    pub s_knots: KnotVector,
    pub t_knots: KnotVector,
}

pub trait UVMeshMut: UVMesh {
    fn push_point(&mut self, point: UVPoint) -> VertID;
    fn push_edge(&mut self, edge: HalfEdge) -> EdgeID;
    fn edge_mut(&mut self, id: EdgeID) -> Option<&mut HalfEdge>;
}

pub trait UVMesh {
    fn points(&self) -> &[UVPoint];
    fn edges(&self) -> &[HalfEdge];

    fn edge(&self, id: EdgeID) -> Option<&HalfEdge> {
        self.edges().get(id.0)
    }
    fn point(&self, id: VertID) -> Option<&UVPoint> {
        self.points().get(id.0)
    }

    fn next_edge(&self, edge: &HalfEdge) -> &HalfEdge {
        &self.edge(edge.next).expect(INVALID_MESH)
    }

    fn connected_edges(&self, id: VertID) -> impl ExactSizeIterator<Item = EdgeID> {
        let mut edges: SmallVec<[EdgeID; 4]> = SmallVec::with_capacity(4); // max 4

        let start_edge_id = self.point(id).expect(INVALID_MESH).outgoing_edge;
        let mut current_edge_id = start_edge_id;

        // forward lookup
        loop {
            let edge = self.edge(current_edge_id).expect(INVALID_MESH);
            edges.push(current_edge_id);

            if let Some(twin_id) = edge.twin {
                let twin = self.edge(twin_id).expect(INVALID_MESH);
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
        current_edge_id = self.edge(start_edge_id).expect(INVALID_MESH).prev;
        loop {
            let edge = self.edge(current_edge_id).expect(INVALID_MESH);
            edges.push(current_edge_id);

            if let Some(twin_id) = edge.twin {
                let twin = self.edge(twin_id).expect(INVALID_MESH);
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

    /// Compute all local knots
    fn local_knots(&self) -> Vec<LocalKnots> {
        (0..self.points().len())
            .map(|i| VertID(i))
            .map(|v| self.infer_local_knots(v))
            .collect()
    }

    fn line(&self, edge: &HalfEdge) -> Line<isize> {
        Line::from_uv_points(
            self.point(edge.origin).expect(INVALID_MESH),
            self.point(self.next_edge(edge).origin).expect(INVALID_MESH),
        )
    }

    fn start_end(&self, edge: &HalfEdge) -> (&UVPoint, &UVPoint) {
        (
            self.point(edge.origin).expect(INVALID_MESH),
            self.point(self.next_edge(edge).origin).expect(INVALID_MESH),
        )
    }

    fn edge_loop<'a>(&'a self, edge: &'a HalfEdge) -> impl Iterator<Item = (EdgeID, &'a HalfEdge)> {
        let start = edge.origin;
        let mut current = edge;
        let mut edges = Vec::new();

        loop {
            let next_id = current.next;
            current = self.next_edge(&current);
            edges.push((next_id, current));

            if current.origin == start {
                break;
            }
        }

        edges.into_iter()
    }

    /// Loops around a vertex `id` and returns all verteces connected to it
    fn connected_verteces(&self, id: VertID) -> impl ExactSizeIterator<Item = VertID> {
        self.connected_edges(id).map(move |e| {
            let edge = self.edge(e).expect(INVALID_MESH);
            if edge.origin == id {
                self.next_edge(edge).origin
            } else {
                edge.origin
            }
        })
    }

    /// The distinct list of faces within the mesh, each represented by a unique edge
    fn faces(&self) -> impl Iterator<Item = EdgeID> {
        let mut faces = Vec::new();
        let mut seen = BTreeSet::new();

        for (i, edge) in self.edges().iter().enumerate() {
            if seen.contains(&EdgeID(i)) {
                continue;
            } else {
                faces.push(EdgeID(i));
            }

            for (i, _) in self.edge_loop(edge) {
                if !seen.contains(&i) {
                    seen.insert(i);
                }
            }
        }

        faces.into_iter()
    }

    fn contains_uv<T: Numeric + 'static>(&self, point: impl UVCoord<T>) -> bool {
        let mut intersections = 0;
        for edge in self.edges() {
            let line: Line<T> = Line::from_isize(self.line(edge)).unwrap();

            if line.is_touching(&point) {
                return true; // on the edge, must be inside shape
            }

            if line.intersection(&point, Direction::T, true).is_some() {
                intersections += 1;
            }
        }

        intersections % 2 == 1
    }

    /// Infers the local knot vectors for a specific control point.
    /// Returns (s_vector, t_vector).
    fn infer_local_knots(&self, v_id: VertID) -> LocalKnots {
        let s_knots = self.trace_local_knots(v_id, Direction::S);
        let t_knots = self.trace_local_knots(v_id, Direction::T);

        LocalKnots { s_knots, t_knots }
    }

    fn trace_local_knots(&self, v_id: VertID, direction: Direction) -> KnotVector {
        let v = self.point(v_id).expect(INVALID_MESH);
        let c = match direction {
            Direction::S => v.s,
            Direction::T => v.t,
        };

        // Trace two knots in each of the four cardinal directions
        let pos: [_; 2] = self.trace_knots(v_id, direction, true); // s3, s4
        let neg: [_; 2] = self.trace_knots(v_id, direction, false); // s1, s0

        match (neg[0], pos[0]) {
            (Some(n_0), None) => [n_0, c, c, c, c],
            (None, Some(p_0)) => [c, c, c, c, p_0],
            (Some(n_0), Some(p_0)) => [neg[1].unwrap_or(n_0), n_0, c, p_0, pos[1].unwrap_or(p_0)],
            (None, None) => unreachable!(),
        }
    }

    /// Traces a ray from start_v in a direction to find the next two orthogonal knots.
    fn trace_knots<const DEPTH: usize>(
        &self,
        start_v: VertID,
        axis: Direction,
        positive: bool,
    ) -> [Option<isize>; DEPTH] {
        enum Start {
            Vertex(VertID),
            Hit(UVPoint),
        }

        let mut results = [None; DEPTH];
        let mut next_v = Start::Vertex(start_v);

        for i in 0..DEPTH {
            match next_v {
                Start::Vertex(v) => {
                    if let Some(found) = self.find_next_vertex_in_direction(v, axis, positive) {
                        let point = self.point(found).expect(INVALID_MESH);
                        results[i] = point.value_in_dir(axis).into();
                        next_v = Start::Vertex(found);
                    } else if let Some(point) =
                        self.trace_in_direction(self.point(v).expect(INVALID_MESH), axis, positive)
                    {
                        results[i] = point.value_in_dir(axis).into();
                        next_v = Start::Hit(point);
                    } else {
                        break;
                    }
                }
                Start::Hit(p) => {
                    if let Some(point) = self.trace_in_direction(&p, axis, positive) {
                        results[i] = point.value_in_dir(axis).into();
                        next_v = Start::Hit(point);
                    } else {
                        break;
                    }
                }
            }
        }

        results
    }

    fn trace_for_edge_loop(
        &self,
        edge: &HalfEdge,
        start: &UVPoint,
        axis: Direction,
        positive: bool,
    ) -> Option<UVPoint> {
        for line in self.edge_loop(edge).map(|(_, e)| self.line(e)) {
            if line.is_touching::<UVPoint>(start) {
                continue;
            }

            if let Some(intersection) = line.intersection(start, axis, positive) {
                return Some(intersection);
            }
        }
        None
    }

    fn trace_in_direction(
        &self,
        start: &UVPoint,
        axis: Direction,
        positive: bool,
    ) -> Option<UVPoint> {
        let edge = self.edge(start.outgoing_edge).expect(INVALID_MESH);
        if let Some(i) = self.trace_for_edge_loop(&edge, start, axis, positive) {
            return Some(i);
        }

        if let Some(twin_id) = edge.twin {
            let twin = self.edge(twin_id).expect(INVALID_MESH);
            if let Some(i) = self.trace_for_edge_loop(&twin, start, axis, positive) {
                return Some(i);
            }
        }

        None
    }

    /// Helper to find the next vertex along the mesh edges in a specific direction.
    fn find_next_vertex_in_direction(
        &self,
        v_id: VertID,
        axis: Direction,
        positive: bool,
    ) -> Option<VertID> {
        let v = self.point(v_id).expect(INVALID_MESH);
        for vertex in self.connected_verteces(v_id) {
            let dest_v = &self.point(vertex).expect(INVALID_MESH);
            let l = Line::from_uv_points(dest_v, v);
            let delta = l.delta(axis);

            if l.is_axis_aligned(axis) && ((positive && delta > 0) || (!positive && delta < 0)) {
                return Some(vertex);
            }
        }

        None
    }

    fn validate_uv_mesh_integrity(&self) -> Result<(), ValidationError> {
        for point in self.points() {
            if let Some(edge) = self.edge(point.outgoing_edge) {
                let l = self.line(edge);
                if !l.is_orthogonal() {
                    return Err(ValidationError::NonOrthogonal());
                }
                if l.length() == 0 {
                    return Err(ValidationError::ZeroLengthEdge());
                }

                if let Some(twin_id) = edge.twin {
                    if let Some(twin) = self.edge(twin_id) {
                        let l = self.line(twin);
                        if !l.is_orthogonal() {
                            return Err(ValidationError::NonOrthogonal());
                        }
                        if l.length() == 0 {
                            return Err(ValidationError::ZeroLengthEdge());
                        }
                    }
                }
            } else {
                return Err(ValidationError::InvalidOutgoingEdge());
            }
        }

        for edge in self.edges() {
            if self.point(edge.origin).is_none() {
                return Err(ValidationError::InvalidOrigin());
            }

            if let Some(_) = self.edge(edge.next) {
                // TODO: check next and prev
            } else {
                return Err(ValidationError::InvalidNextEdge());
            }

            if self.edge(edge.prev).is_none() {
                return Err(ValidationError::InvalidPrevEdge());
            }

            if let Some(twin) = edge.twin {
                if let Some(twin) = self.edge(twin) {
                    if let Some(_origin_twin) = twin.twin {
                        // TODO: check if origin_twin == edge.id
                    } else {
                        return Err(ValidationError::InvalidTwinEdge());
                    }

                    if edge.origin != self.edge(twin.next).unwrap().origin {
                        return Err(ValidationError::MisalignedTwin());
                    }

                    if twin.origin != self.edge(edge.next).unwrap().origin {
                        return Err(ValidationError::MisalignedTwin());
                    }
                } else {
                    return Err(ValidationError::InvalidTwinEdge());
                }
            }
        }

        Ok(())
    }

    fn validate_is_manifold(&self) -> Result<(), ValidationError> {
        for edge in self.edges() {
            if edge.twin.is_none() {
                return Err(ValidationError::NonManifold());
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Error, PartialEq)]
pub enum ValidationError {
    #[error("edge is non manifold (missing twin)")]
    NonManifold(),
    #[error("edge uv points are not othogonal")]
    NonOrthogonal(),
    #[error("edge must have a length")]
    ZeroLengthEdge(),
    #[error("outgoing edge is an invalid reference")]
    InvalidOutgoingEdge(),
    #[error("origin is an invalid reference")]
    InvalidOrigin(),
    #[error("next edge is an invalid reference")]
    InvalidNextEdge(),
    #[error("previous edge is an invalid reference")]
    InvalidPrevEdge(),
    #[error("twin is an invalid reference")]
    InvalidTwinEdge(),
    #[error("points and control points are mismatched")]
    DisconnectedPoints(),
    #[error("twin does not align")]
    MisalignedTwin(),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TSpline;

    #[test]
    fn it_has_valid_unit_square() {
        let mesh = TSpline::new_unit_square();
        assert_eq!(Ok(()), mesh.validate_uv_mesh_integrity());
    }

    #[test]
    fn it_finds_edge_loops() {
        let mesh = TSpline::new_unit_square();
        assert_eq!(4, mesh.edge_loop(&mesh.edges[0]).collect::<Vec<_>>().len());
    }

    #[test]
    fn it_finds_connected_edges() {
        let mesh = TSpline::new_unit_square();

        let edges = mesh.connected_edges(VertID(0));
        assert_eq!(2, edges.len()); // find edge in both directions
    }

    #[test]
    fn it_finds_vertex_in_direction() {
        let mesh = TSpline::new_unit_square();

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
        let mesh = TSpline::new_unit_square();

        let trace = mesh.trace_knots(VertID(0), Direction::S, true);
        assert_eq!([Some(1), None], trace);
    }

    #[test]
    fn it_can_infer_local_knots() {
        let mesh = TSpline::new_unit_square();

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

    #[test]
    fn it_finds_faces() {
        let mesh = TSpline::new_unit_square();

        assert_eq!(1, mesh.faces().count());
    }
}
