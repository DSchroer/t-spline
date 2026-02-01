use rayon::prelude::*;
use thiserror::Error;
use crate::models::tmesh::TMesh;
use crate::models::VertID;
use crate::operation::SplineOp;

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

impl From<TSpline> for TMesh {
    fn from(value: TSpline) -> Self {
        value.into_mesh()
    }
}

#[derive(Error, Debug)]
pub enum SplineError<T> {
    #[error("spline operation failed: {0:?}")]
    Operation(T)
}

impl TSpline {
    pub fn new(mesh: TMesh) -> Self {
        let knot_cache = Self::build_knot_cache(&mesh);
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

    /// Perform an operation on the underlying TMesh
    ///
    /// Simple modifications can be done using clojures:
    /// ```
    /// # use t_spline::models::*;
    /// let mut spline = TSpline::new_unit_square();
    /// spline.perform(&mut |m: &mut TMesh| m.vertices[0].geometry.z = 1.0);
    /// ```
    ///
    /// Errors will be passed through as needed:
    /// ```
    /// # use t_spline::models::*;
    /// let mut spline = TSpline::new_unit_square();
    /// let res = spline.perform(&mut |m: &mut TMesh| Err(()));
    /// res.unwrap_err();
    /// ```
    ///
    /// Dynamic dispatch modifications are also possible:
    /// ```
    /// # use t_spline::operation::*;
    /// # use t_spline::models::*;
    /// let mut spline = TSpline::new_unit_square();
    /// let mut dynOp: Box<dyn SplineOp<Error=()>> = Box::new(|m: &mut TMesh|{});
    /// spline.perform(dynOp.as_mut());
    /// ```
    ///
    /// Complex operations should implement [SplineOp].
    pub fn perform<T: SplineOp + ?Sized>(&mut self, op: &mut T) -> Result<&mut Self, SplineError<T::Error>> {
        op.perform(&mut self.mesh).map_err(SplineError::Operation)?;

        self.knot_cache = Self::build_knot_cache(&self.mesh);
        Ok(self)
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

    fn build_knot_cache(mesh: &TMesh) -> Vec<([f64; 5], [f64; 5])> {
        // calculate the knot_cache from the mesh
        // do not allow edits to mesh after to ensure that the cache is correct
        (0..mesh.vertices.len())
            .into_par_iter()
            .map(|v| {
                mesh.infer_local_knots(VertID(v))
            }).collect()
    }
}
