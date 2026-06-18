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

pub mod algorithms;
mod numeric;
pub mod uv_mesh;

pub use crate::numeric::Numeric;
use crate::uv_mesh::UVMesh;
use alloc::vec::Vec;
pub use nalgebra::{Point3, Vector4};

pub type ControlPoints<T> = Vec<Vector4<T>>;

#[derive(Debug, Default, Clone)]
pub struct TMesh<T> {
    pub mesh: UVMesh,
    pub control_points: ControlPoints<T>,
}

#[derive(Debug, Default, Clone)]
pub struct TSpline<T> {
    mesh: TMesh<T>,
}

impl<T> TSpline<T> {
    pub fn new(mesh: UVMesh, control_points: ControlPoints<T>) -> Self {
        assert_eq!(mesh.points.len(), control_points.len());

        Self {
            mesh: TMesh{ mesh, control_points},
        }
    }

    /// Perform an operation on the underlying TMesh
    ///
    /// Simple modifications can be done using clojures:
    /// ```
    /// # use t_spline::*;
    /// # use t_spline::uv_mesh::UVMesh;
    /// let mut spline = TSpline::new_unit_square();
    /// spline.edit(&mut |m: &mut TMesh<f64>| m.control_points[0].z = 1.0);
    /// ```
    ///
    /// More state modifications can be done using functions:
    /// ```
    /// # use t_spline::*;
    /// # use t_spline::uv_mesh::UVMesh;
    /// let mut spline = TSpline::new_unit_square();
    /// spline.edit(&mut modification);
    ///
    /// fn modification(m: &mut TMesh<f64>) {
    /// }
    /// ```
    ///
    /// Dynamic dispatch modifications are also possible:
    /// ```
    /// # use t_spline::uv_mesh::*;
    /// # use t_spline::*;
    /// let mut spline = TSpline::new_unit_square();
    /// let mut dynOp: Box<dyn Edit<f64, Result=()>> = Box::new(|m: &mut TMesh<f64>|{});
    /// spline.edit(dynOp.as_mut());
    /// ```
    ///
    /// Complex operations should implement [Command].
    pub fn edit<C: Edit<T> + ?Sized>(&mut self, op: &mut C) -> C::Result {
        let r = op.execute(&mut self.mesh);

        assert!(self.mesh.mesh.is_valid());
        assert_eq!(self.mesh.mesh.points.len(), self.mesh.control_points.len());

        r
    }

    pub fn mesh(&self) -> &UVMesh {
        &self.mesh.mesh
    }

    pub fn control_points(&self) -> &ControlPoints<T> {
        &self.mesh.control_points
    }
}

impl<T: Numeric> TSpline<T> {
    pub fn new_unit_square() -> Self {
        let mesh = UVMesh::new_unit_square();
        let control_points = mesh.points
            .iter()
            .map(|p| Vector4::new(
                T::from_isize(p.s).unwrap(),
                T::from_isize(p.t).unwrap(),
                T::zero(),
                T::one()))
            .collect();
        Self::new(mesh, control_points)
    }
}

/// Edit operation to perform on a spline mesh
pub trait Edit<T> {
    /// Error type of the operation
    type Result;

    /// Perform the operation, returning any error as needed
    fn execute(&mut self, mesh: &mut TMesh<T>)
    -> Self::Result;
}

impl<T, F, Out> Edit<T> for F
where
    F: FnMut(&mut TMesh<T>) -> Out,
{
    type Result = Out;

    fn execute(&mut self, mesh: &mut TMesh<T>) -> Out {
        self(mesh)
    }
}
