use rayon::prelude::*;
use crate::models::tmesh::TMesh;
use crate::models::VertID;

#[derive(Debug, Default, Clone)]
pub struct TSpline {
    mesh: TMesh,
    knot_cache: Vec<([f64; 5], [f64; 5])>,
}

pub struct Bounds {
    pub s: (f64, f64),
    pub t: (f64, f64),
}

impl From<TMesh> for TSpline {
    fn from(value: TMesh) -> Self {
        TSpline::new(value)
    }
}

impl TSpline {
    pub fn new(mesh: TMesh) -> Self {
        // calculate the knot_cache from the mesh
        // do not allow edits to mesh after to ensure that the cache is correct
        let knot_cache = (0..mesh.vertices.len())
            .into_par_iter()
            .map(|v| {
                mesh.infer_local_knots(VertID(v))
            }).collect();

        Self{ mesh, knot_cache }
    }

    pub fn bounds(&self) -> Bounds {
        let mut s_min = f64::MAX;
        let mut s_max = f64::MIN;

        let mut t_min = f64::MAX;
        let mut t_max = f64::MIN;
        for v in &self.mesh.vertices {
            if v.uv.s < s_min {
                s_min = v.uv.s;
            }
            if v.uv.s > s_max {
                s_max = v.uv.s;
            }

            if v.uv.t < t_min {
                t_min = v.uv.t;
            }
            if v.uv.t > t_max {
                t_max = v.uv.t;
            }
        }

        Bounds{
            s: (s_min, s_max),
            t: (t_min, t_max),
        }
    }

    pub fn knot_cache(&self) -> &Vec<([f64; 5], [f64; 5])> {
        &self.knot_cache
    }

    pub fn mesh(&self) -> &TMesh {
        &self.mesh
    }

    pub fn into_mesh(self) -> TMesh {
        self.mesh
    }
}