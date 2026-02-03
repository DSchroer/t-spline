use cgmath::Vector4;

pub mod commands;
pub mod export;
mod shapes;
pub mod tmesh;

use crate::commands::{Command, CommandMut};
use crate::tmesh::TMesh;

#[derive(Debug, Default, Clone)]
pub struct TSpline {
    mesh: TMesh,
}

impl From<TMesh> for TSpline {
    fn from(value: TMesh) -> Self {
        TSpline::new(value)
    }
}

impl From<TSpline> for TMesh {
    fn from(value: TSpline) -> Self {
        value.into_mesh()
    }
}

impl TSpline {
    pub fn new(mesh: TMesh) -> Self {
        Self { mesh }
    }

    /// Perform an operation on the underlying TMesh
    ///
    /// Simple modifications can be done using clojures:
    /// ```
    /// # use t_spline::*;
    /// # use t_spline::tmesh::TMesh;
    /// let mut spline = TSpline::new_unit_square();
    /// spline.apply_mut(&mut |m: &mut TMesh| m.vertices[0].geometry.z = 1.0);
    /// ```
    ///
    /// Dynamic dispatch modifications are also possible:
    /// ```
    /// # use t_spline::commands::*;
    /// # use t_spline::tmesh::*;
    /// # use t_spline::*;
    /// let mut spline = TSpline::new_unit_square();
    /// let mut dynOp: Box<dyn CommandMut<Result=()>> = Box::new(|m: &mut TMesh|{});
    /// spline.apply_mut(dynOp.as_mut());
    /// ```
    ///
    /// Complex operations should implement [Command].
    pub fn apply_mut<T: CommandMut + ?Sized>(&mut self, op: &mut T) -> T::Result {
        op.execute(&mut self.mesh)
    }

    /// Perform an operation on the underlying TMesh
    ///
    /// Simple modifications can be done using clojures:
    /// ```
    /// # use t_spline::tmesh::*;
    /// # use t_spline::*;
    /// let mut spline = TSpline::new_unit_square();
    /// spline.apply(&mut |m: &TMesh| {});
    /// ```
    ///
    /// Dynamic dispatch modifications are also possible:
    /// ```
    /// # use t_spline::commands::*;
    /// # use t_spline::tmesh::*;
    /// # use t_spline::*;
    /// let mut spline = TSpline::new_unit_square();
    /// let mut dynOp: Box<dyn Command<Result=()>> = Box::new(|m: &TMesh|{});
    /// spline.apply(dynOp.as_mut());
    /// ```
    ///
    /// Complex operations should implement [Command].
    pub fn apply<T: Command + ?Sized>(&self, op: &mut T) -> T::Result {
        op.execute(&self.mesh)
    }

    pub fn mesh(&self) -> &TMesh {
        &self.mesh
    }

    pub fn into_mesh(self) -> TMesh {
        self.mesh
    }
}
