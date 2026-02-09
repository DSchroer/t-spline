use thiserror::Error;
use t_spline::{CommandMut, Numeric, Vector4};
use t_spline::tmesh::direction::Direction;
use t_spline::tmesh::half_edge::HalfEdge;
use t_spline::tmesh::ids::{VertID};
use t_spline::tmesh::TMesh;

pub struct Extrude<T> {
    pub vert: VertID,
    pub offset: Vector4<T>,
    pub direction: Direction
}

#[derive(Error, Debug)]
pub enum ExtrudeError {
    #[error("extruded edge is not a boundary")]
    EdgeHasTwin()
}

impl<T: Numeric + 'static> CommandMut<T> for Extrude<T> {
    type Result = Result<(), ExtrudeError>;

    fn execute(&mut self, mesh: &mut TMesh<T>) -> Self::Result {
        todo!();
    }
}