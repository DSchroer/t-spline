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

use crate::tmesh::ids::EdgeID;
use crate::tmesh::segment::ParamPoint;
use nalgebra::Vector4;

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
