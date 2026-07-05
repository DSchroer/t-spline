/*
 * Copyright (C) 2026 Dominick Schroer
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */
use t_spline::control_mesh::ControlMeshMut;
use t_spline::uv_mesh::half_edge::HalfEdge;
use t_spline::uv_mesh::ids::EdgeID;
use t_spline::uv_mesh::uv_point::UVCoord;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ExtrudeError {
    #[error("mesh is missing edge")]
    MissingEdge(),
    #[error("mesh is missing control point")]
    MissingControlPoint(),
    #[error("edge already has twin")]
    HasTwin(),
}

/// extrude `edge_id` by 1 unit
pub fn extrude_edge(mesh: &mut impl ControlMeshMut, edge_id: EdgeID) -> Result<(), ExtrudeError> {
    let edge = mesh.edge(edge_id).ok_or(ExtrudeError::MissingEdge())?;
    if edge.twin.is_some() {
        return Err(ExtrudeError::HasTwin());
    }

    let line = mesh.line(edge);
    let axis = line.direction();
    let line = mesh.start_end(edge);

    let mut offset = 1;
    let edge_axis = mesh
        .point(edge.origin)
        .ok_or(ExtrudeError::MissingControlPoint())?
        .value_in_dir(axis);
    for (_, e) in mesh.edge_loop(edge) {
        let o = mesh
            .point(e.origin)
            .ok_or(ExtrudeError::MissingControlPoint())?
            .value_in_dir(axis);
        if o < edge_axis {
            offset = 1;
            break;
        } else if o > edge_axis {
            offset = -1;
            break;
        }
    }

    let a_id = edge.origin;
    let b_id = mesh
        .edge(edge.next)
        .ok_or(ExtrudeError::MissingEdge())?
        .origin;

    let mut c = line.1.clone();
    c.add_in_dir(axis, offset);
    let c_cp = mesh
        .control_point(a_id)
        .ok_or(ExtrudeError::MissingControlPoint())?
        .clone();

    let mut d = line.0.clone();
    d.add_in_dir(axis, offset);
    let d_cp = mesh
        .control_point(b_id)
        .ok_or(ExtrudeError::MissingControlPoint())?
        .clone();

    let id_start = mesh.edges().len();
    let e1 = EdgeID(id_start);
    let e2 = EdgeID(id_start + 1);
    let e3 = EdgeID(id_start + 2);
    let e4 = EdgeID(id_start + 3);

    c.outgoing_edge = e3;
    let c_id = mesh.push_point(c);
    mesh.push_control_point(c_cp);

    d.outgoing_edge = e4;
    let d_id = mesh.push_point(d);
    mesh.push_control_point(d_cp);

    // e1
    let twin_id = mesh.push_edge(HalfEdge {
        origin: b_id,
        twin: Some(edge_id),
        next: e2,
        prev: e4,
    });
    mesh.edge_mut(edge_id)
        .ok_or(ExtrudeError::MissingEdge())?
        .twin = Some(twin_id);

    // e2
    mesh.push_edge(HalfEdge {
        origin: a_id,
        twin: None,
        next: e3,
        prev: e1,
    });

    // e3
    mesh.push_edge(HalfEdge {
        origin: d_id,
        twin: None,
        next: e4,
        prev: e2,
    });

    // e4
    mesh.push_edge(HalfEdge {
        origin: c_id,
        twin: None,
        next: e1,
        prev: e3,
    });

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tessellate::tessellate;
    use t_spline::TSpline;
    use t_spline::control_mesh::ControlMesh;
    use t_spline::uv_mesh::UVMesh;
    use t_spline::uv_mesh::ids::VertID;

    #[test]
    fn it_extrudes_unit_square() {
        let mut mesh = TSpline::new_unit_square();

        extrude_edge(&mut mesh, EdgeID(2)).unwrap();

        mesh.validate_uv_mesh_integrity().unwrap();
        mesh.validate_control_mesh().unwrap();

        // verteces are connected correctly
        assert_eq!(2, mesh.connected_verteces(VertID(0)).len());
        assert_eq!(2, mesh.connected_verteces(VertID(1)).len());
        assert_eq!(3, mesh.connected_verteces(VertID(2)).len());
        assert_eq!(3, mesh.connected_verteces(VertID(3)).len());
        assert_eq!(2, mesh.connected_verteces(VertID(4)).len());
        assert_eq!(2, mesh.connected_verteces(VertID(5)).len());

        assert_eq!(2, mesh.faces().count());
    }

    #[test]
    fn it_tessellates_extrusion() {
        let mut mesh = TSpline::new_unit_square();
        extrude_edge(&mut mesh, EdgeID(2)).unwrap();

        tessellate(&mesh, 10).unwrap();
    }
}
