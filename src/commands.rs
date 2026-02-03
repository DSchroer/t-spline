mod tessellate;

pub use tessellate::*;

use crate::TSpline;
use crate::tmesh::TMesh;

/// Edit operation to perform on a spline mesh
pub trait Command {
    /// Output of the operation
    type Result;

    /// Perform the operation, returning any error as needed
    fn execute(&mut self, mesh: &TMesh) -> Self::Result;

    fn apply(&mut self, spline: &TSpline) -> Self::Result {
        spline.apply(self)
    }
}

/// Edit operation to perform on a spline mesh
pub trait CommandMut {
    /// Error type of the operation
    type Result;

    /// Perform the operation, returning any error as needed
    fn execute(&mut self, mesh: &mut TMesh) -> Self::Result;

    fn apply(&mut self, spline: &mut TSpline) -> Self::Result {
        spline.apply_mut(self)
    }
}

impl<T, Out> Command for T
where
    T: FnMut(&TMesh) -> Out,
{
    type Result = Out;

    fn execute(&mut self, mesh: &TMesh) -> Self::Result {
        self(mesh)
    }
}

impl<T, Out> CommandMut for T
where
    T: FnMut(&mut TMesh) -> Out,
{
    type Result = Out;

    fn execute(&mut self, mesh: &mut TMesh) -> Out {
        self(mesh)
    }
}
