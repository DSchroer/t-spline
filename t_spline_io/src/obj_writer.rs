use std::fmt::Write;
use t_spline::tmesh::TMesh;
use t_spline::{Numeric, Point3};

#[derive(Debug, Default, Clone)]
pub struct ObjWriter {
    obj: String,
    vertex_count: usize,
}

impl ObjWriter {
    pub fn with_points<T: Numeric + 'static>(
        mut self,
        name: &str,
        points: &[Point3<T>],
    ) -> Result<Self, std::fmt::Error> {
        writeln!(self.obj, r"o {name}")?;

        for point in points {
            self.vertex_count += 1;
            writeln!(self.obj, "v {} {} {}", point.x, point.y, point.z)?;
        }

        Ok(self)
    }

    pub fn with_triangles<T: Numeric + 'static>(
        mut self,
        name: &str,
        points: &[Point3<T>],
        triangles: &[[usize; 3]],
    ) -> Result<Self, std::fmt::Error> {
        let vertex_offset = self.vertex_count + 1;
        writeln!(self.obj, r"o {name}")?;

        for point in points {
            self.vertex_count += 1;
            writeln!(self.obj, "v {} {} {}", point.x, point.y, point.z)?;
        }

        for t in triangles {
            writeln!(self.obj, "f {} {} {}", t[0] + vertex_offset, t[1] + vertex_offset, t[2] + vertex_offset)?;
        }

        Ok(self)
    }

    pub fn with_control_surface<T: Numeric + 'static>(
        mut self,
        name: &str,
        mesh: &TMesh<T>,
    ) -> Result<Self, std::fmt::Error> {
        let vertex_offset = self.vertex_count + 1;
        writeln!(self.obj, r"o {name}")?;

        for point in &mesh.vertices {
            self.vertex_count += 1;
            writeln!(
                self.obj,
                "v {} {} {}",
                point.geometry.x, point.geometry.y, point.geometry.z
            )?;
        }

        for edge in &mesh.edges {
            writeln!(
                self.obj,
                "l {} {}",
                edge.origin.0 + vertex_offset,
                mesh.edge(edge.next).origin.0 + vertex_offset
            )?;
        }

        Ok(self)
    }

    pub fn write(self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        write!(w, "{}", self.obj)
    }
}
