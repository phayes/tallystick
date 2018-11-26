use num_traits::int::PrimInt;
use num_traits::float::FloatCore;
use num_traits::Num;
use num_traits::NumCast;
use std::ops::AddAssign;
use num_traits::real::Real;

pub trait Numeric {
  fn floor(self) -> Self;
  fn fraction() -> bool;
}

impl<T> Numeric for T {
  default fn floor(self) -> Self {
    self
  }
  default fn fraction() -> bool {
    false
  }
}

// TODO: no_std: should swap Real for FloatCore
// TODO: rational: Check that Ratio implements Real
impl<T> Numeric for T
where
  T: Real,
{
  fn floor(self) -> Self {
    self.floor()
  }
  fn fraction() -> bool {
    true
  }
}
