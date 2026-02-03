use crate::tmesh::control_point::ControlPoint;
use crate::tmesh::ids::FaceID;
use crate::tmesh::TMesh;

#[derive(Debug, Clone, Copy)]
pub struct Bounds {
    pub s: (f64, f64),
    pub t: (f64, f64),
}

impl Default for Bounds {
    fn default() -> Self {
        Self{
            s: (f64::MAX, f64::MIN),
            t: (f64::MAX, f64::MIN),
        }
    }
}

impl Bounds {
    /// Area of the ST Bounds
    pub fn area(&self) -> f64 {
        (self.s.1) - self.s.0 * (self.t.1 - self.t.0)
    }

    /// Center of the ST Bounds
    pub fn center(&self) -> (f64, f64) {
        ((self.s.0 + self.s.1) / 2., (self.t.0 + self.t.1) / 2.)
    }

    /// Point i in a grid of resolution * resolution
    pub fn interpolate(&self, i: usize, resolution: usize) -> (f64, f64) {
        let denom = (resolution - 1) as f64;

        let u_i = i % resolution;
        let v_i = i / resolution;
        let s = if resolution > 1 {
            self.s.0 + (u_i as f64 / denom) * (self.s.1 - self.s.0)
        } else {
            self.s.0
        };

        let t = if resolution > 1 {
            self.t.0 + (v_i as f64 / denom) * (self.t.1 - self.t.0)
        } else {
            self.t.0
        };

        (s, t)
    }

    pub fn add_mesh(&mut self, mesh: &TMesh) {
        for v in &mesh.vertices {
            self.s.0 = self.s.0.min(v.uv.s);
            self.s.1 = self.s.1.max(v.uv.s);

            self.t.0 = self.t.0.min(v.uv.t);
            self.t.1 = self.t.1.max(v.uv.t);
        }
    }

    pub fn add_face(&mut self, mesh: &TMesh, face: FaceID) {
        for e in mesh.face_edges(face) {
            let v = mesh.vertex(mesh.edge(e).origin);

            self.s.0 = self.s.0.min(v.uv.s);
            self.s.1 = self.s.1.max(v.uv.s);

            self.t.0 = self.t.0.min(v.uv.t);
            self.t.1 = self.t.1.max(v.uv.t);
        }
    }

}