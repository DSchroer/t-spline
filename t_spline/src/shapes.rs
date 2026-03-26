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

use crate::tmesh::control_point::ControlPoint;
use crate::tmesh::face::Face;
use crate::tmesh::half_edge::HalfEdge;
use crate::tmesh::ids::{EdgeID, FaceID, VertID};
use crate::tmesh::segment::ParamPoint;
use crate::tmesh::*;
use crate::*;
use alloc::vec::Vec;
use nalgebra::Vector4;

impl<T: Numeric> TSpline<T> {
    pub fn new_unit_square() -> TSpline<T> {
        let mut mesh = TMesh {
            vertices: Vec::with_capacity(4),
            edges: Vec::with_capacity(4),
            faces: Vec::with_capacity(1),
        };

        // 1. Define 4 Corner Vertices
        let coords = [
            (T::zero(), T::zero()),
            (T::one(), T::zero()),
            (T::one(), T::one()),
            (T::zero(), T::one()),
        ];
        for (i, (s, t)) in coords.iter().enumerate() {
            mesh.vertices.push(ControlPoint {
                geometry: Vector4::new(*s, *t, T::zero(), T::one()),
                uv: ParamPoint {
                    s: s.to_isize().unwrap(),
                    t: t.to_isize().unwrap(),
                },
                outgoing_edge: EdgeID(i), // Inner edges are 0..4
                is_t_junction: false,
            });
        }

        // 2. Define 4 inner Half-Edges in a CCW loop
        for i in 0..4 {
            mesh.edges.push(HalfEdge {
                origin: VertID(i),
                next: EdgeID((i + 1) % 4),
                prev: EdgeID((i + 3) % 4),
                twin: None,
                face: FaceID(0),
            });
        }

        // 4. Define the Face
        mesh.faces.push(Face { edge: EdgeID(0) });

        mesh.into()
    }
}

impl TSpline<f64> {
    pub fn new_rounded_cube() -> TSpline<f64> {
        let mut mesh = TMesh {
            vertices: Vec::with_capacity(14),
            edges: Vec::with_capacity(24),
            faces: Vec::with_capacity(6),
        };

        // Vertices for an unfolded cube (cross layout)
        // F is center at (1,1)-(2,2)
        // L left, R right, B right of R
        // Top above F, Bot below F
        let raw_verts = [
            (0.0, 1.0, -1.0, -1.0, -1.0), // 0: L-bot-left
            (1.0, 1.0, -1.0, -1.0, 1.0),  // 1: L-bot-right / F-bot-left
            (2.0, 1.0, 1.0, -1.0, 1.0),   // 2: F-bot-right / R-bot-left
            (3.0, 1.0, 1.0, -1.0, -1.0),  // 3: R-bot-right / B-bot-left
            (4.0, 1.0, -1.0, -1.0, -1.0), // 4: B-bot-right
            (0.0, 2.0, -1.0, 1.0, -1.0),  // 5: L-top-left
            (1.0, 2.0, -1.0, 1.0, 1.0),   // 6: L-top-right / F-top-left
            (2.0, 2.0, 1.0, 1.0, 1.0),    // 7: F-top-right / R-top-left
            (3.0, 2.0, 1.0, 1.0, -1.0),   // 8: R-top-right / B-top-left
            (4.0, 2.0, -1.0, 1.0, -1.0),  // 9: B-top-right
            (1.0, 0.0, -1.0, -1.0, -1.0), // 10: Bot-bot-left
            (2.0, 0.0, 1.0, -1.0, -1.0),  // 11: Bot-bot-right
            (1.0, 3.0, -1.0, 1.0, -1.0),  // 12: Top-top-left
            (2.0, 3.0, 1.0, 1.0, -1.0),   // 13: Top-top-right
        ];

        for (u, v, x, y, z) in raw_verts.iter() {
            mesh.vertices.push(ControlPoint {
                geometry: Vector4::new(*x, *y, *z, 1.0),
                uv: ParamPoint {
                    s: *u as isize,
                    t: *v as isize,
                },
                outgoing_edge: EdgeID(0),
                is_t_junction: false,
            });
        }

        let mut add_edge =
            |origin: usize, next: usize, prev: usize, twin: Option<usize>, face: usize| {
                let id = mesh.edges.len();
                mesh.edges.push(HalfEdge {
                    origin: VertID(origin),
                    next: EdgeID(next),
                    prev: EdgeID(prev),
                    twin: twin.map(EdgeID),
                    face: FaceID(face),
                });
                mesh.vertices[origin].outgoing_edge = EdgeID(id);
            };

        // L (Face 0)
        add_edge(0, 1, 3, None, 0);      // 0  boundary
        add_edge(1, 2, 0, Some(7), 0);   // 1  (Twin 7)  L-F shared
        add_edge(6, 3, 1, None, 0);      // 2  boundary
        add_edge(5, 0, 2, None, 0);      // 3  boundary

        // F (Face 1)
        add_edge(1, 5, 7, Some(18), 1);  // 4  (Twin 18) F-Bot shared
        add_edge(2, 6, 4, Some(11), 1);  // 5  (Twin 11) F-R shared
        add_edge(7, 7, 5, Some(20), 1);  // 6  (Twin 20) F-Top shared
        add_edge(6, 4, 6, Some(1), 1);   // 7  (Twin 1)  F-L shared

        // R (Face 2)
        add_edge(2, 9, 11, None, 2);     // 8  boundary
        add_edge(3, 10, 8, Some(15), 2); // 9  (Twin 15) R-B shared
        add_edge(8, 11, 9, None, 2);     // 10 boundary
        add_edge(7, 8, 10, Some(5), 2);  // 11 (Twin 5)  R-F shared

        // B (Face 3)
        add_edge(3, 13, 15, None, 3);    // 12 boundary
        add_edge(4, 14, 12, None, 3);    // 13 boundary
        add_edge(9, 15, 13, None, 3);    // 14 boundary
        add_edge(8, 12, 14, Some(9), 3); // 15 (Twin 9)  B-R shared

        // Bot (Face 4)
        add_edge(10, 17, 19, None, 4);   // 16 boundary
        add_edge(11, 18, 16, None, 4);   // 17 boundary
        add_edge(2, 19, 17, Some(4), 4); // 18 (Twin 4)  Bot-F shared
        add_edge(1, 16, 18, None, 4);    // 19 boundary

        // Top (Face 5)
        add_edge(6, 21, 23, Some(6), 5); // 20 (Twin 6)  Top-F shared
        add_edge(7, 22, 20, None, 5);    // 21 boundary
        add_edge(13, 23, 21, None, 5);   // 22 boundary
        add_edge(12, 20, 22, None, 5);   // 23 boundary

        // Faces
        for i in 0..6 {
            mesh.faces.push(Face {
                edge: EdgeID(i * 4),
            });
        }

        mesh.into()
    }

    /// Creates a simple T-Spline mesh with a T-junction, which is impossible
    /// to represent as a single NURBS patch.
    /// Topology:
    /// F0 (Left): (0,0)-(1,0)-(1,1)-(1,2)-(0,2) [Pentagon with T-junction at (1,1)]
    /// F1 (Bot-Right): (1,0)-(2,0)-(2,1)-(1,1)
    /// F2 (Top-Right): (1,1)-(2,1)-(2,2)-(1,2)
    pub fn new_t_junction() -> TSpline<f64> {
        let mut mesh = TMesh {
            vertices: Vec::with_capacity(8),
            edges: Vec::with_capacity(20),
            faces: Vec::with_capacity(3),
        };

        // 1. Vertices
        let coords = [
            (0.0, 0.0), // 0
            (1.0, 0.0), // 1
            (2.0, 0.0), // 2
            (1.0, 1.0), // 3 (T-Junction)
            (2.0, 1.0), // 4
            (0.0, 2.0), // 5
            (1.0, 2.0), // 6
            (2.0, 2.0), // 7
        ];

        for (i, (s, t)) in coords.iter().enumerate() {
            let is_t = i == 3;
            // Outgoing edge will be set by the edge construction below.
            // We set a default here, but correct it later or ensure we point to a valid one.
            // For safety, let's just pick the first edge that starts at this vertex.
            // But since we haven't built edges, we'll patch it or use a logic.
            // Here, we just push with None and fill later?
            // TMesh struct expects Option<EdgeID>.
            mesh.vertices.push(ControlPoint {
                geometry: Vector4::new(*s, *t, 0.0, 1.0),
                uv: ParamPoint {
                    s: *s as isize,
                    t: *t as isize,
                },
                outgoing_edge: EdgeID(0),
                is_t_junction: is_t,
            });
        }

        // Helper to add edge
        let mut add_edge =
            |origin: usize, next: usize, prev: usize, twin: Option<usize>, face: usize| {
                let id = mesh.edges.len();
                mesh.edges.push(HalfEdge {
                    origin: VertID(origin),
                    next: EdgeID(next),
                    prev: EdgeID(prev),
                    twin: twin.map(EdgeID),
                    face: FaceID(face),
                });
                // Update vertex outgoing edge if not set
                mesh.vertices[origin].outgoing_edge = EdgeID(id);
            };

        // Inner Half-Edges
        // F0 (0->1->3->6->5->0)
        add_edge(0, 1, 4, Some(13), 0); // 0
        add_edge(1, 2, 0, Some(8), 0); // 1
        add_edge(3, 3, 1, Some(12), 0); // 2
        add_edge(6, 4, 2, Some(18), 0); // 3 (Actually -S)
        add_edge(5, 0, 3, Some(19), 0); // 4 (Actually -T)

        // F1 (1->2->4->3->1)
        add_edge(1, 6, 8, Some(14), 1); // 5
        add_edge(2, 7, 5, Some(15), 1); // 6
        add_edge(4, 8, 6, Some(9), 1); // 7 (Actually -S)
        add_edge(3, 5, 7, Some(1), 1); // 8 (Actually -T)

        // F2 (3->4->7->6->3)
        add_edge(3, 10, 12, Some(7), 2); // 9
        add_edge(4, 11, 9, Some(16), 2); // 10
        add_edge(7, 12, 10, Some(17), 2); // 11 (Actually -S)
        add_edge(6, 9, 11, Some(2), 2); // 12 (Actually -T)

        // Faces
        mesh.faces.push(Face { edge: EdgeID(0) });
        mesh.faces.push(Face { edge: EdgeID(5) });
        mesh.faces.push(Face { edge: EdgeID(9) });

        mesh.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rounded_cube_topology() {
        let spline = TSpline::new_rounded_cube();
        let mesh = spline.into_mesh();

        assert_eq!(mesh.vertices.len(), 14, "Should have 14 vertices");
        assert_eq!(mesh.edges.len(), 24, "Should have 24 half-edges");
        assert_eq!(mesh.faces.len(), 6, "Should have 6 faces");

        for e in mesh.edges.iter() {
            assert!(e.twin.is_some(), "Should have twin");
        }
    }
}
