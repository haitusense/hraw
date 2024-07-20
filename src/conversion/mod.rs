use crate::*;
use anyhow::Context;
use paste::paste;

// use std::io::{BufReader, Bytes, Seek};

/* path to png */

pub fn read_png_head(path:&str) -> anyhow::Result<(usize, usize)> {
  use image::io::Reader as ImageReader;
  let img = ImageReader::open(path)?.decode()?;
  let width = img.width() as usize;
  let height = img.height() as usize;
  Ok((width, height))
}

pub fn read_png_i32(path:&str) -> anyhow::Result<(usize, usize, Vec<i32>)> {
  use image::io::Reader as ImageReader;
  let img = ImageReader::open(path)?.decode()?;

  let width = img.width() as usize;
  let height = img.height() as usize;
  let mut vec = vec![0i32; width*height];

  img.into_rgba8().enumerate_pixels()
  .for_each(|(x, y, pixel)| {
    let index = x as usize + y as usize * width;
    vec[index] = i32::from_be_bytes(pixel.0);
  });

  Ok((width, height, vec))
}


/* hraw to vec */

pub trait HrawToVec {
  fn to_vec_i32<'a, 'b, T>(&'a mut self, subpath:T) -> anyhow::Result<Vec<i32>> where T: Into<StringNumber<'b>>, 'a : 'b;
  fn to_vec_f32<'a, 'b, T>(&'a mut self, subpath:T) -> anyhow::Result<Vec<f32>> where T: Into<StringNumber<'b>>, 'a : 'b;
  fn to_vec_f64<'a, 'b, T>(&'a mut self, subpath:T) -> anyhow::Result<Vec<f64>> where T: Into<StringNumber<'b>>, 'a : 'b;
}

macro_rules! impl_to_vec { ($to_t:tt; $reader:ident, $header:ident; $(($endian:tt, $from_t:tt))*) => { paste! {
  match $header.bitfield {
    $(
      BitField::[<$endian _ $from_t>] => {
        let mut buf = [0u8];
        for _ in 0..$header.offset { let _ = $reader.read_exact(&mut buf); }

        let mut dst : Vec<$to_t> = vec![Default::default(); $header.total];
        let mut buf : [u8; std::mem::size_of::<$from_t>()] = unsafe { std::mem::MaybeUninit::zeroed().assume_init() };
        for i in 0..$header.total {
          if $reader.read_exact(&mut buf).is_err() { break; }
          dst[i] = <$to_t>::clamp_from( <$from_t>::[<from_ $endian _bytes>](buf) );
        }
        dst
      },
    )*
    BitField::unknown => { // ランダムアクセスさせるので一度全部読む
      let decoder = $header.decoder.context("not found decoder")?;
      let mut src: Vec<u8> = Vec::new();
      $reader.read_to_end(&mut src)?;
      match decoder.lang {
        ScriptEnum::Py => { src.[<to_vec_ $to_t _with_py>]("main", decoder.code.as_str(), $header.width, $header.height)? },
        _=> { src.[<to_vec_ $to_t _with_lua>]("main", decoder.code.as_str(), $header.width, $header.height)? }
      }
    },
  }
}}}


impl HrawToVec for Hraw {

  fn to_vec_i32<'a, 'b, T>(&'a mut self, subpath:T) -> anyhow::Result<Vec<i32>> where T: Into<StringNumber<'b>>, 'a : 'b{
    let header = self.header.clone();
    let mut reader = self.to_bufread(subpath)?;
    let dst = impl_to_vec!{ i32; reader, header;
      (le,u8) (be,u8) (le,i8) (be,i8)
      (le,u16) (be,u16) (le,i16) (be,i16)
      (le,u32) (be,u32) (le,i32) (be,i32)
      (le,u64) (be,u64) (le,i64) (be,i64)
      (le,f32) (be,f32) (le,f64) (be,f64)
    };
    Ok(dst)
  }

  fn to_vec_f32<'a, 'b, T>(&'a mut self, subpath:T) -> anyhow::Result<Vec<f32>> where T: Into<StringNumber<'b>>, 'a : 'b {
    let header = self.header.clone();
    let mut reader = self.to_bufread(subpath)?;
    let dst = impl_to_vec!{ f32; reader, header;
      (le,u8) (be,u8) (le,i8) (be,i8)
      (le,u16) (be,u16) (le,i16) (be,i16)
      (le,u32) (be,u32) (le,i32) (be,i32)
      (le,u64) (be,u64) (le,i64) (be,i64)
      (le,f32) (be,f32) (le,f64) (be,f64)
    };
    Ok(dst)
  }

  fn to_vec_f64<'a, 'b, T>(&'a mut self, subpath:T) -> anyhow::Result<Vec<f64>> where T: Into<StringNumber<'b>>, 'a : 'b {
    let header = self.header.clone();
    let mut reader = self.to_bufread(subpath)?;
    let dst = impl_to_vec!{ f64; reader, header;
      (le,u8) (be,u8) (le,i8) (be,i8)
      (le,u16) (be,u16) (le,i16) (be,i16)
      (le,u32) (be,u32) (le,i32) (be,i32)
      (le,u64) (be,u64) (le,i64) (be,i64)
      (le,f32) (be,f32) (le,f64) (be,f64)
    };
    Ok(dst)
  }

}


/* hraw to enumerate */

pub struct HrawIterator<'a, T: RawNumber> {
  stream : BufReader<Box<dyn std::io::Read + 'a>>,  // <-(change)- ZipFile<'a>
  index: usize,
  max: usize,
  phantom: std::marker::PhantomData<T>
}

pub trait HrawEnumerater {
  fn enumerate<'a, 'b, T, U>(&'a mut self, subpath: T) -> anyhow::Result<HrawIterator<U>> where T: Into<StringNumber<'b>> , U: RawNumber, 'a : 'b;
}
impl HrawEnumerater for Hraw {
  fn enumerate<'a, 'b, T, U>(&'a mut self, subpath: T) -> anyhow::Result<HrawIterator<U>> where T: Into<StringNumber<'b>> , U: RawNumber, 'a : 'b {
    let offset = self.header.offset;
    let max = self.header.total;
    let mut stream = self.to_bufread(subpath)?;
    
    let mut buf = [0u8];
    for _ in 0..offset { let _ = stream.read_exact(&mut buf); }
    
    Ok(HrawIterator {
      stream: stream,
      index: 0,
      max: max,
      phantom: std::marker::PhantomData
    })
  }
}

macro_rules! impl_rawnum_iter { ($(($endian:ident, $to_t:ty))*) => { paste! {
  $(
    impl<'a> Iterator for HrawIterator<'a, [<$endian _ $to_t>]> {
      type Item = (usize, $to_t);
      fn next(&mut self) -> Option<Self::Item> {
        let mut buf : [u8; std::mem::size_of::<$to_t>()] = unsafe { std::mem::MaybeUninit::zeroed().assume_init() };
        let current = self.index;
        self.index += 1;
        if current >= self.max { return None; }
        if self.stream.read_exact(&mut buf).is_err() { return None; }
        Some((current, $to_t::[<from_ $endian _bytes>](buf)))
      }
    }
  )*
}}}

impl_rawnum_iter!{
  (le,u8) (be,u8) (le,i8) (be,i8)
  (le,u16) (be,u16) (le,i16) (be,i16)
  (le,u32) (be,u32) (le,i32) (be,i32)
  (le,u64) (be,u64) (le,i64) (be,i64)
  (le,f32) (be,f32) (le,f64) (be,f64)
}

/*
  逆パターンはとりあえず廃止, 
  impl FromHraw for [i32]
  impl FromPng for [i32]
*/
