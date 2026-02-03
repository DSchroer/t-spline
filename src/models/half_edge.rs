use crate::models::direction::Direction;
use crate::models::ids::{EdgeID, FaceID, VertID};

#[derive(Debug, Clone)]
pub struct HalfEdge {
    pub origin: VertID,       // Vertex where this edge starts
    pub twin: Option<EdgeID>, // The opposite half-edge
    pub face: Option<FaceID>, // The face to the left of this edge
    pub next: EdgeID,         // Next half-edge in the face loop
    pub prev: EdgeID,         // Previous half-edge in the face loop

    /// T-Spline specific: Knot Interval associated with this edge
    /// If the edge runs in S direction, this is a delta-s.
    pub knot_interval: f64,
    pub direction: Direction,
}
