use crate::*;
use crate::models::*;

impl TSpline {
    pub fn new_unit_square() -> TSpline {
        let mut mesh = TMesh {
            vertices: Vec::with_capacity(4),
            edges: Vec::with_capacity(8),
            faces: Vec::with_capacity(1),
        };

        // 1. Define 4 Corner Vertices
        let coords = [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)];
        for (i, (s, t)) in coords.iter().enumerate() {
            mesh.vertices.push(ControlPoint {
                geometry: Vector4::new(*s, *t, 0.0, 1.0),
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
                direction: if i % 2 == 0 { Direction::S } else { Direction::T },
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
                direction: if i % 2 == 0 { Direction::S } else { Direction::T },
            });
        }

        // 4. Define the Face
        mesh.faces.push(Face {
            edge: EdgeID(0),
        });

        mesh.into()
    }

    /// Creates a simple T-Spline mesh with a T-junction, which is impossible
    /// to represent as a single NURBS patch.
    /// Topology:
    /// F0 (Left): (0,0)-(1,0)-(1,1)-(1,2)-(0,2) [Pentagon with T-junction at (1,1)]
    /// F1 (Bot-Right): (1,0)-(2,0)-(2,1)-(1,1)
    /// F2 (Top-Right): (1,1)-(2,1)-(2,2)-(1,2)
    pub fn new_t_junction() -> TSpline {
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
        let mut add_edge = |origin: usize, next: usize, prev: usize, twin: Option<usize>, face: Option<usize>, dir: Direction| {
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
        add_edge(1, 2, 0, Some(8),  Some(0), Direction::T); // 1
        add_edge(3, 3, 1, Some(12), Some(0), Direction::T); // 2
        add_edge(6, 4, 2, Some(18), Some(0), Direction::S); // 3 (Actually -S)
        add_edge(5, 0, 3, Some(19), Some(0), Direction::T); // 4 (Actually -T)

        // F1 (1->2->4->3->1)
        add_edge(1, 6, 8, Some(14), Some(1), Direction::S); // 5
        add_edge(2, 7, 5, Some(15), Some(1), Direction::T); // 6
        add_edge(4, 8, 6, Some(9),  Some(1), Direction::S); // 7 (Actually -S)
        add_edge(3, 5, 7, Some(1),  Some(1), Direction::T); // 8 (Actually -T)

        // F2 (3->4->7->6->3)
        add_edge(3, 10, 12, Some(7),  Some(2), Direction::S); // 9
        add_edge(4, 11, 9,  Some(16), Some(2), Direction::T); // 10
        add_edge(7, 12, 10, Some(17), Some(2), Direction::S); // 11 (Actually -S)
        add_edge(6, 9,  11, Some(2),  Some(2), Direction::T); // 12 (Actually -T)

        // Boundary Half-Edges
        // Loop: 13->19->18->17->16->15->14->13
        add_edge(1, 19, 14, Some(0),  None, Direction::S); // 13 (Twin of 0)
        add_edge(2, 13, 15, Some(5),  None, Direction::S); // 14 (Twin of 5)
        add_edge(4, 14, 16, Some(6),  None, Direction::T); // 15 (Twin of 6)
        add_edge(7, 15, 17, Some(10), None, Direction::T); // 16 (Twin of 10)
        add_edge(6, 16, 18, Some(11), None, Direction::S); // 17 (Twin of 11)
        add_edge(5, 17, 19, Some(3),  None, Direction::S); // 18 (Twin of 3)
        add_edge(0, 18, 13, Some(4),  None, Direction::T); // 19 (Twin of 4)

        // Faces
        mesh.faces.push(Face { edge: EdgeID(0) });
        mesh.faces.push(Face { edge: EdgeID(5) });
        mesh.faces.push(Face { edge: EdgeID(9) });

        mesh.into()
    }
}
