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
use crate::uv_mesh::uv_point::{UVCoord, UVPoint};
use nalgebra::{Scalar, Vector2};
use num_traits::{Num, NumAssign};

#[derive(Copy, Clone, Debug)]
pub struct Line<T>(Vector2<T>, Vector2<T>);

impl Line<isize> {
    pub fn from_uv_points(a: &UVPoint, b: &UVPoint) -> Self {
        Self(Vector2::new(a.s, a.t), Vector2::new(b.s, b.t))
    }
}

impl<T: Scalar + Copy + Num + NumAssign + Ord + 'static> Line<T> {
    pub fn s0(&self) -> T {
        self.0.x
    }
    pub fn t0(&self) -> T {
        self.0.y
    }
    pub fn s1(&self) -> T {
        self.1.x
    }
    pub fn t1(&self) -> T {
        self.1.y
    }

    pub fn delta(&self, axis: Direction) -> T {
        match axis {
            Direction::S => self.s0() - self.s1(),
            Direction::T => self.t0() - self.t1(),
        }
    }

    pub fn direction(&self) -> Direction {
        if self.s0() == self.s1() {
            Direction::S
        } else if self.t1() == self.t1() {
            Direction::T
        } else {
            panic!("line is not axis aligned")
        }
    }

    pub fn length(&self) -> T {
        let dir = self.direction().opposite();
        self.max(dir) - self.min(dir)
    }

    pub fn is_axis_aligned(&self, axis: Direction) -> bool {
        match axis {
            Direction::S => self.t0() == self.t1(),
            Direction::T => self.s0() == self.s1(),
        }
    }

    pub fn is_orthogonal(&self) -> bool {
        self.is_axis_aligned(Direction::S) || self.is_axis_aligned(Direction::T)
    }

    fn min(&self, axis: Direction) -> T {
        debug_assert!(
            self.is_axis_aligned(axis),
            "line {self:?} is aligned along {axis:?}, can not take min"
        );
        T::min(self.0.value_in_dir(axis), self.1.value_in_dir(axis))
    }

    fn max(&self, axis: Direction) -> T {
        debug_assert!(
            self.is_axis_aligned(axis),
            "line {self:?} is aligned along {axis:?}, can not take max"
        );
        T::max(self.0.value_in_dir(axis), self.1.value_in_dir(axis))
    }

    fn axis_coord(&self, axis: Direction) -> T {
        debug_assert!(
            self.is_axis_aligned(axis),
            "line {self:?} is not aligned along {axis:?}"
        );
        match axis {
            Direction::S => self.t0(),
            Direction::T => self.s0(),
        }
    }

    pub fn is_touching<C: UVCoord<T>>(&self, origin: &C) -> bool {
        let axis = self.direction();

        let line_coord = self.axis_coord(axis.opposite());
        let origin_axis_coord = origin.value_in_dir(axis);
        let origin_opposite_coord = origin.value_in_dir(axis.opposite());

        // bounds check
        if origin_opposite_coord < self.min(axis.opposite())
            || origin_opposite_coord > self.max(axis.opposite())
        {
            return false;
        }

        line_coord == origin_axis_coord
    }

    pub fn intersection<C: UVCoord<T>>(
        &self,
        origin: &C,
        axis: Direction,
        positive: bool,
    ) -> Option<C> {
        // Check if the line is collinear with the ray direction (i.e., it's aligned)
        if self.is_axis_aligned(axis) {
            return None; // The line is aligned with the ray, so no intersection
        }

        let line_coord = self.axis_coord(axis.opposite());
        let origin_axis_coord = origin.value_in_dir(axis);
        let origin_opposite_coord = origin.value_in_dir(axis.opposite());

        // check if we are touching the line
        if self.is_touching(origin) {
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
            Direction::S => *intersection_point.s_mut() = line_coord,
            Direction::T => *intersection_point.t_mut() = line_coord,
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
            Line::from_uv_points(&point!($s1, $t1), &point!($s2, $t2))
        };
    }

    #[test]
    fn it_touches_along_t() {
        let line = line!((0, 1), (2, 1));

        // check points on the line
        assert!(!line.is_touching(&point!(-1, 1)));
        assert!(line.is_touching(&point!(0, 1)));
        assert!(line.is_touching(&point!(1, 1)));
        assert!(line.is_touching(&point!(2, 1)));
        assert!(!line.is_touching(&point!(3, 1)));
    }

    #[test]
    fn it_intersects_line_along_t() {
        let line = line!((0, 1), (2, 1));

        // check points along the line
        assert_eq!(
            Some(point!(0, 1)),
            line.intersection(&point!(0, 0), Direction::T, true)
        );
        assert_eq!(
            Some(point!(1, 1)),
            line.intersection(&point!(1, 0), Direction::T, true)
        );
        assert_eq!(
            Some(point!(2, 1)),
            line.intersection(&point!(2, 0), Direction::T, true)
        );

        // check points on the line
        assert_eq!(None, line.intersection(&point!(0, 1), Direction::T, true));
        assert_eq!(None, line.intersection(&point!(1, 1), Direction::T, true));
        assert_eq!(None, line.intersection(&point!(2, 1), Direction::T, true));

        // check past boundaries
        assert_eq!(None, line.intersection(&point!(-1, 0), Direction::T, true));
        assert_eq!(None, line.intersection(&point!(3, 0), Direction::T, true));

        // check backwards ray
        assert_eq!(None, line.intersection(&point!(0, 0), Direction::T, false));
        assert_eq!(None, line.intersection(&point!(1, 0), Direction::T, false));
        assert_eq!(None, line.intersection(&point!(2, 0), Direction::T, false));

        // check backwards ray from other side
        assert_eq!(
            Some(point!(0, 1)),
            line.intersection(&point!(0, 2), Direction::T, false)
        );
        assert_eq!(
            Some(point!(1, 1)),
            line.intersection(&point!(1, 2), Direction::T, false)
        );
        assert_eq!(
            Some(point!(2, 1)),
            line.intersection(&point!(2, 2), Direction::T, false)
        );
    }

    #[test]
    fn it_intersects_line_along_s() {
        let line = line!((1, 0), (1, 2));

        // check points along the line
        assert_eq!(
            Some(point!(1, 0)),
            line.intersection(&point!(0, 0), Direction::S, true)
        );
        assert_eq!(
            Some(point!(1, 1)),
            line.intersection(&point!(0, 1), Direction::S, true)
        );
        assert_eq!(
            Some(point!(1, 2)),
            line.intersection(&point!(0, 2), Direction::S, true)
        );

        // check points on the line
        assert_eq!(None, line.intersection(&point!(1, 0), Direction::S, true));
        assert_eq!(None, line.intersection(&point!(1, 1), Direction::S, true));
        assert_eq!(None, line.intersection(&point!(1, 2), Direction::S, true));

        // check past boundaries
        assert_eq!(None, line.intersection(&point!(0, -1), Direction::S, true));
        assert_eq!(None, line.intersection(&point!(0, 3), Direction::S, true));

        // check backwards ray
        assert_eq!(None, line.intersection(&point!(0, 0), Direction::S, false));
        assert_eq!(None, line.intersection(&point!(0, 1), Direction::S, false));
        assert_eq!(None, line.intersection(&point!(0, 2), Direction::S, false));

        // check backwards ray from other side
        assert_eq!(
            Some(point!(1, 0)),
            line.intersection(&point!(2, 0), Direction::S, false)
        );
        assert_eq!(
            Some(point!(1, 1)),
            line.intersection(&point!(2, 1), Direction::S, false)
        );
        assert_eq!(
            Some(point!(1, 2)),
            line.intersection(&point!(2, 2), Direction::S, false)
        );
    }
}
