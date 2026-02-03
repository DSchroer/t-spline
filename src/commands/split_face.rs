use crate::commands::CommandMut;
use crate::tmesh::control_point::ControlPoint;
use crate::tmesh::direction::Direction;
use crate::tmesh::face::Face;
use crate::tmesh::half_edge::HalfEdge;
use crate::tmesh::ids::{EdgeID, FaceID, VertID};
use crate::tmesh::segment::ParamPoint;
use crate::tmesh::TMesh;

pub struct SplitFace {
    pub face: FaceID,
    pub direction: Direction,
}

impl CommandMut for SplitFace {
    type Result = ();

    fn execute(&mut self, mesh: &mut TMesh) -> Self::Result {
        // 1. Calculate Face Bounds and Split Parameter
        let edges = mesh.face_edges(self.face);
        if edges.is_empty() {
            return;
        }

        let mut s_min = f64::MAX;
        let mut s_max = f64::MIN;
        let mut t_min = f64::MAX;
        let mut t_max = f64::MIN;

        for &e_id in &edges {
            let v = mesh.vertex(mesh.edge(e_id).origin);
            s_min = s_min.min(v.uv.s);
            s_max = s_max.max(v.uv.s);
            t_min = t_min.min(v.uv.t);
            t_max = t_max.max(v.uv.t);
        }

        let split_val = match self.direction {
            Direction::S => (t_min + t_max) / 2.0, // Horizontal split (constant T)
            Direction::T => (s_min + s_max) / 2.0, // Vertical split (constant S)
        };

        // 2. Find intersecting edges
        // We look for edges that span across the split_val in the orthogonal direction.
        let mut split_candidates = Vec::new();

        for &e_id in &edges {
            let edge = mesh.edge(e_id);
            let v_start = mesh.vertex(edge.origin);
            // Robustly find end vertex
            let next_edge = mesh.edge(edge.next);
            let v_end = mesh.vertex(next_edge.origin);

            let (start_val, end_val) = match self.direction {
                Direction::S => (v_start.uv.t, v_end.uv.t),
                Direction::T => (v_start.uv.s, v_end.uv.s),
            };

            // Check intersection (strict, excluding endpoints to avoid corners)
            let min_val = start_val.min(end_val);
            let max_val = start_val.max(end_val);

            if split_val > min_val + 1e-6 && split_val < max_val - 1e-6 {
                split_candidates.push(e_id);
            }
        }

        if split_candidates.len() != 2 {
            // Cannot split: strange topology or degenerate face
            return;
        }

        let e1_id = split_candidates[0];
        let e2_id = split_candidates[1];

        // 3. Split the boundary edges
        let v1 = split_edge(mesh, e1_id, split_val, self.direction);
        let v2 = split_edge(mesh, e2_id, split_val, self.direction);

        // 4. Connect v1 and v2 with a new edge (and twin)
        // We need to identify which new edges at v1 and v2 belong to the current face loop
        // When split_edge was called, it inserted the new vertex into the edge E.
        // E becomes (Origin -> V). E_new becomes (V -> OldDest).
        // Both are still part of 'face'.
        // So the loop is now: ... -> E1 -> E1_new -> ... -> E2 -> E2_new -> ...

        // We want to create a cross edge.
        // Let's verify the order in the face loop.
        // Since we split E1, we have E1 ending at v1, and E1_new starting at v1.
        // Since we split E2, we have E2 ending at v2, and E2_new starting at v2.
        
        // We need to determine if the cut goes v1->v2 or v2->v1 based on the face winding (CCW).
        // But since we are creating a twin pair, we create both.
        // The question is how to wire them.
        
        // Loop 1 (New Face?): E1 -> CrossEdge -> E2_new ...
        // Loop 2 (Old Face?): E2 -> CrossEdgeTwin -> E1_new ...
        
        // We need to check if E2_new follows E1 in the loop or precedes it.
        // Face edges: E1, E1_new, ..., E2, E2_new, ...
        // So Loop 1: E1 (ends v1) -> (v1->v2) -> E2_new (starts v2) ... -> E1.
        // This skips E1_new...E2.
        // Loop 2: E2 (ends v2) -> (v2->v1) -> E1_new (starts v1) ... -> E2.
        // This includes E1_new...E2.
        
        // Wait, does E2_new follow E1? 
        // Iterate to check order.
        // E1 is at pos1. E1_new should be effectively at pos1 + 1 (conceptually, though IDs changed).
        // Actually, split_edge modifies the linked list, so `face_edges` needs refresh or careful logic.
        // split_edge keeps E1 as the first part, and sets E1.next = E1_new.
        
        // So traversing from E1: E1 -> E1_new -> ... -> E2 -> E2_new ...
        // Assuming E2 comes after E1.
        // What if E1 comes after E2? (Wrap around).
        // Let's trace from E1. If we hit E2, then order is E1...E2.
        // If we hit E2 before E1 (impossible in simple loop), ok.
        
        let mut cursor = mesh.edge(e1_id).next; // This is E1_new
        let mut e2_is_downstream = false;
        loop {
            if cursor == e2_id {
                e2_is_downstream = true;
                break;
            }
            if cursor == e1_id {
                break;
            }
            cursor = mesh.edge(cursor).next;
        }

        // If E2 is downstream from E1:
        // E1 -> E1_new -> ... -> E2 -> E2_new -> ... -> E1
        // We link v1 (end of E1) to v2 (start of E2_new).
        // CrossEdge A: v1 -> v2.
        // CrossEdge B: v2 -> v1.
        
        // Loop 1: E1 -> A -> E2_new ... (Short cut)
        // Loop 2: E2 -> B -> E1_new ... (The other part)
        
        let (v_start, v_end, e_in_1, e_out_1, e_in_2, e_out_2) = if e2_is_downstream {
            // v1 -> v2
             // E1 ends at v1. E2_new starts at v2.
             // E2 ends at v2. E1_new starts at v1.
             (v1, v2, e1_id, mesh.edge(e2_id).next, e2_id, mesh.edge(e1_id).next)
        } else {
            // v2 -> v1
            // E2 -> E2_new -> ... -> E1 -> E1_new
            // Link v2 to v1.
            (v2, v1, e2_id, mesh.edge(e1_id).next, e1_id, mesh.edge(e2_id).next)
        };

        // Create Cross Edges
        let cross_id = EdgeID(mesh.edges.len());
        let cross_twin_id = EdgeID(mesh.edges.len() + 1);
        let new_face_id = FaceID(mesh.faces.len());
        
        // Length of cross edge
        let len = match self.direction {
            Direction::S => (mesh.vertex(v_end).uv.s - mesh.vertex(v_start).uv.s).abs(),
            Direction::T => (mesh.vertex(v_end).uv.t - mesh.vertex(v_start).uv.t).abs(),
        };

        let cross_edge = HalfEdge {
            origin: v_start,
            twin: Some(cross_twin_id),
            face: Some(self.face), // Loop 1 keeps old face
            next: e_out_1,
            prev: e_in_1,
            knot_interval: len,
            direction: self.direction,
        };

        let cross_twin = HalfEdge {
            origin: v_end,
            twin: Some(cross_id),
            face: Some(new_face_id), // Loop 2 gets new face
            next: e_out_2,
            prev: e_in_2,
            knot_interval: len,
            direction: self.direction,
        };

        mesh.edges.push(cross_edge);
        mesh.edges.push(cross_twin);
        mesh.faces.push(Face { edge: cross_twin_id });

        // Update links for Loop 1 (old face)
        // E_in_1 (E1) -> Cross (A) -> E_out_1 (E2_new)
        let e_in_1_idx = e_in_1.0;
        mesh.edges[e_in_1_idx].next = cross_id;
        let e_out_1_idx = e_out_1.0;
        mesh.edges[e_out_1_idx].prev = cross_id;
        
        // Ensure face pointer is correct for old face (it might have pointed to an edge that moved to new face)
        mesh.faces[self.face.0].edge = cross_id;

        // Update links for Loop 2 (new face)
        // E_in_2 (E2) -> CrossTwin (B) -> E_out_2 (E1_new)
        let e_in_2_idx = e_in_2.0;
        mesh.edges[e_in_2_idx].next = cross_twin_id;
        let e_out_2_idx = e_out_2.0;
        mesh.edges[e_out_2_idx].prev = cross_twin_id;
        
        // Walk Loop 2 and update face pointers
        let mut curr = cross_twin_id;
        loop {
            mesh.edges[curr.0].face = Some(new_face_id);
            curr = mesh.edges[curr.0].next;
            if curr == cross_twin_id { break; }
        }
    }
}

fn split_edge(mesh: &mut TMesh, edge_id: EdgeID, split_val: f64, split_dir: Direction) -> VertID {
    // 1. Get info
    let (origin, dest, twin_id, face_id, old_next, _old_prev, dir, interval) = {
        let e = mesh.edge(edge_id);
        let dest = if let Some(tw) = e.twin {
            mesh.edge(tw).origin
        } else {
            mesh.edge(e.next).origin // Fallback for boundary loop
        };
        (e.origin, dest, e.twin, e.face, e.next, e.prev, e.direction, e.knot_interval)
    };

    // Calculate new vertex geometry and UV
    let v_origin = mesh.vertex(origin);
    let v_dest = mesh.vertex(dest);
    
    // Interpolation factor t
    // If splitting S-edge (Dir S), split_dir is T (constant S).
    // Wait, passed split_val is the coordinate.
    let (c1, c2) = match split_dir {
        Direction::S => (v_origin.uv.t, v_dest.uv.t), // Split S -> Horizontal cut -> Edge is Vertical?
        // No. If Direction::S passed to SplitFace, we are making a Horizontal cut (S-edge).
        // The intersected edges are Vertical (T-edges).
        // So split_dir is S. The edges we split are T.
        // So we interpolate T.
        Direction::T => (v_origin.uv.s, v_dest.uv.s),
    };
    
    // Wait, logic check:
    // SplitFace(S) -> Horizontal cut. New edge is S.
    // Intersected edges are T-edges.
    // split_val is a T-coordinate.
    // So we interpolate based on T.
    // If edge direction is T, we interpolate T.
    // If edge direction is S, we interpolate S.
    // But we found this edge because it crosses split_val.
    // So we use split_val to find alpha.
    
    let alpha = (split_val - c1) / (c2 - c1);
    
    let geom = v_origin.geometry + (v_dest.geometry - v_origin.geometry) * alpha;
    let uv = ParamPoint {
        s: v_origin.uv.s + (v_dest.uv.s - v_origin.uv.s) * alpha,
        t: v_origin.uv.t + (v_dest.uv.t - v_origin.uv.t) * alpha,
    };
    
    let new_vert_id = VertID(mesh.vertices.len());
    
    // New Vertex
    mesh.vertices.push(ControlPoint {
        geometry: geom,
        uv,
        outgoing_edge: None, // Will set below
        is_t_junction: true, // Initially a T-junction until fully connected?
                             // Strictly, it's a T-junction if it has 3 edges.
                             // After split_edge, it has 2 (collinear).
                             // We will add the 3rd one later in execute().
    });

    // Create New Edges
    // Old Edge E: Origin -> Dest
    // Becomes: E (Origin -> New) -> E_new (New -> Dest)
    let e_new_id = EdgeID(mesh.edges.len());
    
    // Update E
    {
        let e = &mut mesh.edges[edge_id.0];
        e.next = e_new_id;
        e.knot_interval *= alpha; // Approximate interval split
        // Twin? If E has twin T, T must also be split.
        // T (Dest -> Origin) becomes T (Dest -> New)? No.
        // T starts at Dest. So T (Dest -> New) -> T_new (New -> Origin).
        // Then E.twin = T_new. E_new.twin = T.
    }
    
    // Create E_new
    mesh.edges.push(HalfEdge {
        origin: new_vert_id,
        twin: None, // Set later
        face: face_id,
        next: old_next,
        prev: edge_id,
        knot_interval: interval * (1.0 - alpha),
        direction: dir,
    });
    // Update old_next.prev
    let next_edge_idx = old_next.0;
    mesh.edges[next_edge_idx].prev = e_new_id;

    // Set vertex outgoing
    mesh.vertices[new_vert_id.0].outgoing_edge = Some(e_new_id);

    // Handle Twin
    if let Some(t_id) = twin_id {
        let (_t_origin, t_face, t_next, _t_prev, t_dir, t_interval) = {
            let t = mesh.edge(t_id);
            (t.origin, t.face, t.next, t.prev, t.direction, t.knot_interval)
        };
        
        // T (Dest -> Origin) split at New.
        // T becomes (Dest -> New). T_new becomes (New -> Origin).
        // Wait, T starts at Dest (which is Origin of Twin).
        // So T goes Dest -> Origin.
        // New vertex is between Dest and Origin.
        // So T -> T_new.
        // T connects Dest -> New.
        // T_new connects New -> Origin.
        
        let t_new_id = EdgeID(mesh.edges.len());
        
        // Update T
        {
            let t = &mut mesh.edges[t_id.0];
            t.next = t_new_id;
            t.knot_interval = t_interval * (1.0 - alpha); // T runs opposite? Interval logic same.
        }
        
        // Create T_new
        mesh.edges.push(HalfEdge {
            origin: new_vert_id,
            twin: Some(edge_id), // T_new matches E
            face: t_face,
            next: t_next,
            prev: t_id,
            knot_interval: t_interval * alpha,
            direction: t_dir,
        });
        
        // Update T_next.prev
        let t_next_idx = t_next.0;
        mesh.edges[t_next_idx].prev = t_new_id;
        
        // Link Twins
        // E.twin = T_new
        mesh.edges[edge_id.0].twin = Some(t_new_id);
        // E_new.twin = T
        mesh.edges[e_new_id.0].twin = Some(t_id);
        mesh.edges[t_id.0].twin = Some(e_new_id);
        
        // Note: Vertices outgoing edge check
        // Dest outgoing edge might have been T. Still is T (Dest->New). OK.
        // Origin outgoing edge might have been E. Still is E (Origin->New). OK.
    } else {
        // Boundary edge case (no twin)
        // E_new is boundary.
        // No T to split.
    }
    
    new_vert_id
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TSpline;

    #[test]
    fn it_splits_face_horizontally() {
        let mut spline = TSpline::new_unit_square();
        // Face 0 is unit square.
        // Split S (Horizontal). Should cut vertical edges (1 and 3).
        let cmd = SplitFace {
            face: FaceID(0),
            direction: Direction::S,
        };
        spline.apply_mut(&mut { cmd });
        
        let mesh = spline.mesh();
        assert_eq!(mesh.faces.len(), 2);
        assert_eq!(mesh.vertices.len(), 6); // 4 + 2 new
        
        // Check new vertices are at t=0.5
        let v4 = &mesh.vertices[4];
        let v5 = &mesh.vertices[5];
        assert!((v4.uv.t - 0.5).abs() < 1e-6);
        assert!((v5.uv.t - 0.5).abs() < 1e-6);
    }
    
    #[test]
    fn it_splits_face_vertically() {
        let mut spline = TSpline::new_unit_square();
        // Face 0 is unit square.
        // Split T (Vertical). Should cut horizontal edges (0 and 2).
        let cmd = SplitFace {
            face: FaceID(0),
            direction: Direction::T,
        };
        spline.apply_mut(&mut { cmd });
        
        let mesh = spline.mesh();
        assert_eq!(mesh.faces.len(), 2);
        assert_eq!(mesh.vertices.len(), 6);
        
        // Check new vertices are at s=0.5
        let v4 = &mesh.vertices[4];
        let v5 = &mesh.vertices[5];
        assert!((v4.uv.s - 0.5).abs() < 1e-6);
        assert!((v5.uv.s - 0.5).abs() < 1e-6);
    }
}