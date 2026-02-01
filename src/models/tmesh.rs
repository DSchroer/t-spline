use crate::models::*;

#[derive(Debug, Clone, Default)]
pub struct TMesh {
    pub vertices: Vec<ControlPoint>,
    pub edges: Vec<HalfEdge>,
    pub faces: Vec<Face>,
}

impl TMesh {
    pub fn vertex(&self, id: VertID) -> &ControlPoint { &self.vertices[id.0] }

    pub fn edge(&self, id: EdgeID) -> &HalfEdge { &self.edges[id.0] }

    pub fn face(&self, id: FaceID) -> &Face { &self.faces[id.0] }

    pub fn face_edges(&self, face_id: FaceID) -> Vec<EdgeID> {
        let start_edge = self.faces[face_id.0].edge;
        let mut edges = Vec::new();
        let mut curr = start_edge;

        loop {
            edges.push(curr);
            curr = self.edge(curr).next;
            if curr == start_edge { break; }
        }
        edges
    }

    pub fn find_edge(&self, v_start: VertID, v_end: VertID) -> Option<EdgeID> {
        let start_v = self.vertex(v_start);
        let first_edge = start_v.outgoing_edge?;

        // Circulate around vertex (using twin -> next logic)
        let mut curr = first_edge;
        loop {
            let edge = self.edge(curr);
            // The destination of an edge is the origin of its twin
            // Or we can look at the origin of the 'next' edge if the face is valid
            // Standard approach: dest(e) = origin(twin(e))
            if let Some(twin_id) = edge.twin {
                let twin = self.edge(twin_id);
                if twin.origin == v_end {
                    return Some(curr);
                }
                // Move to next spoke: twin -> next
                curr = twin.next;
            } else {
                // Boundary case handling needed here
                break;
            }

            if curr == first_edge { break; }
        }
        None
    }

    /// Infers the local knot vectors for a specific control point.
    /// Returns (s_vector, t_vector).
    pub fn infer_local_knots(&self, v_id: VertID) -> ([f64; 5], [f64; 5]) {
        let v = &self.vertices[v_id.0];
        let (s2, t2) = (v.uv.s, v.uv.t);

        // Trace two knots in each of the four cardinal directions
        let s_pos = self.trace_knots(v_id, Direction::S, true);  // s3, s4
        let s_neg = self.trace_knots(v_id, Direction::S, false); // s1, s0
        let t_pos = self.trace_knots(v_id, Direction::T, true);  // t3, t4
        let t_neg = self.trace_knots(v_id, Direction::T, false); // t1, t0

        (
            [s_neg[1], s_neg[1], s2, s_pos[1], s_pos[1]],
            [t_neg[1], t_neg[1], t2, t_pos[1], t_pos[1]],
        )
    }

    /// Traces a ray from start_v in a direction to find the next two orthogonal knots.
    fn trace_knots(&self, start_v: VertID, axis: Direction, positive: bool) -> [f64; 2] {
        let mut results = [0.0; 2];
        let mut found_count = 0;
        let mut current_v = start_v;

        while found_count < 2 {
            // Find an edge starting at current_v that aligns with our ray axis
            if let Some(next_v) = self.find_next_vertex_in_direction(current_v, axis, positive) {
                current_v = next_v;

                // Rule 1: A knot is defined by the intersection with an 'orthogonal edge'.
                // In a T-mesh, every vertex (including T-junctions) on the ray path
                // provides a knot for the orthogonal dimension.[2]
                results[found_count] = match axis {
                    Direction::S => self.vertices[current_v.0].uv.s,
                    Direction::T => self.vertices[current_v.0].uv.t,
                };
                found_count += 1;
            } else {
                // Boundary Condition: If the ray hits the edge of the T-mesh,
                // repeat the last found knot to simulate an open knot vector.
                let last_knot = if found_count > 0 {
                    results[found_count - 1]
                } else {
                    match axis {
                        Direction::S => self.vertices[current_v.0].uv.s,
                        Direction::T => self.vertices[current_v.0].uv.t,
                    }
                };

                while found_count < 2 {
                    results[found_count] = last_knot;
                    found_count += 1;
                }
            }
        }
        results
    }

    /// Helper to find the next vertex along the mesh edges in a specific direction.
    fn find_next_vertex_in_direction(&self, v_id: VertID, axis: Direction, positive: bool) -> Option<VertID> {
        let v = &self.vertices[v_id.0];
        let start_edge = v.outgoing_edge?;
        let mut curr_e_id = start_edge;

        loop {
            let edge = &self.edges[curr_e_id.0];
            let dest_id = self.edges[edge.twin?.0].origin;
            let dest_v = &self.vertices[dest_id.0];

            // Calculate parametric delta
            let delta = match axis {
                Direction::S => dest_v.uv.s - v.uv.s,
                Direction::T => dest_v.uv.t - v.uv.t,
            };

            // Check if the edge is collinear with the search axis
            let is_collinear = match axis {
                Direction::S => (dest_v.uv.t - v.uv.t).abs() < 1e-12,
                Direction::T => (dest_v.uv.s - v.uv.s).abs() < 1e-12,
            };

            if is_collinear && ((positive && delta > 1e-12) || (!positive && delta < -1e-12)) {
                return Some(dest_id);
            }

            // Circulate to the next outgoing edge at this vertex
            curr_e_id = self.edges[edge.prev.0].twin?;
            if curr_e_id == start_edge { break; }
        }
        None
    }

    /// Casts a ray in `direction` for `steps` topological units.
    /// Returns the coordinate found.
    fn cast_ray_for_knot(&self, start_v: VertID, dir: Direction, steps: i32) -> f64 {
        let mut curr_v = start_v;
        let is_forward = steps > 0;
        let count = steps.abs();

        for _ in 0..count {
            match self.find_next_orthogonal_edge(curr_v, dir, is_forward) {
                Some(next_v) => {
                    curr_v = next_v;
                }
                None => {
                    // Boundary reached.
                    // Standard T-Spline rule: extend the last interval or repeat knot.
                    // Here we simply return the boundary coordinate.
                    // A more robust impl would repeat the boundary knot if steps remain.
                    return match dir {
                        Direction::S => self.vertex(curr_v).uv.s,
                        Direction::T => self.vertex(curr_v).uv.t,
                    };
                }
            }
        }

        match dir {
            Direction::S => self.vertex(curr_v).uv.s,
            Direction::T => self.vertex(curr_v).uv.t,
        }
    }

    /// Finds the next vertex connected by an edge in the given direction.
    /// This abstracts the topology navigation.
    fn find_next_orthogonal_edge(&self, v: VertID, dir: Direction, forward: bool) -> Option<VertID> {
        let start_edge = self.vertex(v).outgoing_edge?;
        let mut curr = start_edge;

        // Circulate to find edge aligned with direction
        loop {
            let edge = self.edge(curr);

            // Check alignment. This assumes edges store their parametric direction.
            // In a real implementation, we might check the geometry of the UVs.
            let is_aligned = edge.direction == dir;

            // "Forward" in S means increasing S.
            // We need to check geometry or explicit flags.
            // Simplified logic: assume edges are directed.
            let geometry_delta = if let Some(twin) = edge.twin {
                let dest = self.edge(twin).origin;
                let uv_dest = self.vertex(dest).uv;
                let uv_src = self.vertex(edge.origin).uv;
                match dir {
                    Direction::S => uv_dest.s - uv_src.s,
                    Direction::T => uv_dest.t - uv_src.t,
                }
            } else { 0.0 };

            if is_aligned {
                if (forward && geometry_delta > 0.0)
                    || (!forward && geometry_delta < 0.0) {
                    return edge.twin.map(|id| self.edge(id).origin);
                }
            }

            // Move to next spoke
            if let Some(twin) = edge.twin {
                curr = self.edge(twin).next;
            } else {
                break;
            }
            if curr == start_edge { break; }
        }
        None
    }

    /// Validates if the current mesh satisfies ASTS conditions.
    pub fn validate_asts(&self) -> bool {
        let h_exts = self.collect_extensions(Direction::S);
        let v_exts = self.collect_extensions(Direction::T);

        for h in &h_exts {
            for v in &v_exts {
                if h.intersects(v) {
                    return false; // Intersection detected!
                }
            }
        }
        true
    }

    fn collect_extensions(&self, dir: Direction) -> Vec<Segment> {
        let mut extensions = Vec::new();

        for (idx, vert) in self.vertices.iter().enumerate() {
            if!vert.is_t_junction { continue; }

            // Check if T-junction points in 'dir'
            // A T-junction "points" into the face it is missing an edge for.
            // We need logic to determine orientation of the T.

            if self.t_junction_orientation(VertID(idx)) == dir {
                // Trace ray until it hits a perpendicular full edge
                let start_uv = vert.uv;
                let end_val = self.cast_ray_for_knot(VertID(idx), dir, 2); // heuristic distance
                // Real ASTS tracing must go until it hits a line in the T-mesh
                // that is perpendicular to the extension.

                let end_uv = match dir {
                    Direction::S => ParamPoint { s: end_val, t: start_uv.t },
                    Direction::T => ParamPoint { s: start_uv.s, t: end_val },
                };
                extensions.push(Segment { start: start_uv, end: end_uv });
            }
        }
        extensions
    }

    fn t_junction_orientation(&self, v: VertID) -> Direction {
        // Logic to inspect neighbors and determine if T points up/down (T) or left/right (S)
        Direction::S // Stub
    }
}
