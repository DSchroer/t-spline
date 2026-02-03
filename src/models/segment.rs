use std::ops::Sub;

#[derive(Debug, Copy, Clone)]
pub struct ParamPoint {
    /// Horizontal UV coordinate
    pub s: f64,
    /// Vertical UV coordinate
    pub t: f64,
}

impl ParamPoint {
    fn cross(&self, rhs: &Self) -> f64 {
        self.s * rhs.t - self.t * rhs.s
    }

    fn orient(a: ParamPoint, b: ParamPoint, c: ParamPoint) -> f64 {
        (b - a).cross(&(c - a))
    }

    fn on_segment(p: ParamPoint, a: ParamPoint, b: ParamPoint) -> bool {
        p.s <= f64::max(a.s, b.s)
            && p.s >= f64::min(a.s, b.s)
            && p.t <= f64::max(a.t, b.t)
            && p.t >= f64::min(a.t, b.t)
    }
}

#[derive(Debug)]
pub struct Segment {
    pub start: ParamPoint,
    pub end: ParamPoint,
}

impl Sub for ParamPoint {
    type Output = ParamPoint;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::Output {
            s: self.s - rhs.s,
            t: self.t - rhs.t,
        }
    }
}

impl Segment {
    /// Determine if two segments intersect.
    pub fn intersects(&self, other: &Segment) -> bool {
        let oa = ParamPoint::orient(other.start, other.end, self.start);
        let ob = ParamPoint::orient(other.start, other.end, self.end);
        let oc = ParamPoint::orient(self.start, self.end, other.start);
        let od = ParamPoint::orient(self.start, self.end, other.end);

        // General case: segments cross each other, not touching.
        if oa * ob < 0.0 && oc * od < 0.0 {
            return true;
        }

        // Special Cases: segments are collinear or touching at an endpoint.
        // Check if self.start lies on the 'other' segment.
        if oa.abs() < 1e-9 && ParamPoint::on_segment(self.start, other.start, other.end) {
            return true;
        }

        // Check if self.end lies on the 'other' segment.
        if ob.abs() < 1e-9 && ParamPoint::on_segment(self.end, other.start, other.end) {
            return true;
        }

        // Check if other.start lies on the 'self' segment.
        if oc.abs() < 1e-9 && ParamPoint::on_segment(other.start, self.start, self.end) {
            return true;
        }

        // Check if other.end lies on the 'self' segment.
        if od.abs() < 1e-9 && ParamPoint::on_segment(other.end, self.start, self.end) {
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
