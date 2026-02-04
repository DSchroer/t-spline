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

use crate::tmesh::direction::Direction;
use crate::tmesh::ids::{EdgeID, FaceID, VertID};

#[derive(Debug, Clone)]
pub struct HalfEdge {
    pub origin: VertID,       // Vertex where this edge starts
    pub twin: Option<EdgeID>, // The opposite half-edge
    pub face: Option<FaceID>, // The face to the left of this edge
    pub next: EdgeID,         // Next half-edge in the face loop
    pub prev: EdgeID,         // Previous half-edge in the face loop

    /// T-Spline specific: Knot Interval associated with this edge
    /// If the edge runs in S direction, this is a delta-s.
    pub knot_interval: f64,
    pub direction: Direction,
}
