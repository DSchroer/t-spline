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
pub mod bounds;
pub mod control_mesh;
mod numeric;
pub mod uv_mesh;

use crate::control_mesh::ControlMesh;
pub use crate::numeric::Numeric;
use crate::uv_mesh::UVMesh;
use crate::uv_mesh::half_edge::HalfEdge;
use crate::uv_mesh::ids::{EdgeID, VertID};
use crate::uv_mesh::uv_point::UVPoint;
use alloc::vec::Vec;
pub use nalgebra::{Point3, Vector4};
use num_traits::ToPrimitive;

#[derive(Debug, Default, Clone)]
pub struct TSpline {
    points: Vec<UVPoint>,
    edges: Vec<HalfEdge>,
    control_points: Vec<Vector4<f64>>,
}

impl ControlMesh for TSpline {
    type Unit = f64;

    fn control_points(&self) -> &[Vector4<f64>] {
        &self.control_points
    }
}

impl UVMesh for TSpline {
    fn points(&self) -> &[UVPoint] {
        &self.points
    }

    fn edges(&self) -> &[HalfEdge] {
        &self.edges
    }
}

impl TSpline {
    pub fn new_unit_square() -> Self {
        let mut mesh = TSpline {
            points: Vec::with_capacity(4),
            edges: Vec::with_capacity(4),
            control_points: Vec::with_capacity(4),
        };

        // 1. Define 4 Corner Vertices
        let coords: [(isize, isize); _] = [(0, 0), (1, 0), (1, 1), (0, 1)];
        for (i, (s, t)) in coords.into_iter().enumerate() {
            mesh.points.push(UVPoint {
                s,
                t,
                outgoing_edge: EdgeID(i), // Inner edges are 0..4
            });

            mesh.control_points.push(Vector4::new(
                s.to_f64().unwrap(),
                t.to_f64().unwrap(),
                0f64,
                1f64,
            ));
        }

        // 2. Define 4 inner Half-Edges in a CCW loop
        for i in 0..4 {
            mesh.edges.push(HalfEdge {
                origin: VertID(i),
                next: EdgeID((i + 1) % 4),
                prev: EdgeID((i + 3) % 4),
                twin: None,
            });
        }

        mesh
    }
}
