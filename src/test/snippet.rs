#![allow(dead_code, unused_variables)]
#[cfg(test)]

use std::io::Write;

#[test]
fn hraw_yaml() -> anyhow::Result<()> {

  use crate::Header;

  let test_yaml = r##"
ver      : "0.1"   #
width    : 1024    # [pixel]
height   : 768     # [pixel]
offset   : 64      # [byte]
bitfield : le_i32  # [BitField]
bayer: |           # unused
  R G
  G B
data :             # [raw data body name]
  - 1.raw
  - 2.raw
  - 3.raw
"##;
let test_yaml_2 = r##"
width    : 1024    # [pixel]
height   : 768     # [pixel]
offset   : 64      # [byte]
bitfield : le_i32  # [BitField]
"##;

  let value :serde_json::Value = serde_yaml::from_str(&test_yaml).unwrap();
  println!("value : {:?}", value);

  let obj: Header = serde_json::from_value(value).unwrap();
  println!("obj : {:?}", obj);

  let obj: Header = serde_yaml::from_str(test_yaml_2).unwrap();
  println!("obj : {:?}", obj);

  // let dst :BitField = serde_json::from_value(value["bitfield"].to_owned()).unwrap();
  // println!("dst : {:?}", dst);
  Ok(())
}

#[test]
fn simple_clamp() {
  use crate::rawnumber::ClampFrom;

  let dst = i32::clamp_from(3u8);
  println!("{:?}", dst);
  let dst = i32::clamp_from(i16::MAX);
  println!("{:?}", dst);

  println!("{} {}", i64::MAX, i64::MAX as f64);

}

#[test]
fn write_stream() -> anyhow::Result<()> {
  let vec = vec![0u8;256];
  let cur = std::io::Cursor::new(vec);
  let mut sw = std::io::BufWriter::new(cur); /* vec u8じゃないと使えない */
  for i in 0..=255u8 {
    let buf = [i];
    let _ = sw.write(&buf);
  }
  let _ = sw.flush();
  let vec = sw.get_ref().get_ref(); /* 所有権を取り戻す */
  println!("{:?}", &vec[0..4]);
  Ok(())
}

#[test]
fn nalgebra_test() {
  extern crate nalgebra as na;
  use na::DMatrix;

  let m = na::Matrix3::new(
    0, 0, 0,
    0, 1, 0,
    0, 0, 0
  );
  let src = vec![1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16];
  let dm = DMatrix::from_vec(4, 4, src);
  println!("{}", m);
  println!("{}", dm);
  println!("{}", dm.row(1));
  println!("{}", dm.transpose());
  println!("slice {}", dm.view((0, 0), (3, 3)));
  println!("dot {}", dm.view((0, 0), (3, 3)).dot(&m));
  println!("(x, y) = (0, 0) {} {}", dm.view_range(0, 0), dm.view_range(0, 0).to_scalar());

  for y in 0..4 {
    for x in 0..4 {
      println!("{}", dm.view_range(x, y).to_scalar());
    }  
  }

  for y in 1..3 {
    for x in 1..3 {
      let slice = dm.view((y-1, x-1), (3, 3));
      println!("{}", slice);
    }  
  }
}


#[allow(unused)]
mod deprecation_processing {
  use image::codecs::png::PngEncoder;

  trait CustomArrayConverter {
    fn to_a(&self, index:usize, stride:usize, color:i32) -> (i32, i32, i32);
  }
  
  impl CustomArrayConverter for Vec<i32> {
    #[inline(always)]
    fn to_a(&self, index:usize, stride:usize, color:i32) -> (i32, i32, i32) {
      let y = (index / stride) % 2;
      let x = index % 2;
      
      let dst = match (color, x, y) {
        (0, _, _) => (self[index], self[index], self[index]),
        (_, 0, 0) => {
          let r = self[index - 1] + self[index + 1];
          let g = self[index];
          let b = self[index - stride] + self[index + stride];
          (r / 2, g, b / 2)
        }
        (_, 1, 0) => {
          let r = self[index];
          let g = self[index - 1] + self[index + 1] + self[index - stride] + self[index + stride];
          let b = self[index - stride - 1] + self[index - stride + 1] + self[index + stride - 1] + self[index + stride + 1];
          (r, g / 4, b / 4)
        }
        (_, 0, 1) => {
          let r = self[index - stride - 1] + self[index - stride + 1] + self[index + stride - 1] + self[index + stride + 1];
          let g = self[index - 1] + self[index + 1] + self[index - stride] + self[index + stride];
          let b = self[index];
          (r / 4, g / 4, b)
        }
        (_, 1, 1) => {
          let r = self[index - stride] + self[index +stride];
          let g = self[index];
          let b = self[index - 1] + self[index + 1];
          (r / 2, g, b / 2)
        },
        (_, _, _) => (self[index], self[index], self[index]),
      };
      dst
    }
   }
  
  pub fn dummy(){
    let mut img = image::RgbImage::new(512, 512);
    let rgb = 2;
    for (x, y, pixel) in img.enumerate_pixels_mut() { 
      *pixel = match ((x / 10) % 2, (y / 10) % 2) {
        (1, 0) => image::Rgb([rgb, rgb, rgb]),
        (0, 1) => image::Rgb([rgb, rgb, rgb]),
        _=> image::Rgb([0, 0, 0]),
      }
    }
    let mut writer = Vec::new();
    let encoder = PngEncoder::new(&mut writer);
    img.write_with_encoder(encoder).unwrap();
  }

}

#[allow(unused)]
mod deprecation_stream {
  use std::io::BufReader;
  use std::io::BufWriter;
  use std::io::Read;
  use std::io::Write;
  
  pub fn stream_to_stream<T, U>(sr:&mut BufReader<T>, sw:&mut BufWriter<U>) -> anyhow::Result<()>
  where T: ?Sized + std::io::Read, U: ?Sized + std::io::Write {
    let mut buf : [u8; 4] = unsafe { std::mem::MaybeUninit::zeroed().assume_init() };
    for _i in 0..640*480 {
      sr.read_exact(&mut buf)?;
      
      let value = i32::from_le_bytes(buf) as u16;
      let dst = value.to_le_bytes();
      sw.write(&dst).unwrap();
    }
    sw.flush().unwrap();
    Ok(())
  }
}
