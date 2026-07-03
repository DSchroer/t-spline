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
use crate::uv_mesh::direction::Direction;
use crate::uv_mesh::uv_point::UVPoint;

#[derive(Copy, Clone, Debug)]
pub struct Line<'a>(pub &'a UVPoint, pub &'a UVPoint);

impl Line<'_> {
    pub fn delta(&self, axis: Direction) -> isize {
        match axis {
            Direction::S => self.0.s - self.1.s,
            Direction::T => self.0.t - self.1.t,
        }
    }

    pub fn is_axis_aligned(&self, axis: Direction) -> bool {
        match axis {
            Direction::S => self.0.t == self.1.t,
            Direction::T => self.0.s == self.1.s,
        }
    }

    pub fn is_orthogonal(&self) -> bool {
        self.is_axis_aligned(Direction::S) || self.is_axis_aligned(Direction::T)
    }

    fn min(&self, axis: Direction) -> isize {
        debug_assert!(
            self.is_axis_aligned(axis),
            "line {self:?} is aligned along {axis:?}, can not take min"
        );
        isize::min(self.0.value_in_dir(axis), self.1.value_in_dir(axis))
    }

    fn max(&self, axis: Direction) -> isize {
        debug_assert!(
            self.is_axis_aligned(axis),
            "line {self:?} is aligned along {axis:?}, can not take max"
        );
        isize::max(self.0.value_in_dir(axis), self.1.value_in_dir(axis))
    }

    fn axis_coord(&self, axis: Direction) -> isize {
        debug_assert!(
            self.is_axis_aligned(axis),
            "line {self:?} is not aligned along {axis:?}"
        );
        match axis {
            Direction::S => self.0.t,
            Direction::T => self.0.s,
        }
    }

    pub fn intersects(&self, origin: &UVPoint, axis: Direction, positive: bool) -> Option<UVPoint> {
        // Check if the line is collinear with the ray direction (i.e., it's aligned)
        if self.is_axis_aligned(axis) {
            return None; // The line is aligned with the ray, so no intersection
        }

        let line_coord = self.axis_coord(axis.opposite());
        let origin_axis_coord = origin.value_in_dir(axis);
        let origin_opposite_coord = origin.value_in_dir(axis.opposite());

        // check the origin is on the line
        if line_coord == origin_axis_coord {
            return None;
        }

        // check the origin is behind the line
        if positive && origin_axis_coord > line_coord {
            return None;
        }

        // check the origin is in front of the line
        if !positive && origin_axis_coord < line_coord {
            return None;
        }

        // bounds check
        if origin_opposite_coord < self.min(axis.opposite())
            || origin_opposite_coord > self.max(axis.opposite())
        {
            return None;
        }

        // Create a new point at the intersection (this is where the ray intersects with the infinite line)
        let mut intersection_point = origin.clone();
        match axis {
            Direction::S => intersection_point.s = line_coord,
            Direction::T => intersection_point.t = line_coord,
        }

        Some(intersection_point)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::uv_mesh::ids::EdgeID;

    macro_rules! point {
        ($s: literal, $t: literal) => {
            UVPoint {
                s: $s,
                t: $t,
                outgoing_edge: EdgeID(0),
            }
        };
    }

    macro_rules! line {
        (($s1: literal, $t1: literal), ($s2: literal, $t2: literal)) => {
            Line(&point!($s1, $t1), &point!($s2, $t2))
        };
    }

    #[test]
    fn it_intersects_line_along_t() {
        let line = line!((0, 1), (2, 1));

        // check points along the line
        assert_eq!(
            Some(point!(0, 1)),
            line.intersects(&point!(0, 0), Direction::T, true)
        );
        assert_eq!(
            Some(point!(1, 1)),
            line.intersects(&point!(1, 0), Direction::T, true)
        );
        assert_eq!(
            Some(point!(2, 1)),
            line.intersects(&point!(2, 0), Direction::T, true)
        );

        // check points on the line
        assert_eq!(None, line.intersects(&point!(0, 1), Direction::T, true));
        assert_eq!(None, line.intersects(&point!(1, 1), Direction::T, true));
        assert_eq!(None, line.intersects(&point!(2, 1), Direction::T, true));

        // check past boundaries
        assert_eq!(None, line.intersects(&point!(-1, 0), Direction::T, true));
        assert_eq!(None, line.intersects(&point!(3, 0), Direction::T, true));

        // check backwards ray
        assert_eq!(None, line.intersects(&point!(0, 0), Direction::T, false));
        assert_eq!(None, line.intersects(&point!(1, 0), Direction::T, false));
        assert_eq!(None, line.intersects(&point!(2, 0), Direction::T, false));

        // check backwards ray from other side
        assert_eq!(
            Some(point!(0, 1)),
            line.intersects(&point!(0, 2), Direction::T, false)
        );
        assert_eq!(
            Some(point!(1, 1)),
            line.intersects(&point!(1, 2), Direction::T, false)
        );
        assert_eq!(
            Some(point!(2, 1)),
            line.intersects(&point!(2, 2), Direction::T, false)
        );
    }

    #[test]
    fn it_intersects_line_along_s() {
        let line = line!((1, 0), (1, 2));

        // check points along the line
        assert_eq!(
            Some(point!(1, 0)),
            line.intersects(&point!(0, 0), Direction::S, true)
        );
        assert_eq!(
            Some(point!(1, 1)),
            line.intersects(&point!(0, 1), Direction::S, true)
        );
        assert_eq!(
            Some(point!(1, 2)),
            line.intersects(&point!(0, 2), Direction::S, true)
        );

        // check points on the line
        assert_eq!(None, line.intersects(&point!(1, 0), Direction::S, true));
        assert_eq!(None, line.intersects(&point!(1, 1), Direction::S, true));
        assert_eq!(None, line.intersects(&point!(1, 2), Direction::S, true));

        // check past boundaries
        assert_eq!(None, line.intersects(&point!(0, -1), Direction::S, true));
        assert_eq!(None, line.intersects(&point!(0, 3), Direction::S, true));

        // check backwards ray
        assert_eq!(None, line.intersects(&point!(0, 0), Direction::S, false));
        assert_eq!(None, line.intersects(&point!(0, 1), Direction::S, false));
        assert_eq!(None, line.intersects(&point!(0, 2), Direction::S, false));

        // check backwards ray from other side
        assert_eq!(
            Some(point!(1, 0)),
            line.intersects(&point!(2, 0), Direction::S, false)
        );
        assert_eq!(
            Some(point!(1, 1)),
            line.intersects(&point!(2, 1), Direction::S, false)
        );
        assert_eq!(
            Some(point!(1, 2)),
            line.intersects(&point!(2, 2), Direction::S, false)
        );
    }
}
