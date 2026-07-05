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
use num_traits::{One, ToPrimitive, Zero};
use t_spline::Vector4;
use t_spline::control_mesh::ControlMeshMut;
use t_spline::uv_mesh::half_edge::HalfEdge;
use t_spline::uv_mesh::ids::{EdgeID, VertID};
use t_spline::uv_mesh::uv_point::UVPoint;

pub fn unit_square<T: ControlMeshMut + Default>() -> T {
    let mut mesh = T::default();

    // 1. Define 4 Corner Vertices
    let coords: [(T::Unit, T::Unit); _] = [
        (T::Unit::zero(), T::Unit::zero()),
        (T::Unit::one(), T::Unit::zero()),
        (T::Unit::one(), T::Unit::one()),
        (T::Unit::zero(), T::Unit::one()),
    ];
    for (i, (s, t)) in coords.into_iter().enumerate() {
        mesh.push_point(UVPoint {
            s: s.to_isize().unwrap(),
            t: t.to_isize().unwrap(),
            outgoing_edge: EdgeID(i), // Inner edges are 0..4
        });

        mesh.push_control_point(Vector4::new(s, t, T::Unit::zero(), T::Unit::one()));
    }

    // 2. Define 4 inner Half-Edges in a CCW loop
    for i in 0..4 {
        mesh.push_edge(HalfEdge {
            origin: VertID(i),
            next: EdgeID((i + 1) % 4),
            prev: EdgeID((i + 3) % 4),
            twin: None,
        });
    }

    mesh
}

#[cfg(test)]
mod test {
    use super::*;
    use t_spline::TSpline;
    use t_spline::control_mesh::ControlMesh;
    use t_spline::uv_mesh::UVMesh;

    #[test]
    pub fn it_creates_t_spline() {
        let mesh: TSpline = unit_square();

        assert!(mesh.validate_control_mesh().is_ok());

        assert_eq!(4, mesh.points().len());
        assert_eq!(4, mesh.edges().len());
    }
}
