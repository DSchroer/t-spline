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

use crate::Numeric;
use core::ops::Sub;

#[derive(Debug, Copy, Clone)]
pub struct ParamPoint<T> {
    /// Horizontal UV coordinate
    pub s: T,
    /// Vertical UV coordinate
    pub t: T,
}

impl<T: Numeric> ParamPoint<T> {
    fn cross(&self, rhs: &Self) -> T {
        self.s * rhs.t - self.t * rhs.s
    }

    fn orient(a: ParamPoint<T>, b: ParamPoint<T>, c: ParamPoint<T>) -> T {
        (b - a).cross(&(c - a))
    }

    fn on_segment(p: ParamPoint<T>, a: ParamPoint<T>, b: ParamPoint<T>) -> bool {
        let s_max = if a.s >= b.s { a.s } else { b.s };
        let s_min = if a.s <= b.s { a.s } else { b.s };
        let t_max = if a.t >= b.t { a.t } else { b.t };
        let t_min = if a.t <= b.t { a.t } else { b.t };

        p.s <= s_max && p.s >= s_min && p.t <= t_max && p.t >= t_min
    }
}

#[derive(Debug)]
pub struct Segment<T> {
    pub start: ParamPoint<T>,
    pub end: ParamPoint<T>,
}

impl<T: Sub<Output = T>> Sub for ParamPoint<T> {
    type Output = ParamPoint<T>;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::Output {
            s: self.s - rhs.s,
            t: self.t - rhs.t,
        }
    }
}

impl<T: Numeric> Segment<T> {
    /// Determine if two segments intersect.
    pub fn intersects(&self, other: &Segment<T>) -> bool {
        let oa = ParamPoint::orient(other.start, other.end, self.start);
        let ob = ParamPoint::orient(other.start, other.end, self.end);
        let oc = ParamPoint::orient(self.start, self.end, other.start);
        let od = ParamPoint::orient(self.start, self.end, other.end);

        // General case: segments cross each other, not touching.
        if oa * ob < T::zero() && oc * od < T::zero() {
            return true;
        }

        // Special Cases: segments are collinear or touching at an endpoint.
        // Check if self.start lies on the 'other' segment.
        if oa.abs() < T::from_f32(1e-9).unwrap()
            && ParamPoint::on_segment(self.start, other.start, other.end)
        {
            return true;
        }

        // Check if self.end lies on the 'other' segment.
        if ob.abs() < T::from_f32(1e-9).unwrap()
            && ParamPoint::on_segment(self.end, other.start, other.end)
        {
            return true;
        }

        // Check if other.start lies on the 'self' segment.
        if oc.abs() < T::from_f32(1e-9).unwrap()
            && ParamPoint::on_segment(other.start, self.start, self.end)
        {
            return true;
        }

        // Check if other.end lies on the 'self' segment.
        if od.abs() < T::from_f32(1e-9).unwrap()
            && ParamPoint::on_segment(other.end, self.start, self.end)
        {
            return true;
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_detects_intersections() {
        // Helper to create points easily
        let p = |s: f64, t: f64| ParamPoint { s, t };
        // Helper to create segments
        let seg = |s1, t1, s2, t2| Segment {
            start: p(s1, t1),
            end: p(s2, t2),
        };

        let cases = vec![
            // 1. Identical segments (Collinear overlap)
            (seg(0., 0., 1., 0.), seg(0., 0., 1., 0.)),
            // 2. Basic Crossing (X-shape)
            (seg(0., 0., 2., 2.), seg(0., 2., 2., 0.)),
            // 3. T-Junction (One ends on the other)
            (seg(0., 0., 2., 0.), seg(1., 0., 1., 2.)),
            // 4. Shared Endpoint (V-shape)
            // Note: Check your logic; usually touching endpoints counts as intersection
            (seg(0., 0., 1., 1.), seg(1., 1., 2., 0.)),
            // 5. Partial Overlap (Collinear)
            (seg(0., 0., 2., 0.), seg(1., 0., 3., 0.)),
        ];
        for (a, b) in cases {
            assert!(a.intersects(&b), "{a:?} failed to intersect {b:?}");
        }
    }

    #[test]
    fn it_detects_non_intersections() {
        // Helper to create points easily
        let p = |s: f64, t: f64| ParamPoint { s, t };
        // Helper to create segments
        let seg = |s1, t1, s2, t2| Segment {
            start: p(s1, t1),
            end: p(s2, t2),
        };

        let cases = vec![
            // Parallel lines (distinct)
            (seg(0., 0., 1., 0.), seg(0., 1., 1., 1.)),
            // Collinear but Disjoint
            (seg(0., 0., 1., 0.), seg(2., 0., 3., 0.)),
            // Separated in space (General disjoint)
            (seg(0., 0., 1., 1.), seg(2., 2., 3., 2.)),
        ];
        for (a, b) in cases {
            assert!(!a.intersects(&b), "{a:?} detected intersection with {b:?}");
        }
    }
}
