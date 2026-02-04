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

    fn delta() -> Self;
}

macro_rules! impl_numeric_float {
    ($($t:ty),*) => {
        $(
            impl Numeric for $t {
                fn delta() -> Self { <$t>::EPSILON }
            }
        )*
    }
}

impl_numeric_float!(f32, f64);

#[cfg(feature = "fixed")]
mod fixed_impl {
    use super::*;

    macro_rules! impl_numeric_fixed {
        ($($t:ident),*) => {
            $(
                impl<Frac: fixed::types::extra::LeEqU32> Numeric for fixed::$t<Frac>
                where
                    fixed::$t<Frac>: fixed::traits::FixedSigned + Num + Signed + NumAssign + FromPrimitive + Bounded
                {
                    fn delta() -> Self { Self::DELTA }
                }
            )*
        }
    }

    impl_numeric_fixed!(FixedI8, FixedI16, FixedI32, FixedI64, FixedI128);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::hint::black_box;

    #[test]
    fn it_supports_floats() {
        needs_numeric(1f64);
        needs_numeric(1f32);
    }

    #[cfg(feature = "fixed")]
    #[test]
    fn it_supports_fixed() {
        use num_traits::One;

        needs_numeric(fixed::types::I10F22::one());
    }

    fn needs_numeric(value: impl Numeric) {
        black_box(value);
    }
}
