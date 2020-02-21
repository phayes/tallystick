use num_traits::real::Real;
use num_traits::Num;

/// A trait for numeric types used to count votes.
///
/// Generally seen as the generic `C` in this codebase, this type should be automatically implemented for all numeric types you wish to use.
/// It is used to provide trait specialization so differnetial logic can applied to integer or fractional (float) based vote counting.
///
/// You should pobably not implement this trait. If you have a numeric type that does not implement `Numeric`,
/// you should instead implement [`num_traits::Num`](/num-traits/latest/num_traits/trait.Num.html)
/// (and optionally [`num_traits::real::Real`](/num-traits/latest/num_traits/real/trait.Real.html) for types that support fractions.)
pub trait Numeric {
    /// Get the floor for this numeric type.
    /// For non-fractional types, this just returns self.
    fn floor(self) -> Self;

    /// Does this numeric type support fractional values?
    /// Integer-based types will return false.
    /// Float, or num_rational::Ratio based types should return true.
    fn fraction() -> bool;

    /// Get max upper bound for this numeric type
    /// If this type has no upper bound, return zero
    fn max_value() -> Self;
}

// Default implemention of numeric, assumes everything is an integer (non-fraction).
#[cfg(feature = "nightly")]
impl<T: Num> Numeric for T {
    default fn floor(self) -> Self {
        self
    }
    default fn fraction() -> bool {
        false
    }

    default fn max_value() -> Self {
        T::zero()
    }
}

// Specialize Numeric using Real.
// Real covers all floats, as well as things like num_rational::Ratio and num_rational::BigRational.
impl<T: Num + Real> Numeric for T {
    fn floor(self) -> Self {
        self.floor()
    }
    fn fraction() -> bool {
        true
    }
    fn max_value() -> Self {
        T::max_value()
    }
}

// TODO: no_std: should swap Real for num_traits::float::FloatCore
// TODO: rational: Check that Ratio implements Real
