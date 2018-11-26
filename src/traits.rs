use num_traits::float::FloatCore;
use num_traits::Num;
use num_traits::real::Real;

/// A trait for numeric types used to count votes. 
/// This type should be automatically implemented for all numeric types you wish to use.
/// It is used to provide trait specialization so differnetial logic can applied to 
/// integer or fractional (float) based vote counting.
pub trait Numeric {

  /// Get the floor for this numeric type. 
  /// For non-fractional types, this just returns self.
  fn floor(self) -> Self;

  /// Does this numeric type support fractional values?
  /// Integer-based types will return false.
  /// Float, or num_rational::Ratio based types should return true. 
  fn fraction() -> bool;
}

// Default implemention of numeric, assumes everything is an integer (non-fraction).
impl<T: Num> Numeric for T {
  default fn floor(self) -> Self {
    self
  }
  default fn fraction() -> bool {
    false
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
}

// TODO: no_std: should swap Real for FloatCore
// TODO: rational: Check that Ratio implements Real