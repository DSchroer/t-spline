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

use crate::Numeric;
use crate::uv_mesh::ids::VertID;
use crate::uv_mesh::{UVMesh, UVMeshMut, ValidationError};
use nalgebra::Vector4;

pub trait ControlMeshMut: ControlMesh + UVMeshMut {
    fn push_control_point(&mut self, point: Vector4<Self::Unit>) -> VertID;
    fn control_point_mut(&mut self, id: VertID) -> Option<&mut Vector4<Self::Unit>>;
}

pub trait ControlMesh: UVMesh {
    type Unit: Numeric + Send + Sync + 'static;

    fn control_points(&self) -> &[Vector4<Self::Unit>];

    fn control_point(&self, id: VertID) -> Option<&Vector4<Self::Unit>> {
        self.control_points().get(id.0)
    }

    fn validate_control_mesh(&self) -> Result<(), ValidationError> {
        if self.control_points().len() != self.points().len() {
            return Err(ValidationError::DisconnectedPoints());
        }
        UVMesh::validate_uv_mesh_integrity(self)
    }
}
