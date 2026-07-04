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
use num_traits::FromPrimitive;
use t_spline::control_mesh::ControlMeshMut;
use t_spline::uv_mesh::ids::VertID;
use thiserror::Error;

#[derive(Copy, Clone, Debug, Error)]
pub enum AlignError {
    #[error("missing point")]
    MissingPoint,
    #[error("missing control point")]
    MissingControlPoint,
    #[error("failed to cast")]
    FailedToCast,
}

pub fn align_control_points_to_cage<T: ControlMeshMut>(mesh: &mut T) -> Result<(), AlignError> {
    for i in 0..mesh.points().len() {
        let id = VertID(i);
        let p = mesh.point(id).ok_or(AlignError::MissingPoint)?.clone();
        let cp = mesh
            .control_point_mut(id)
            .ok_or(AlignError::MissingControlPoint)?;

        cp.x = T::Unit::from_isize(p.s).ok_or(AlignError::FailedToCast)?;
        cp.y = T::Unit::from_isize(p.t).ok_or(AlignError::FailedToCast)?;
    }

    Ok(())
}
