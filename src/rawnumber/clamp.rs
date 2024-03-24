/* modify based on https://doc.rust-lang.org/src/core/convert/num.rs.html */

pub trait ClampFrom<T>: Sized {
  fn clamp_from(value: T) -> Self;
}
macro_rules! impl_from {
  ($Small: ty, $Large: ty) => {
    impl ClampFrom<$Small> for $Large {
      #[inline(always)]
      fn clamp_from(small: $Small) -> Self {
        small as Self
      }
    }
  };
}

// Unsigned -> Unsigned
impl_from! { u8, u16 }
impl_from! { u8, u32 }
impl_from! { u8, u64 }
impl_from! { u8, u128 }
impl_from! { u8, usize }
impl_from! { u16, u32 }
impl_from! { u16, u64 }
impl_from! { u16, u128 }
impl_from! { u32, u64 }
impl_from! { u32, u128 }
impl_from! { u64, u128 }

// Signed -> Signed
impl_from! { i8, i16 }
impl_from! { i8, i32 }
impl_from! { i8, i64 }
impl_from! { i8, i128 }
impl_from! { i8, isize }
impl_from! { i16, i32 }
impl_from! { i16, i64 }
impl_from! { i16, i128 }
impl_from! { i32, i64 }
impl_from! { i32, i128 }
impl_from! { i64, i128 }

// Unsigned -> Signed
impl_from! { u8, i16 }
impl_from! { u8, i32 }
impl_from! { u8, i64 }
impl_from! { u8, i128 }
impl_from! { u16, i32 }
impl_from! { u16, i64 }
impl_from! { u16, i128 }
impl_from! { u32, i64 }
impl_from! { u32, i128 }
impl_from! { u64, i128 }

impl_from! { u16, usize }
impl_from! { u8, isize }
impl_from! { i16, isize }

// Signed -> Float
// impl_from! { i8, f32 }
// impl_from! { i8, f64 }
// impl_from! { i16, f32 }
// impl_from! { i16, f64 }
// impl_from! { i32, f64 }

// Unsigned -> Float
// impl_from! { u8, f32 }
// impl_from! { u8, f64 }
// impl_from! { u16, f32 }
// impl_from! { u16, f64 }
// impl_from! { u32, f64 }

// Float -> Float
impl_from! { f32, f64 }

// [Additions] same, to float
impl_from! { u8, u8 }
impl_from! { u16, u16 }
impl_from! { u32, u32 }
impl_from! { u64, u64 }
impl_from! { u128, u128 }
impl_from! { i8, i8 }
impl_from! { i16, i16 }
impl_from! { i32, i32 }
impl_from! { i64, i64 }
impl_from! { i128, i128 }
impl_from! { usize, usize }
impl_from! { isize, isize }
impl_from! { f32, f32 }
impl_from! { f64, f64 }

impl_from! { u8, f32 }
impl_from! { u16, f32 }
impl_from! { u32, f32 }
impl_from! { u64, f32 }
impl_from! { u128, f32 }
impl_from! { i8, f32 }
impl_from! { i16, f32 }
impl_from! { i32, f32 }
impl_from! { i64, f32 }
impl_from! { i128, f32 }
impl_from! { u8, f64 }
impl_from! { u16, f64 }
impl_from! { u32, f64 }
impl_from! { u64, f64 }
impl_from! { u128, f64 }
impl_from! { i8, f64 }
impl_from! { i16, f64 }
impl_from! { i32, f64 }
impl_from! { i64, f64 }
impl_from! { i128, f64 }
impl_from! { f64, f32 }

macro_rules! clamp_from_unbounded {
  ($source:ty, $($target:ty),*) => {$(
    impl ClampFrom<$source> for $target {
      #[inline]
      fn clamp_from(value: $source) -> Self {
        value as Self
      }
    }
  )*}
}

macro_rules! clamp_from_lower_bounded {
  ($source:ty, $($target:ty),*) => {$(
    impl ClampFrom<$source> for $target {
      #[inline]
      fn clamp_from(u: $source) -> Self {
        if u >= 0 {
          u as Self
        } else {
          0
        }
      }
    }
  )*}
}

macro_rules! clamp_from_upper_bounded {
  ($source:ty, $($target:ty),*) => {$(
    impl ClampFrom<$source> for $target {
      #[inline]
      fn clamp_from(u: $source) -> Self {
        if u > (Self::MAX as $source) {
          Self::MAX
        } else {
          u as Self
        }
      }
    }
  )*}
}

macro_rules! clamp_from_both_bounded {
  ($source:ty, $($target:ty),*) => {$(
    impl ClampFrom<$source> for $target {
      #[inline]
      fn clamp_from(u: $source) -> Self {
        let min = Self::MIN as $source;
        let max = Self::MAX as $source;
        if u < min {
          Self::MIN
        } else if u > max {
          Self::MAX
        } else {
          u as Self
        }
      }
    }
  )*}
}

macro_rules! rev {
  ($mac:ident, $source:ty, $($target:ty),*) => {$(
    $mac!($target, $source);
  )*}
}

// intra-sign conversions
clamp_from_upper_bounded!(u16, u8);
clamp_from_upper_bounded!(u32, u16, u8);
clamp_from_upper_bounded!(u64, u32, u16, u8);
clamp_from_upper_bounded!(u128, u64, u32, u16, u8);

clamp_from_both_bounded!(i16, i8);
clamp_from_both_bounded!(i32, i16, i8);
clamp_from_both_bounded!(i64, i32, i16, i8);
clamp_from_both_bounded!(i128, i64, i32, i16, i8);

// unsigned-to-signed
clamp_from_upper_bounded!(u8, i8);
clamp_from_upper_bounded!(u16, i8, i16);
clamp_from_upper_bounded!(u32, i8, i16, i32);
clamp_from_upper_bounded!(u64, i8, i16, i32, i64);
clamp_from_upper_bounded!(u128, i8, i16, i32, i64, i128);

// signed-to-unsigned
clamp_from_lower_bounded!(i8, u8, u16, u32, u64, u128);
clamp_from_lower_bounded!(i16, u16, u32, u64, u128);
clamp_from_lower_bounded!(i32, u32, u64, u128);
clamp_from_lower_bounded!(i64, u64, u128);
clamp_from_lower_bounded!(i128, u128);
clamp_from_both_bounded!(i16, u8);
clamp_from_both_bounded!(i32, u16, u8);
clamp_from_both_bounded!(i64, u32, u16, u8);
clamp_from_both_bounded!(i128, u64, u32, u16, u8);

// usize/isize
clamp_from_upper_bounded!(usize, isize);
clamp_from_lower_bounded!(isize, usize);

#[cfg(target_pointer_width = "16")]
mod ptr_clamp_from_impls {
  use super::ClampFrom;

  clamp_from_upper_bounded!(usize, u8);
  clamp_from_unbounded!(usize, u16, u32, u64, u128);
  clamp_from_upper_bounded!(usize, i8, i16);
  clamp_from_unbounded!(usize, i32, i64, i128);

  clamp_from_both_bounded!(isize, u8);
  clamp_from_lower_bounded!(isize, u16, u32, u64, u128);
  clamp_from_both_bounded!(isize, i8);
  clamp_from_unbounded!(isize, i16, i32, i64, i128);

  rev!(clamp_from_upper_bounded, usize, u32, u64, u128);
  rev!(clamp_from_lower_bounded, usize, i8, i16);
  rev!(clamp_from_both_bounded, usize, i32, i64, i128);

  rev!(clamp_from_upper_bounded, isize, u16, u32, u64, u128);
  rev!(clamp_from_both_bounded, isize, i32, i64, i128);
}

#[cfg(target_pointer_width = "32")]
mod ptr_clamp_from_impls {
  use super::ClampFrom;

  clamp_from_upper_bounded!(usize, u8, u16);
  clamp_from_unbounded!(usize, u32, u64, u128);
  clamp_from_upper_bounded!(usize, i8, i16, i32);
  clamp_from_unbounded!(usize, i64, i128);

  clamp_from_both_bounded!(isize, u8, u16);
  clamp_from_lower_bounded!(isize, u32, u64, u128);
  clamp_from_both_bounded!(isize, i8, i16);
  clamp_from_unbounded!(isize, i32, i64, i128);

  rev!(clamp_from_unbounded, usize, u32);
  rev!(clamp_from_upper_bounded, usize, u64, u128);
  rev!(clamp_from_lower_bounded, usize, i8, i16, i32);
  rev!(clamp_from_both_bounded, usize, i64, i128);

  rev!(clamp_from_unbounded, isize, u16);
  rev!(clamp_from_upper_bounded, isize, u32, u64, u128);
  rev!(clamp_from_unbounded, isize, i32);
  rev!(clamp_from_both_bounded, isize, i64, i128);
}

#[cfg(target_pointer_width = "64")]
mod ptr_clamp_from_impls {
  use super::ClampFrom;

  clamp_from_upper_bounded!(usize, u8, u16, u32);
  clamp_from_unbounded!(usize, u64, u128);
  clamp_from_upper_bounded!(usize, i8, i16, i32, i64);
  clamp_from_unbounded!(usize, i128);

  clamp_from_both_bounded!(isize, u8, u16, u32);
  clamp_from_lower_bounded!(isize, u64, u128);
  clamp_from_both_bounded!(isize, i8, i16, i32);
  clamp_from_unbounded!(isize, i64, i128);

  rev!(clamp_from_unbounded, usize, u32, u64);
  rev!(clamp_from_upper_bounded, usize, u128);
  rev!(clamp_from_lower_bounded, usize, i8, i16, i32, i64);
  rev!(clamp_from_both_bounded, usize, i128);

  rev!(clamp_from_unbounded, isize, u16, u32);
  rev!(clamp_from_upper_bounded, isize, u64, u128);
  rev!(clamp_from_unbounded, isize, i32, i64);
  rev!(clamp_from_both_bounded, isize, i128);
}

// [Additions]
clamp_from_both_bounded!(f32, u8, u16, u32, u64, u128);
clamp_from_both_bounded!(f32, i8, i16, i32, i64, i128);
clamp_from_both_bounded!(f64, u8, u16, u32, u64, u128);
clamp_from_both_bounded!(f64, i8, i16, i32, i64, i128);