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
use crate::uv_mesh::{UVMesh, ValidationError};
use nalgebra::Vector4;

pub trait ControlMesh: UVMesh {
    type Unit: Numeric + Send + Sync + 'static;

    fn control_points(&self) -> &[Vector4<Self::Unit>];

    fn validate(&self) -> Result<(), ValidationError> {
        if self.control_points().len() != self.points().len() {
            return Err(ValidationError::DisconnectedPoints());
        }
        UVMesh::validate(self)
    }
}
