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

use crate::TSpline;
use crate::tmesh::TMesh;

/// Edit operation to perform on a spline mesh
pub trait Command<T> {
    /// Output of the operation
    type Result;

    /// Perform the operation, returning any error as needed
    fn execute(&mut self, mesh: &TMesh<T>) -> Self::Result;

    fn apply(&mut self, spline: &TSpline<T>) -> Self::Result {
        spline.apply(self)
    }
}

/// Edit operation to perform on a spline mesh
pub trait CommandMut<T> {
    /// Error type of the operation
    type Result;

    /// Perform the operation, returning any error as needed
    fn execute(&mut self, mesh: &mut TMesh<T>) -> Self::Result;

    fn apply(&mut self, spline: &mut TSpline<T>) -> Self::Result {
        spline.apply_mut(self)
    }
}

impl<T, F, Out> Command<T> for F
where
    F: FnMut(&TMesh<T>) -> Out,
{
    type Result = Out;

    fn execute(&mut self, mesh: &TMesh<T>) -> Self::Result {
        self(mesh)
    }
}

impl<T, F, Out> CommandMut<T> for F
where
    F: FnMut(&mut TMesh<T>) -> Out,
{
    type Result = Out;

    fn execute(&mut self, mesh: &mut TMesh<T>) -> Out {
        self(mesh)
    }
}
