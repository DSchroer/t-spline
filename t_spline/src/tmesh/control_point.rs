use crate::tmesh::ids::EdgeID;
use crate::tmesh::segment::ParamPoint;
use cgmath::Vector4;

#[derive(Debug, Clone)]
pub struct ControlPoint<T> {
    /// Homogeneous coordinates (x, y, z, w) for rational surfaces
    pub geometry: Vector4<T>,
    /// The parametric location (knot value) of this point
    pub uv: ParamPoint<T>,
    /// Index of one half-edge starting at this vertex
    pub outgoing_edge: Option<EdgeID>,
    /// ASTS Metadata: Is this a T-junction?
    pub is_t_junction: bool,
}
