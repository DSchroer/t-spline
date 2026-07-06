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
use crate::uv_mesh::ids::EdgeID;
use nalgebra::{Scalar, Vector2};
use num_traits::NumAssign;

#[derive(Debug, Clone, PartialEq)]
pub struct UVPoint {
    pub s: isize,
    pub t: isize,
    pub outgoing_edge: EdgeID,
}

impl UVPoint {
    pub fn st(&self) -> (isize, isize) {
        (self.s, self.t)
    }
}

impl<T: Scalar + Copy + NumAssign> UVCoord<T> for Vector2<T> {
    fn s(&self) -> T {
        self.x
    }

    fn s_mut(&mut self) -> &mut T {
        &mut self.x
    }

    fn t(&self) -> T {
        self.y
    }

    fn t_mut(&mut self) -> &mut T {
        &mut self.y
    }
}

impl UVCoord<isize> for UVPoint {
    fn s(&self) -> isize {
        self.s
    }

    fn s_mut(&mut self) -> &mut isize {
        &mut self.s
    }

    fn t(&self) -> isize {
        self.t
    }

    fn t_mut(&mut self) -> &mut isize {
        &mut self.t
    }
}

impl<T: Scalar + Copy + NumAssign> UVCoord<T> for (T, T) {
    fn s(&self) -> T {
        self.0
    }

    fn s_mut(&mut self) -> &mut T {
        &mut self.0
    }

    fn t(&self) -> T {
        self.1
    }

    fn t_mut(&mut self) -> &mut T {
        &mut self.1
    }
}

pub trait UVCoord<T: NumAssign>: Clone {
    fn s(&self) -> T;
    fn s_mut(&mut self) -> &mut T;

    fn t(&self) -> T;
    fn t_mut(&mut self) -> &mut T;

    fn value_in_dir(&self, direction: Direction) -> T {
        match direction {
            Direction::S => self.s(),
            Direction::T => self.t(),
        }
    }

    fn add_in_dir(&mut self, direction: Direction, value: T) {
        match direction {
            Direction::S => *self.s_mut() += value,
            Direction::T => *self.t_mut() += value,
        }
    }
}
