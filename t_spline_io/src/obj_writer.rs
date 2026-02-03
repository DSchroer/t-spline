use t_spline::tmesh::TMesh;
use t_spline::math::Point;
use std::fmt::Write;

#[derive(Debug, Default, Clone)]
pub struct ObjWriter {
    obj: String,
    vertex_count: usize
}

impl ObjWriter {
    pub fn with_points(mut self, name: &str, points: &[Point]) -> Result<Self, std::fmt::Error> {
        let vertex_offset = self.vertex_count + 1;
        writeln!(self.obj, r"o {name}")?;

        for point in points {
            self.vertex_count += 1;
            writeln!(self.obj, "v {} {} {}", point.x, point.y, point.z)?;
        }

        write!(self.obj, "l ")?;
        for i in 0..points.len() {
            write!(self.obj, "{} ", i + vertex_offset)?;
        }
        writeln!(self.obj)?;

        Ok(self)
    }

    pub fn with_control_surface(mut self, name: &str,  mesh: &TMesh) -> Result<Self, std::fmt::Error> {
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
