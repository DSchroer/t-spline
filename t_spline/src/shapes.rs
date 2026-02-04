use crate::tmesh::control_point::ControlPoint;
use crate::tmesh::direction::Direction;
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
            edges: Vec::with_capacity(8),
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
                uv: ParamPoint { s: *s, t: *t },
                outgoing_edge: Some(EdgeID(i)), // Inner edges are 0..4
                is_t_junction: false,
            });
        }

        // 2. Define 4 inner Half-Edges in a CCW loop
        for i in 0..4 {
            mesh.edges.push(HalfEdge {
                origin: VertID(i),
                next: EdgeID((i + 1) % 4),
                prev: EdgeID((i + 3) % 4),
                twin: Some(EdgeID(i + 4)), // Outer edges are 4..8
                face: Some(FaceID(0)),
                knot_interval: 1.0,
                direction: if i % 2 == 0 {
                    Direction::S
                } else {
                    Direction::T
                },
            });
        }

        // 3. Define 4 outer Half-Edges in a CW loop
        for i in 0..4 {
            let origin_v_idx = (i + 1) % 4;
            let next_outer_edge_idx = ((i + 3) % 4) + 4;
            let prev_outer_edge_idx = ((i + 1) % 4) + 4;

            mesh.edges.push(HalfEdge {
                origin: VertID(origin_v_idx),
                next: EdgeID(next_outer_edge_idx),
                prev: EdgeID(prev_outer_edge_idx),
                twin: Some(EdgeID(i)),
                face: None,
                knot_interval: 1.0,
                direction: if i % 2 == 0 {
                    Direction::S
                } else {
                    Direction::T
                },
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
                uv: ParamPoint { s: *u, t: *v },
                outgoing_edge: None,
                is_t_junction: false,
            });
        }

        let mut add_edge = |origin: usize,
                            next: usize,
                            prev: usize,
                            twin: Option<usize>,
                            face: usize,
                            dir: Direction| {
            let id = mesh.edges.len();
            mesh.edges.push(HalfEdge {
                origin: VertID(origin),
                next: EdgeID(next),
                prev: EdgeID(prev),
                twin: twin.map(EdgeID),
                face: Some(FaceID(face)),
                knot_interval: 1.0,
                direction: dir,
            });
            if mesh.vertices[origin].outgoing_edge.is_none() {
                mesh.vertices[origin].outgoing_edge = Some(EdgeID(id));
            }
        };

        // L (Face 0)
        add_edge(0, 1, 3, None, 0, Direction::S); // 0
        add_edge(1, 2, 0, Some(7), 0, Direction::T); // 1 (Twin 7)
        add_edge(6, 3, 1, None, 0, Direction::S); // 2
        add_edge(5, 0, 2, None, 0, Direction::T); // 3

        // F (Face 1)
        add_edge(1, 5, 7, Some(18), 1, Direction::S); // 4 (Twin 18)
        add_edge(2, 6, 4, Some(11), 1, Direction::T); // 5 (Twin 11)
        add_edge(7, 7, 5, Some(20), 1, Direction::S); // 6 (Twin 20)
        add_edge(6, 4, 6, Some(1), 1, Direction::T); // 7 (Twin 1)

        // R (Face 2)
        add_edge(2, 9, 11, None, 2, Direction::S); // 8
        add_edge(3, 10, 8, Some(15), 2, Direction::T); // 9 (Twin 15)
        add_edge(8, 11, 9, None, 2, Direction::S); // 10
        add_edge(7, 8, 10, Some(5), 2, Direction::T); // 11 (Twin 5)

        // B (Face 3)
        add_edge(3, 13, 15, None, 3, Direction::S); // 12
        add_edge(4, 14, 12, None, 3, Direction::T); // 13
        add_edge(9, 15, 13, None, 3, Direction::S); // 14
        add_edge(8, 12, 14, Some(9), 3, Direction::T); // 15 (Twin 9)

        // Bot (Face 4)
        add_edge(10, 17, 19, None, 4, Direction::S); // 16
        add_edge(11, 18, 16, None, 4, Direction::T); // 17
        add_edge(2, 19, 17, Some(4), 4, Direction::S); // 18 (Twin 4)
        add_edge(1, 16, 18, None, 4, Direction::T); // 19

        // Top (Face 5)
        add_edge(6, 21, 23, Some(6), 5, Direction::S); // 20 (Twin 6)
        add_edge(7, 22, 20, None, 5, Direction::T); // 21
        add_edge(13, 23, 21, None, 5, Direction::S); // 22
        add_edge(12, 20, 22, None, 5, Direction::T); // 23

        // Faces
        for i in 0..6 {
            mesh.faces.push(Face { edge: EdgeID(i * 4) });
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
                uv: ParamPoint { s: *s, t: *t },
                outgoing_edge: None,
                is_t_junction: is_t,
            });
        }

        // Helper to add edge
        let mut add_edge = |origin: usize,
                            next: usize,
                            prev: usize,
                            twin: Option<usize>,
                            face: Option<usize>,
                            dir: Direction| {
            let id = mesh.edges.len();
            mesh.edges.push(HalfEdge {
                origin: VertID(origin),
                next: EdgeID(next),
                prev: EdgeID(prev),
                twin: twin.map(EdgeID),
                face: face.map(FaceID),
                knot_interval: 1.0, // Simplified
                direction: dir,
            });
            // Update vertex outgoing edge if not set
            if mesh.vertices[origin].outgoing_edge.is_none() {
                mesh.vertices[origin].outgoing_edge = Some(EdgeID(id));
            }
        };

        // Inner Half-Edges
        // F0 (0->1->3->6->5->0)
        add_edge(0, 1, 4, Some(13), Some(0), Direction::S); // 0
        add_edge(1, 2, 0, Some(8), Some(0), Direction::T); // 1
        add_edge(3, 3, 1, Some(12), Some(0), Direction::T); // 2
        add_edge(6, 4, 2, Some(18), Some(0), Direction::S); // 3 (Actually -S)
        add_edge(5, 0, 3, Some(19), Some(0), Direction::T); // 4 (Actually -T)

        // F1 (1->2->4->3->1)
        add_edge(1, 6, 8, Some(14), Some(1), Direction::S); // 5
        add_edge(2, 7, 5, Some(15), Some(1), Direction::T); // 6
        add_edge(4, 8, 6, Some(9), Some(1), Direction::S); // 7 (Actually -S)
        add_edge(3, 5, 7, Some(1), Some(1), Direction::T); // 8 (Actually -T)

        // F2 (3->4->7->6->3)
        add_edge(3, 10, 12, Some(7), Some(2), Direction::S); // 9
        add_edge(4, 11, 9, Some(16), Some(2), Direction::T); // 10
        add_edge(7, 12, 10, Some(17), Some(2), Direction::S); // 11 (Actually -S)
        add_edge(6, 9, 11, Some(2), Some(2), Direction::T); // 12 (Actually -T)

        // Boundary Half-Edges
        // Loop: 13->19->18->17->16->15->14->13
        add_edge(1, 19, 14, Some(0), None, Direction::S); // 13 (Twin of 0)
        add_edge(2, 13, 15, Some(5), None, Direction::S); // 14 (Twin of 5)
        add_edge(4, 14, 16, Some(6), None, Direction::T); // 15 (Twin of 6)
        add_edge(7, 15, 17, Some(10), None, Direction::T); // 16 (Twin of 10)
        add_edge(6, 16, 18, Some(11), None, Direction::S); // 17 (Twin of 11)
        add_edge(5, 17, 19, Some(3), None, Direction::S); // 18 (Twin of 3)
        add_edge(0, 18, 13, Some(4), None, Direction::T); // 19 (Twin of 4)

        // Faces
        mesh.faces.push(Face { edge: EdgeID(0) });
        mesh.faces.push(Face { edge: EdgeID(5) });
        mesh.faces.push(Face { edge: EdgeID(9) });

        mesh.into()
    }

    pub fn new_simple() -> TSpline<f64> {
        let mut mesh = TMesh {
            vertices: Vec::with_capacity(8),
            edges: Vec::with_capacity(20),
            faces: Vec::with_capacity(3),
        };

        // 1. Vertices
        // Mapped from createTPoints in simple.txt
        let coords = [
            (0.0, 0.0, 0.0, 0.0, 0.0, 1.0),  // 0: v0 (0,0) -> p0-2
            (0.5, 0.0, 2.0, 0.0, 0.5, 1.0),  // 1: v1 (0.5,0) -> p1-1
            (1.0, 0.0, 4.0, 0.0, 0.0, 1.0),  // 2: v2 (1,0) -> p2-3
            (0.0, 0.5, 0.0, 2.0, 0.5, 1.0),  // 3: v3 (0,0.5) -> p3-0
            (0.5, 0.5, 2.0, 2.0, -1.0, 1.0), // 4: v4 (0.5,0.5) -> p4-0 (T-Junction)
            (1.0, 0.5, 4.0, 2.0, 0.5, 1.0),  // 5: v5 (1,0.5) -> p5-1
            (0.0, 1.0, 0.0, 4.0, 0.0, 1.0),  // 6: v6 (0,1) -> p6-0
            (1.0, 1.0, 4.0, 4.0, 0.0, 1.0),  // 7: v7 (1,1) -> p7-1
        ];

        for (i, (s, t, x, y, z, w)) in coords.iter().enumerate() {
            let is_t = i == 4;
            mesh.vertices.push(ControlPoint {
                geometry: Vector4::new(*x, *y, *z, *w),
                uv: ParamPoint { s: *s, t: *t },
                outgoing_edge: None,
                is_t_junction: is_t,
            });
        }

        // Helper to add edge
        let mut add_edge = |origin: usize,
                            next: usize,
                            prev: usize,
                            twin: Option<usize>,
                            face: Option<usize>,
                            dir: Direction,
                            interval: f64| {
            let id = mesh.edges.len();
            mesh.edges.push(HalfEdge {
                origin: VertID(origin),
                next: EdgeID(next),
                prev: EdgeID(prev),
                twin: twin.map(EdgeID),
                face: face.map(FaceID),
                knot_interval: interval,
                direction: dir,
            });
            if mesh.vertices[origin].outgoing_edge.is_none() {
                mesh.vertices[origin].outgoing_edge = Some(EdgeID(id));
            }
        };

        // Face 0 (v0->v1->v4->v3)
        // Indices: 0, 1, 2, 3
        add_edge(0, 1, 3, Some(13), Some(0), Direction::S, 0.5); // 0: v0->v1
        add_edge(1, 2, 0, Some(7), Some(0), Direction::T, 0.5); // 1: v1->v4
        add_edge(4, 3, 1, Some(8), Some(0), Direction::S, 0.5); // 2: v4->v3
        add_edge(3, 0, 2, Some(14), Some(0), Direction::T, 0.5); // 3: v3->v0

        // Face 1 (v1->v2->v5->v4)
        // Indices: 4, 5, 6, 7
        add_edge(1, 5, 7, Some(19), Some(1), Direction::S, 0.5); // 4: v1->v2
        add_edge(2, 6, 4, Some(18), Some(1), Direction::T, 0.5); // 5: v2->v5
        add_edge(5, 7, 5, Some(9), Some(1), Direction::S, 0.5); // 6: v5->v4
        add_edge(4, 4, 6, Some(1), Some(1), Direction::T, 0.5); // 7: v4->v1

        // Face 2 (v3->v4->v5->v7->v6)
        // Indices: 8, 9, 10, 11, 12
        add_edge(3, 9, 12, Some(2), Some(2), Direction::S, 0.5); // 8: v3->v4
        add_edge(4, 10, 8, Some(6), Some(2), Direction::S, 0.5); // 9: v4->v5
        add_edge(5, 11, 9, Some(17), Some(2), Direction::T, 0.5); // 10: v5->v7
        add_edge(7, 12, 10, Some(16), Some(2), Direction::S, 1.0); // 11: v7->v6
        add_edge(6, 8, 11, Some(15), Some(2), Direction::T, 0.5); // 12: v6->v3

        // Boundary (Clockwise Loop)
        add_edge(1, 14, 19, Some(0), None, Direction::S, 0.5); // 13: v1->v0
        add_edge(0, 15, 13, Some(3), None, Direction::T, 0.5); // 14: v0->v3
        add_edge(3, 16, 14, Some(12), None, Direction::T, 0.5); // 15: v3->v6
        add_edge(6, 17, 15, Some(11), None, Direction::S, 1.0); // 16: v6->v7
        add_edge(7, 18, 16, Some(10), None, Direction::T, 0.5); // 17: v7->v5
        add_edge(5, 19, 17, Some(5), None, Direction::T, 0.5); // 18: v5->v2
        add_edge(2, 13, 18, Some(4), None, Direction::S, 0.5); // 19: v2->v1

        // Faces
        mesh.faces.push(Face { edge: EdgeID(0) });
        mesh.faces.push(Face { edge: EdgeID(4) });
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

        assert!(mesh.validate_asts(), "Mesh should be Analysis-Suitable (no T-junctions)");
    }
}


