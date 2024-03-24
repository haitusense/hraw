extern crate nalgebra as na;
use rayon::prelude::*;
use num::Integer;
use image::codecs::png::PngEncoder;


const M_G_INT : na::Matrix3<i32> = na::Matrix3::new(
  1, 0, 1,
  0, 4, 0,
  1, 0, 1
);
const M_G_OUT : na::Matrix3<i32> = na::Matrix3::new(
  0, 1, 0,
  1, 0, 1,
  0, 1, 0
);
const M_C_INT : na::Matrix3<i32> = na::Matrix3::new(
  0, 0, 0,
  0, 1, 0,
  0, 0, 0
);
const M_C_OUT : na::Matrix3<i32> = na::Matrix3::new(
  1, 0, 1,
  0, 0, 0,
  1, 0, 1
);
const M_C_ROW : na::Matrix3<i32> = na::Matrix3::new(
  0, 0, 0,
  1, 0, 1,
  0, 0, 0
);
const M_C_COL : na::Matrix3<i32> = na::Matrix3::new(
  0, 1, 0,
  0, 0, 0,
  0, 1, 0
);


trait CustomConverter {
  fn to_u8_sat(self) -> u8;
  fn bitshift(self, shift:i32) -> i32;
}

impl CustomConverter for i32 {
  #[inline(always)]
  fn to_u8_sat(self) -> u8 {
    num::clamp(self, u8::MIN as i32, u8::MAX as i32) as u8
  }
  #[inline(always)]
  fn bitshift(self, shift:i32) -> i32 {
    if shift > 0 { self >> shift } else { self << -1*shift }
  }
}

trait Bayer {
  fn to_mono(self) -> image::Rgb<u8>;
}

impl Bayer for u8 {
  #[inline(always)]
  fn to_mono(self) -> image::Rgb<u8> { image::Rgb([self, self, self]) }
}

#[inline(always)]
fn bayer_postion(slice:na::Matrix<i32, na::Dyn, na::Dyn, na::ViewStorage<'_, i32, na::Dyn, na::Dyn, na::Const<1>, na::Dyn>>, x:bool, y:bool) -> (i32, i32, i32) {
  match (x, y) {
    (true, true) => (slice.dot(&M_C_COL) / 2, slice.dot(&M_G_INT) / 8, slice.dot(&M_C_ROW) / 2),
    (true, false) => (slice.dot(&M_C_OUT) / 4, slice.dot(&M_G_OUT) / 4, slice.dot(&M_C_INT)    ),
    (false, true) => (slice.dot(&M_C_INT)    , slice.dot(&M_G_OUT) / 4, slice.dot(&M_C_OUT) / 4),
    (false, false) => (slice.dot(&M_C_ROW) / 2, slice.dot(&M_G_INT) / 8, slice.dot(&M_C_COL) / 2),
    // _=> panic!("what??")
  }
}

pub fn slice_to_png (src: &[i32], width:usize, height:usize, bitshift:i32, color:i32) -> Vec<u8> {
  let _size = width * height;
  let _stride = width;

  let mut img = image::RgbImage::new(width as u32, height as u32);
  let dm = na::DMatrix::from_column_slice(width, height, src);
  // from_vec -> from_column_slice

  let (offset_x, offset_y) = match color {
    1 | 5 => (0, 0),
    2 | 6 => (1, 0),
    3 | 7 => (0, 1),
    4 | 8 => (1, 1),
    _=> (0, 0)
  };
 
  img.enumerate_pixels_mut()
    .collect::<Vec<(u32, u32, &mut image::Rgb<u8>)>>()
    .par_iter_mut()
    .for_each(|(x, y, pixel)| {
      **pixel = match (color, *x as usize, *y as usize) {
        (1..=4, x, y) if 0 < x && x < width as usize - 1 && 0 < y && y < height as usize - 1 => {
          let slice = dm.view((x - 1, y - 1), (3, 3));
          let dst = bayer_postion(slice, (x + offset_x).is_odd(), (y + offset_y).is_odd());
          image::Rgb([dst.0.bitshift(bitshift).to_u8_sat(), dst.1.bitshift(bitshift).to_u8_sat(), dst.2.bitshift(bitshift).to_u8_sat()])
        },
        (5..=8, x, y) => dm.view_range((x / 2) * 2 + offset_x, (y / 2) * 2 + offset_y).to_scalar().bitshift(bitshift).to_u8_sat().to_mono(),        
        (_, x, y) => dm.view_range(x, y).to_scalar().bitshift(bitshift).to_u8_sat().to_mono(),          
      }
    });
  let mut writer = Vec::new();
  img.write_with_encoder(PngEncoder::new(&mut writer)).unwrap();
  
  writer
}
