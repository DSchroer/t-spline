use crate::Numeric;
use crate::tmesh::TMesh;
use crate::tmesh::control_point::ControlPoint;
use crate::tmesh::ids::{EdgeID, FaceID};

#[derive(Debug, Clone, Copy)]
pub struct Bounds<T> {
    pub s: (T, T),
    pub t: (T, T),
}

impl<T: Numeric> Default for Bounds<T> {
    fn default() -> Self {
        Self {
            s: (T::max_value(), T::min_value()),
            t: (T::max_value(), T::min_value()),
        }
    }
}

impl<T: Numeric> Bounds<T> {
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
        if resolution <= 1 {
            return (self.s.0, self.t.0);
        }

        let denom = T::from_usize(resolution - 1).unwrap();
        let u_i = T::from_usize(i % resolution).unwrap();
        let v_i = T::from_usize(i / resolution).unwrap();

        let s = self.s.0 + (u_i * (self.s.1 - self.s.0)) / denom;
        let t = self.t.0 + (v_i * (self.t.1 - self.t.0)) / denom;

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
        self.s.0 = self.s.0.min(point.uv.s);
        self.s.1 = self.s.1.max(point.uv.s);

        self.t.0 = self.t.0.min(point.uv.t);
        self.t.1 = self.t.1.max(point.uv.t);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_finds_the_center() {
        let b = Bounds{s: (0.0, 1.0), t: (0.0, 1.0), };

        assert_eq!(b.center(), (0.5, 0.5));
    }

    #[test]
    fn it_interpolates_types() {
        it_interpolates::<f64>();
        it_interpolates::<f32>();
        #[cfg(feature = "fixed")]
        it_interpolates::<fixed::types::I10F22>();
    }

    fn it_interpolates<T: Numeric>() {
        let b = Bounds{s: (T::zero(), T::one()), t: (T::zero(), T::one()), };

        assert_eq!(b.interpolate(0, 10), (T::zero(), T::zero()));
        assert_eq!(b.interpolate(99, 10), (T::one(), T::one()));
    }
}
