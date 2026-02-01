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

        TSpline::new(mesh)
    }
}
