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
mod numeric;
pub mod uv_mesh;

pub use crate::numeric::Numeric;
use crate::uv_mesh::UVMesh;
use crate::uv_mesh::half_edge::HalfEdge;
use crate::uv_mesh::ids::{EdgeID, VertID};
use crate::uv_mesh::uv_point::UVPoint;
use alloc::vec::Vec;
pub use nalgebra::{Point3, Vector4};

#[derive(Debug, Default, Clone)]
pub struct TSpline<T> {
    points: Vec<UVPoint>,
    edges: Vec<HalfEdge>,
    control_points: Vec<Vector4<T>>,
}

pub struct TSplineMut<'a, T> {
    pub points: &'a mut Vec<UVPoint>,
    pub edges: &'a mut Vec<HalfEdge>,
    pub control_points: &'a mut Vec<Vector4<T>>,
}

impl<'a, T> From<&'a mut TSpline<T>> for TSplineMut<'a, T> {
    fn from(value: &'a mut TSpline<T>) -> Self {
        TSplineMut {
            points: &mut value.points,
            edges: &mut value.edges,
            control_points: &mut value.control_points,
        }
    }
}

impl<T> TSpline<T> {
    pub fn control_points(&self) -> &[Vector4<T>] {
        &self.control_points
    }

    pub fn edit<R>(&mut self, op: impl FnOnce(TSplineMut<'_, T>) -> R) -> R {
        let r = op(self.into());

        assert_eq!(self.points.len(), self.control_points.len());
        assert!(self.is_valid());

        r
    }
}

impl<T> UVMesh for TSpline<T> {
    fn points(&self) -> &[UVPoint] {
        &self.points
    }

    fn edges(&self) -> &[HalfEdge] {
        &self.edges
    }
}

impl<T: Numeric + 'static> TSpline<T> {
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
                T::from_isize(s).unwrap(),
                T::from_isize(t).unwrap(),
                T::zero(),
                T::one(),
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
