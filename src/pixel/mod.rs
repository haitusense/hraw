pub mod scripting;

extern crate nalgebra as na;
use std::fmt::Display;
use paste::paste;
use na::DMatrix;
use na::Scalar;

// use crate::buffer::HrawToVec;
use crate::StringNumber;
use crate::conversion::HrawToVec;


/*** Pixel ***/

#[derive(Debug)]
pub struct Pixel<T> {
  pub width: usize,
  pub height: usize,
  pub data: Vec<T>
}


/*** index ***/

impl<T> std::ops::Index<usize> for Pixel<T> where T: Clone + Copy + Default {
  type Output = T;
  fn index(&self, index: usize) -> &Self::Output {
    &self.data[index]
  }
}


/*** Cast ***/

pub trait PixelCast {
  type Item : Scalar;
  fn into_pixel(self, width:usize, height:usize) -> Pixel<Self::Item>;
}

// self -> move
macro_rules! impl_cast { ($t: ident) => { paste! {
  impl PixelCast for Vec<$t> {
    type Item = $t;
    fn into_pixel(self, width:usize, height:usize) -> Pixel<Self::Item> {
      Pixel {
        width : width,
        height : height,
        data : self //std::mem::take(self),
      }
    }
  }
  
  // impl PixelCast for [$t] {
  //   type Item = $t;
  //   fn into_pixel(self, width:usize, height:usize) -> Pixel<Self::Item> {
  //     Pixel {
  //       width : width,
  //       height : height,
  //       data : self.to_vec()
  //     }
  //   }
  // }
}}}

impl_cast!(i32);
impl_cast!(f32);
impl_cast!(f64);

pub trait PixelCastHraw {
  type Item : Scalar;
  fn to_pixel<'a, T: Into<StringNumber<'a>>>(&'a mut self, subpath: T) -> anyhow::Result<Pixel<Self::Item>>;
}

impl PixelCastHraw for crate::Hraw {
  type Item = i32;
  fn to_pixel<'a, T: Into<StringNumber<'a>>>(&'a mut self, subpath: T) -> anyhow::Result<Pixel<Self::Item>> {
    let width = self.header.width;
    let height = self.header.height;
    let vec = self.to_vec_i32(subpath)?;
    Ok(Pixel {
      width : width,
      height : height,
      data : vec
    })
  }
}

/*** other ***/

macro_rules! impl_acc { ($t: ident) => { paste! {
  pub fn $t(&mut self) -> f64 {
    let dm = DMatrix::from_row_slice(self.height, self.width, self.data.as_slice()).cast::<f64>();
    dm.$t()
  }
  pub fn [<$t _view>](&mut self, (left, top):(usize, usize), (w, h):(usize, usize)) -> f64 {
    let dm = DMatrix::from_row_slice(self.height, self.width, self.data.as_slice()).cast::<f64>();
    let view = dm.view((top, left), (h, w));
    view.$t()
  }
}}}

impl<T: Scalar + Display + simba::scalar::SubsetOf<f64>> Pixel<T> {
  pub fn to_mat(&mut self) -> DMatrix<T> {
    let dm = DMatrix::from_row_slice(self.height, self.width, self.data.as_slice());
    dm
  }
  impl_acc!{ mean }
  impl_acc!{ max }
  impl_acc!{ min }
  impl_acc!{ norm }

  #[allow(unused)]
  fn test(&mut self, (left, top):(usize, usize), (w, h):(usize, usize), (step_x, step_y):(usize, usize)) -> f64 {
    let dm = DMatrix::from_row_slice(self.height, self.width, self.data.as_slice()).cast::<f64>();
    let view = dm.view_with_steps((top, left), (h, w), (step_x, step_y));
    println!("{}",view);
    0.0
  }
}


#[cfg(test)]
mod test {
  #[test]
  fn pixel_test() {
    extern crate nalgebra as na;

    use super::*;
  
    let mut src = vec![0i32;16*12];

    for y in 0..12 {
      for x in 0..16 {
        src[x + y * 16] = (x + y * 100) as i32;
      }
    }

    let mut pixel = src.into_pixel(16, 12);

    let dm = pixel.to_mat();
    println!("{}",dm);
    let _mean = pixel.test((0, 0), (8, 6), (1, 1));

    let a = dm.view((1,2), (2,3));
    println!("{} {}", a.max(), a.min(), );
    let _res = dm.as_slice();

  }
}
