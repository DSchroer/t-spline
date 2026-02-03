mod commands;
mod shapes;
pub mod tmesh;

pub use crate::commands::{Command, CommandMut};
use crate::tmesh::TMesh;
pub use cgmath::Point3;
pub use cgmath::Vector4;
use num_traits::{Bounded, FromPrimitive, Num};

pub trait Numeric: Num + Bounded + FromPrimitive + Copy + PartialOrd {}

#[derive(Debug, Default, Clone)]
pub struct TSpline<T> {
    mesh: TMesh<T>,
}

impl<T> From<TMesh<T>> for TSpline<T> {
    fn from(value: TMesh<T>) -> Self {
        TSpline::new(value)
    }
}

impl<T> From<TSpline<T>> for TMesh<T> {
    fn from(value: TSpline<T>) -> Self {
        value.into_mesh()
    }
}

impl<T> TSpline<T> {
    pub fn new(mesh: TMesh<T>) -> Self {
        Self { mesh }
    }

    /// Perform an operation on the underlying TMesh
    ///
    /// Simple modifications can be done using clojures:
    /// ```
    /// # use t_spline::*;
    /// # use t_spline::tmesh::TMesh;
    /// let mut spline = TSpline::new_unit_square();
    /// spline.apply_mut(&mut |m: &mut TMesh<f64>| m.vertices[0].geometry.z = 1.0);
    /// ```
    ///
    /// Dynamic dispatch modifications are also possible:
    /// ```
    /// # use t_spline::tmesh::*;
    /// # use t_spline::*;
    /// let mut spline = TSpline::new_unit_square();
    /// let mut dynOp: Box<dyn CommandMut<f64, Result=()>> = Box::new(|m: &mut TMesh<f64>|{});
    /// spline.apply_mut(dynOp.as_mut());
    /// ```
    ///
    /// Complex operations should implement [Command].
    pub fn apply_mut<C: CommandMut<T> + ?Sized>(&mut self, op: &mut C) -> C::Result {
        op.execute(&mut self.mesh)
    }

    /// Perform an operation on the underlying TMesh
    ///
    /// Simple modifications can be done using clojures:
    /// ```
    /// # use t_spline::tmesh::*;
    /// # use t_spline::*;
    /// let mut spline = TSpline::new_unit_square();
    /// spline.apply(&mut |m: &TMesh<f64>| {});
    /// ```
    ///
    /// Dynamic dispatch modifications are also possible:
    /// ```
    /// # use t_spline::tmesh::*;
    /// # use t_spline::*;
    /// let mut spline = TSpline::new_unit_square();
    /// let mut dynOp: Box<dyn Command<f64, Result=()>> = Box::new(|m: &TMesh<f64>|{});
    /// spline.apply(dynOp.as_mut());
    /// ```
    ///
    /// Complex operations should implement [Command].
    pub fn apply<C: Command<T> + ?Sized>(&self, op: &mut C) -> C::Result {
        op.execute(&self.mesh)
    }

    pub fn mesh(&self) -> &TMesh<T> {
        &self.mesh
    }

    pub fn into_mesh(self) -> TMesh<T> {
        self.mesh
    }
}
