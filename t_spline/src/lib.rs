/*
 * Copyright (C) 2026 Dominick Schroer
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

#![cfg_attr(not(test), no_std)]

extern crate alloc;

mod commands;
mod numeric;
mod shapes;
pub mod tmesh;

pub use crate::commands::{Command, CommandMut};
pub use crate::numeric::Numeric;
pub use nalgebra::{Point3, Vector4};

use crate::tmesh::TMesh;

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
