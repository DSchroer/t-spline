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

use crate::tmesh::ids::{EdgeID, FaceID, VertID};

/// Half edge
#[derive(Debug, Clone)]
pub struct HalfEdge {
    /// Vertex where this edge starts
    pub origin: VertID,
    /// The opposite half-edge (right side)
    pub twin: Option<EdgeID>,
    /// The face to the left of this edge
    pub face: FaceID,
    /// Next half-edge in the face loop
    pub next: EdgeID,
    /// Previous half-edge in the face loop
    pub prev: EdgeID,
}
