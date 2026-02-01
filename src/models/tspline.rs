use rayon::prelude::*;
use crate::models::tmesh::TMesh;
use crate::models::VertID;

#[derive(Debug, Default, Clone)]
pub struct TSpline {
    mesh: TMesh,
    knot_cache: Vec<([f64; 5], [f64; 5])>,
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