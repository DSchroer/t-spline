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
use rayon::prelude::*;
use t_spline::Point3;
use t_spline::algorithms::subs;
use t_spline::bounds::Bounded;
use t_spline::control_mesh::ControlMesh;
use t_spline::uv_mesh::ids::VertID;
use t_spline::uv_mesh::{LocalKnots, ValidationError};

pub fn tessellate<T: ControlMesh + Sync>(
    mesh: &T,
    resolution: usize,
) -> Result<Vec<Point3<T::Unit>>, ValidationError> {
    mesh.validate_control_mesh()?;

    let bounds = mesh.bounds();

    let knot_cache: Vec<_> = knot_vectors(mesh);

    Ok((0..resolution * resolution)
        .into_par_iter()
        .map(|i| {
            let p = bounds.interpolate(i, resolution);
            if mesh.contains_uv((p.0, p.1)) {
                subs(
                    mesh.control_points(),
                    bounds.interpolate(i, resolution),
                    &knot_cache,
                )
            } else {
                None
            }
        })
        .filter_map(|p| p)
        .collect())
}

fn knot_vectors(mesh: &(impl ControlMesh + Sync)) -> Vec<LocalKnots> {
    (0..mesh.points().len())
        .into_par_iter()
        .map(|v| mesh.infer_local_knots(VertID(v)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::unit_square::unit_square;
    use t_spline::{Point3, TSpline};

    #[test]
    pub fn it_can_evaluate_points_on_square() {
        let square: TSpline = unit_square();
        let knots = knot_vectors(&square);

        assert_eq!(
            Some(Point3::new(0., 0., 0.)),
            subs(square.control_points(), (0.0, 0.0), &knots)
        );
        assert_eq!(
            Some(Point3::new(1., 0., 0.)),
            subs(square.control_points(), (1.0, 0.0), &knots)
        );
        assert_eq!(
            Some(Point3::new(0., 1., 0.)),
            subs(square.control_points(), (0.0, 1.0), &knots)
        );
        assert_eq!(
            Some(Point3::new(1., 1., 0.)),
            subs(square.control_points(), (1.0, 1.0), &knots)
        );
    }

    #[test]
    pub fn it_can_tessellate_a_square() {
        let square: TSpline = unit_square();
        let points = tessellate(&square, 2).unwrap();

        assert_eq!(4, points.len());

        assert_eq!(Point3::new(0., 0., 0.), points[0]);
        assert_eq!(Point3::new(1., 0., 0.), points[1]);
        assert_eq!(Point3::new(0., 1., 0.), points[2]);
        assert_eq!(Point3::new(1., 1., 0.), points[3]);
    }

    #[test]
    pub fn it_can_evaluate_center() {
        let square: TSpline = unit_square();
        let knots = knot_vectors(&square);
        let center = subs(square.control_points(), (0.5, 0.5), &knots).unwrap();

        // Check components with epsilon tolerance
        let expected = Point3::new(0.5, 0.5, 0.0);
        let diff = center - expected;
        assert_eq!(0., diff.x);
        assert_eq!(0., diff.y);
        assert_eq!(0., diff.z);
    }
}
