use crate::*;
use crate::models::*;

impl TSpline {
    pub fn new_square() -> TSpline {
        let mut mesh = TMesh {
            vertices: Vec::new(),
            edges: Vec::new(),
            faces: Vec::new(),
        };

        // 1. Define 4 Control Points (Corners of the square)
        // Geometry is stored as (x, y, z, w)
        let v_coords = [
            ((0.0, 0.0, 0.0), (0.0, 0.0)), // (x, y, z, s, t)
            ((1.0, 0.0, 0.0), (1.0, 0.0)),
            ((1.0, 1.0, 0.0), (1.0, 1.0)),
            ((0.0, 1.0, 0.0), (0.0, 1.0)),
        ];

        for (i, ((x, y, z), (s, t))) in v_coords.into_iter().enumerate() {
            mesh.vertices.push(ControlPoint {
                geometry: Vector4::new(x, y, z, 1.0),
                uv: ParamPoint { s, t },
                outgoing_edge: Some(EdgeID(i)), // Tentative assignment
                is_t_junction: false,
            });
        }

        // 2. Define 4 Half-Edges to form the loop
        // For a single square, we create a counter-clockwise loop
        for i in 0..4 {
            let next_idx = (i + 1) % 4;
            let prev_idx = (i + 3) % 4;

            mesh.edges.push(HalfEdge {
                origin: VertID(i),
                twin: None, // Simplified: no neighbors for a single patch
                face: Some(FaceID(0)),
                next: EdgeID(next_idx),
                prev: EdgeID(prev_idx),
                knot_interval: 1.0,
                direction: if i % 2 == 0 { Direction::S } else { Direction::T },
            });
        }

        // 3. Define the Face
        mesh.faces.push(Face {
            edge: EdgeID(0),
        });

        TSpline::new(mesh)
    }
}
