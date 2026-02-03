use crate::tmesh::TMesh;
use crate::tmesh::control_point::ControlPoint;
use crate::tmesh::ids::{EdgeID, FaceID};
use fixed::traits::Fixed;
use num_traits::real::Real;
use num_traits::{Bounded, FromPrimitive, Num, NumCast};

#[derive(Debug, Clone, Copy)]
pub struct Bounds<T> {
    pub s: (T, T),
    pub t: (T, T),
}

impl<T: Bounded> Default for Bounds<T> {
    fn default() -> Self {
        Self {
            s: (T::max_value(), T::min_value()),
            t: (T::max_value(), T::min_value()),
        }
    }
}

impl<T: Num + Copy + PartialOrd + FromPrimitive> Bounds<T> {
    /// Area of the ST Bounds
    pub fn area(&self) -> T {
        (self.s.1) - self.s.0 * (self.t.1 - self.t.0)
    }

    /// Center of the ST Bounds
    pub fn center(&self) -> (T, T) {
        (
            (self.s.0 + self.s.1) / T::from_usize(2).unwrap(),
            (self.t.0 + self.t.1) / T::from_usize(2).unwrap(),
        )
    }

    /// Point i in a grid of resolution * resolution
    pub fn interpolate(&self, i: usize, resolution: usize) -> (T, T) {
        let denom = T::from_usize(resolution - 1).unwrap();

        let u_i = T::from_usize(i % resolution).unwrap();
        let v_i = T::from_usize(i / resolution).unwrap();
        let s = if resolution > 1 {
            self.s.0 + (u_i / denom) * (self.s.1 - self.s.0)
        } else {
            self.s.0
        };

        let t = if resolution > 1 {
            self.t.0 + (v_i / denom) * (self.t.1 - self.t.0)
        } else {
            self.t.0
        };

        (s, t)
    }

    pub fn add_mesh(&mut self, mesh: &TMesh<T>) {
        for v in &mesh.vertices {
            self.add_vertex(v)
        }
    }

    pub fn add_face(&mut self, mesh: &TMesh<T>, face: FaceID) {
        for e in mesh.face_edges(face) {
            self.add_edge(mesh, e);
        }
    }

    pub fn add_edge(&mut self, mesh: &TMesh<T>, edge: EdgeID) {
        let v = mesh.vertex(mesh.edge(edge).origin);
        self.add_vertex(v)
    }

    pub fn add_vertex(&mut self, point: &ControlPoint<T>) {
        self.s.0 = if self.s.0 <= point.uv.s {
            self.s.0
        } else {
            point.uv.s
        };
        self.s.1 = if self.s.1 >= point.uv.s {
            self.s.1
        } else {
            point.uv.s
        };

        self.t.0 = if self.t.0 <= point.uv.t {
            self.t.0
        } else {
            point.uv.t
        };
        self.t.1 = if self.t.1 >= point.uv.t {
            self.t.1
        } else {
            point.uv.t
        };
    }
}
