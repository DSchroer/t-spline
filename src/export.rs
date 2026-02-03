use crate::models::TMesh;
use cgmath::Point3;
use std::fmt::Write;

// TODO: switch to obj for multiple objects in same file
#[derive(Debug, Default, Clone)]
pub struct PlyWriter {
    header: String,
    body: String,
}

impl PlyWriter {
    pub fn with_point(mut self, points: &[Point3<f64>]) -> Result<PlyWriter, std::fmt::Error> {
        writeln!(
            self.header,
            r"element vertex {}
property float x
property float y
property float z",
            points.len()
        )?;

        for point in points {
            writeln!(self.body, "{} {} {}", point.x, point.y, point.z)?;
        }

        Ok(self)
    }

    pub fn with_geometry(mut self, mesh: &TMesh) -> Result<PlyWriter, std::fmt::Error> {
        writeln!(
            self.header,
            r"element vertex {}
property float x
property float y
property float z
element edge {}
property int vertex1
property int vertex2",
            mesh.vertices.len(),
            mesh.edges.len()
        )?;

        for point in &mesh.vertices {
            writeln!(
                self.body,
                "{} {} {}",
                point.geometry.x, point.geometry.y, point.geometry.z
            )?;
        }

        for edge in &mesh.edges {
            writeln!(
                self.body,
                "{} {}",
                edge.origin.0,
                mesh.edge(edge.next).origin.0
            )?;
        }

        Ok(self)
    }

    pub fn write(&self, w: &mut impl std::io::Write) -> Result<(), std::io::Error> {
        writeln!(w, "ply\nformat ascii 1.0")?;
        write!(w, "{}", self.header)?;
        writeln!(w, "end_header")?;
        write!(w, "{}", self.body)?;
        Ok(())
    }
}
