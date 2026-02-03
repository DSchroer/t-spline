use crate::tmesh::TMesh;
use cgmath::Point3;
use std::fmt::Write;

#[derive(Debug, Default, Clone)]
pub struct ObjWriter {
    obj: String
}

impl ObjWriter {
    pub fn with_points(mut self, name: &str, points: &[Point3<f64>]) -> Result<Self, std::fmt::Error> {
        writeln!(self.obj, r"o {name}")?;

        for point in points {
            writeln!(self.obj, "v {} {} {}", point.x, point.y, point.z)?;
        }

        Ok(self)
    }

    pub fn with_control_surface(mut self, name: &str,  mesh: &TMesh) -> Result<Self, std::fmt::Error> {
        writeln!(self.obj, r"o {name}")?;

        for point in &mesh.vertices {
            writeln!(
                self.obj,
                "v {} {} {} {}",
                point.geometry.x, point.geometry.y, point.geometry.z, point.geometry.w
            )?;
        }

        for edge in &mesh.edges {
            writeln!(
                self.obj,
                "l {} {}",
                edge.origin.0,
                mesh.edge(edge.next).origin.0
            )?;
        }

        Ok(self)
    }

    pub fn write(self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        write!(w, "{}", self.obj)
    }
}
