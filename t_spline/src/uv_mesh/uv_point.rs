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

#[derive(Debug, Clone, PartialEq)]
pub struct UVPoint {
    pub s: isize,
    pub t: isize,
    pub outgoing_edge: EdgeID,
}

impl UVCoord for UVPoint {
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

impl UVCoord for (isize, isize) {
    fn s(&self) -> isize {
        self.0
    }

    fn s_mut(&mut self) -> &mut isize {
        &mut self.0
    }

    fn t(&self) -> isize {
        self.1
    }

    fn t_mut(&mut self) -> &mut isize {
        &mut self.1
    }
}

pub trait UVCoord: Clone {
    fn s(&self) -> isize;
    fn s_mut(&mut self) -> &mut isize;

    fn t(&self) -> isize;
    fn t_mut(&mut self) -> &mut isize;

    fn value_in_dir(&self, direction: Direction) -> isize {
        match direction {
            Direction::S => self.s(),
            Direction::T => self.t(),
        }
    }

    fn add_in_dir(&mut self, direction: Direction, value: isize) {
        match direction {
            Direction::S => *self.s_mut() += value,
            Direction::T => *self.t_mut() += value,
        }
    }
}
