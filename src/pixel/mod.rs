extern crate nalgebra as na;
use std::fmt::Display;
use paste::paste;
use na::DMatrix;
use na::Scalar;

pub struct Pixel<'a, T: Scalar> {
  width: usize,
  height: usize,
  src: &'a mut [T]
}

macro_rules! impl_acc { ($t: ident) => { paste! {
  pub fn $t(&mut self) -> f64 {
    let dm = DMatrix::from_row_slice(self.height, self.width, self.src).cast::<f64>();
    dm.$t()
  }
  pub fn [<$t _view>](&mut self, (left, top):(usize, usize), (w, h):(usize, usize)) -> f64 {
    let dm = DMatrix::from_row_slice(self.height, self.width, self.src).cast::<f64>();
    let view = dm.view((top, left), (h, w));
    view.$t()
  }
}}}

impl<'a, T: Scalar + Display + simba::scalar::SubsetOf<f64>> Pixel<'a, T> {
  pub fn to_mat(&mut self) -> DMatrix<T> {
    let dm = DMatrix::from_row_slice(self.height, self.width, self.src);
    dm
  }
  impl_acc!{ mean }
  impl_acc!{ max }
  impl_acc!{ min }
  impl_acc!{ norm }

  fn test(&mut self, (left, top):(usize, usize), (w, h):(usize, usize), (step_x, step_y):(usize, usize)) -> f64 {
    let dm = DMatrix::from_row_slice(self.height, self.width, self.src).cast::<f64>();
    let view = dm.view_with_steps((top, left), (h, w), (step_x, step_y));
    println!("{}",view);
    0.0
  }
}

pub trait PixelSlice {
  type Item : Scalar;
  fn to_pixel<'a>(&'a mut self, width:usize, height:usize) -> Pixel<'a, Self::Item>;
}

impl PixelSlice for [i32] {
  type Item = i32;
  fn to_pixel<'a>(&'a mut self, width:usize, height:usize) -> Pixel<'a, Self::Item> {
    Pixel {
      width : width,
      height : height,
      src : self
    }
  }
}

#[cfg(test)]
mod test {
  #[test]
  fn pixel_test() {
    extern crate nalgebra as na;
    use na::DMatrix;
    use super::*;
  
    let mut src = vec![0i32;16*12];

    for y in 0..12 {
      for x in 0..16 {
        src[x + y * 16] = (x + y * 100) as i32;
      }
    }

    let mut pixel = src.to_pixel(16, 12);

    let dm = pixel.to_mat();
    println!("{}",dm);
    let mean = pixel.test((0, 0), (8, 6), (1, 1));

    let a = dm.view((1,2), (2,3));
    println!("{} {}", a.max(), a.min(), );
    let res = dm.as_slice();


  }
}