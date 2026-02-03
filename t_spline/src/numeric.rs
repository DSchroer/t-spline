use core::fmt::{Debug, Display};
use num_traits::{Bounded, FromPrimitive, Num, NumAssign, Signed};

pub trait Numeric:
    Num + Copy + PartialOrd + Signed + NumAssign + Debug + Display + Bounded + FromPrimitive
{
    fn max(self, other: Self) -> Self {
        if self > other { self } else { other }
    }

    fn min(self, other: Self) -> Self {
        if self < other { self } else { other }
    }
}

impl<T> Numeric for T where
    T: Num + Copy + PartialOrd + Signed + NumAssign + Debug + Display + Bounded + FromPrimitive
{
}

#[cfg(test)]
mod tests {
    use super::*;
    use fixed::types::I10F22;
    use num_traits::One;
    use std::hint::black_box;

    #[test]
    fn it_supports_floats() {
        needs_numeric(1f64);
        needs_numeric(1f32);
    }

    #[test]
    fn it_supports_fixed() {
        needs_numeric(I10F22::one());
    }

    #[test]
    fn it_supports_integers() {
        needs_numeric(0isize);
    }

    fn needs_numeric(value: impl Numeric) {
        black_box(value);
    }
}
