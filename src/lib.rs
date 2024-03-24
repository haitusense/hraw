mod test;

// pub mod extension;
pub mod processing;
pub mod rawnumber;
// use byteorder::LE;
// use std::borrow::Cow;
// use std::io::BufReader;
use rawnumber::*;

use anyhow::Context as _;
use serde_json::json;
use std::io::Read;
use paste::paste;
use zip::read::ZipFile;

const HEADER_LIST : [&str; 3]= ["header.yaml", "header.yml", "header.json"];
const DEFAULT_DATA : &str = "data.raw";

/*** Hraw : Raw image format ***/

pub struct Hraw {
  zip: zip::ZipArchive<std::io::BufReader<std::fs::File>>,
}

impl Hraw {

  pub fn new(path:&str) -> anyhow::Result<Hraw> {
    let file = std::fs::File::open(path)?;
    let reader = std::io::BufReader::new(file);
    let zip: zip::ZipArchive<std::io::BufReader<std::fs::File>> = zip::ZipArchive::new(reader)?;
    Ok(Hraw { zip })
  }

  pub fn info(&mut self) -> anyhow::Result<()> {
    for i in 0..self.zip.len() {
      let zip_path = self.zip.by_index(i)?;
      let zip_type = match &zip_path {
        n if n.is_dir() => "dir",
        n if n.is_file() => "file",
        _=> "unknown" 
      };
      println!("{} : {} {}", i, zip_type, &zip_path.name());
    }

    HEADER_LIST.iter().for_each(|n| {
      println!("--------{n}--------");
      if let Ok(mut file) = self.zip.by_name(n) { 
        let mut buf = String::new();
        let _ = file.read_to_string(&mut buf);
        let value: serde_json::Value = serde_yaml::from_str(buf.as_str()).unwrap_or(json!({}));
        println!("{:?}", value);
      }
    });
    Ok(())
  }

  pub fn header(&mut self) -> serde_json::Value {
    for i in HEADER_LIST.iter() {
      if let Ok(mut file) = self.zip.by_name(i) {
        let mut buf = String::new();
        let _ = file.read_to_string(&mut buf);
        let value: serde_json::Value = serde_yaml::from_str(buf.as_str()).unwrap_or(json!({}));
        return value;
      }
    }
    json!({ "err" : "header not found" })
  }

  pub fn to_vec(&mut self, path:&str) -> anyhow::Result<Vec<u8>> {
    let mut file = self.zip.by_name(path)?;
    let mut dst: Vec<u8> = Vec::new();
    file.read_to_end(&mut dst)?;
    Ok(dst)
  }

  #[deprecated]
  pub fn to_stream(&mut self, path:&str) -> anyhow::Result<ZipFile<'_>> {
    /*
      元がZipArchive<BufReader<File>>なので
      let stream = std::io::BufReader::new(file);で包むのは止めた
      所有権の返却もできないので使いどころ?
    */
    let file = self.zip.by_name(path)?;
    Ok(file)
  }

}


#[allow(unused)]
#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Header {
  width        : usize,
  height       : usize,

  #[serde(skip)]
  total        : usize,
  #[serde(skip)]
  stride       : usize,
  #[serde(default)]
  offset       : usize,

  #[serde(default = "default_bitfield")]
  bitfield     : BitField,

  #[serde(default = "default_data" )]
  data: Vec<serde_json::Value>
}
fn default_bitfield() -> BitField { BitField::le_i32 }
fn default_data() -> Vec<serde_json::Value> { serde_json::json!([DEFAULT_DATA]).as_array().unwrap().to_owned() }

pub trait HrawHeader {  
  fn to_struct(&self) -> Header;
  fn to_size(&mut self) -> (usize, usize, usize);
  fn to_data_dict(&mut self, index:usize) -> anyhow::Result<String>;
}
impl HrawHeader for serde_json::Value {
  fn to_struct(&self) -> Header{
    let mut dst = serde_json::from_value::<Header>(self.to_owned()).unwrap();
    dst.total = dst.width * dst.height;
    dst.stride = dst.width;
    dst
  }
  fn to_size(&mut self) -> (usize, usize, usize) {
    let dst = self.to_struct();
    (dst.width, dst.height, dst.total)
  }
  fn to_data_dict(&mut self, index:usize) -> anyhow::Result<String> {
    let dst = match self.to_struct().data.get(index) {
      Some(n) => n.as_str().context("path not found")?.to_owned(),
      None => anyhow::bail!("path not found"),
    };
    Ok(dst)
  }
}


/*** PathOrIndex ***/

pub trait PathOrIndex { fn to_name(&self, src: &mut Hraw) -> anyhow::Result<String>; }
impl PathOrIndex for &str { 
  fn to_name(& self, _: &mut Hraw) -> anyhow::Result<String> { Ok(self.to_string()) } 
}
impl PathOrIndex for usize { 
  fn to_name(& self, src: &mut Hraw) -> anyhow::Result<String> { src.header().to_data_dict(*self) }
}

pub trait HrawPathOrIndex {
  fn contain_poi<T:PathOrIndex>(&mut self, path:T) -> anyhow::Result<String>;
  fn to_vec_poi<T:PathOrIndex>(&mut self, path:T) -> anyhow::Result<Vec<u8>>;
}

impl HrawPathOrIndex for Hraw {

  fn contain_poi<T:PathOrIndex>(&mut self, path:T) -> anyhow::Result<String> {
    let path = path.to_name(self)?;
    let _ = self.zip.by_name(path.as_str())?;
    Ok(path)
  }

  fn to_vec_poi<T:PathOrIndex>(&mut self, path:T) -> anyhow::Result<Vec<u8>> {
    let path = path.to_name(self).unwrap_or(DEFAULT_DATA.to_string());
    let mut file = self.zip.by_name(path.as_str())?;
    let mut dst: Vec<u8> = Vec::new();
    file.read_to_end(&mut dst)?;
    Ok(dst)
  }

}


/*** buffer ***/

pub mod buffer {
  use crate::*;
  // use std::io::{Bytes, Seek};
  // use byteorder::ReadBytesExt;

  // Enumerate_i32
  pub struct HrawIterator<'a, T: RawNumber> {
    stream : ZipFile<'a>,
    index: usize,
    max: usize,
    phantom: std::marker::PhantomData<T>
  }

  pub trait HrawEnumerater {
    fn enumerate_index<T: rawnumber::RawNumber>(&mut self, path:usize) -> HrawIterator<T>;
    fn enumerate_path<T: rawnumber::RawNumber>(&mut self, path:&str) -> HrawIterator<T>;
  }
  impl HrawEnumerater for Hraw {
    fn enumerate_index<T: RawNumber>(&mut self, path: usize) -> HrawIterator<T> {
      let path = self.header().to_data_dict(path).unwrap_or(DEFAULT_DATA.to_owned());
      self.enumerate_path(path.as_str())
    }
    fn enumerate_path<T: RawNumber>(&mut self, path: &str) -> HrawIterator<T> {
      let header = &self.header().to_struct();
      let mut stream = self.zip.by_name(path).unwrap();
      let mut buf = [0u8];
      for _ in 0..header.offset {
        let _ = stream.read_exact(&mut buf);
      }
      HrawIterator {
        stream: stream,
        index: 0,
        max: header.total,
        phantom: std::marker::PhantomData
      }
    }
  }
  
  impl<'a> Iterator for HrawIterator<'a, be_i32> {
    type Item = (usize, i32);
    fn next(&mut self) -> Option<Self::Item> {
      let mut buf : [u8; std::mem::size_of::<i32>()] = unsafe { std::mem::MaybeUninit::zeroed().assume_init() };
      let current = self.index;
      self.index += 1;
      if current >= self.max { return None; }
      if self.stream.read_exact(&mut buf).is_err() { return None; }
      Some((current, i32::from_le_bytes(buf)))
    }
  }

  macro_rules! impl_rawnum_iter { ($(($t:ident,$u:ty))*) => { paste! {
    $(
      impl<'a> Iterator for HrawIterator<'a, [<$t _ $u>]> {
        type Item = (usize, $u);
        fn next(&mut self) -> Option<Self::Item> {
          let mut buf : [u8; std::mem::size_of::<$u>()] = unsafe { std::mem::MaybeUninit::zeroed().assume_init() };
          let current = self.index;
          self.index += 1;
          if current >= self.max { return None; }
          if self.stream.read_exact(&mut buf).is_err() { return None; }
          Some((current, $u::[<from_ $t _bytes>](buf)))
        }
      }
    )*
  }}}
  impl_rawnum_iter!{
    (le,u8) (be,u8) (le,i8) (be,i8)
    (le,u16) (be,u16) (le,i16) (be,i16)
    (le,u32) (be,u32) (le,i32)
    (le,u64) (be,u64) (le,i64) (be,i64)
    (le,f32) (be,f32) (le,f64) (be,f64)
  }


  pub trait FromHraw {
    fn from_hraw(&mut self, path:&str, file:&str);
  }
  macro_rules! impl_from_hraw { ($t:tt;$self:ident,$path:ident,$file:ident;$($tt:tt)*) => {
    let mut raw = Hraw::new($path).unwrap();
    let header = raw.header().to_struct();
    match header.bitfield {
      $(
        BitField::$tt => raw.enumerate_path::<$tt>($file).for_each(|(i, n)| { $self[i] = $t::clamp_from(n); }),
      )*
      _=> panic!("unknown")
    }
  }}
  impl FromHraw for [i32] {
    fn from_hraw(&mut self, path:&str, file:&str) {
      impl_from_hraw!{
        i32; self, path, file;
        le_u8 be_u8 le_i8 be_i8
        le_u16 be_u16 le_i16 be_i16
        le_u32 be_u32 le_i32 be_i32
        le_u64 be_u64 le_i64 be_i64
        le_f32 be_f32 le_f64 be_f64
      }
    }
  }
  impl FromHraw for [f32] {
    fn from_hraw(&mut self, path:&str, file:&str) {
      impl_from_hraw!{
        f32; self, path, file;
        le_u8 be_u8 le_i8 be_i8
        le_u16 be_u16 le_i16 be_i16
        le_u32 be_u32 le_i32 be_i32
        le_u64 be_u64 le_i64 be_i64
        le_f32 be_f32 le_f64 be_f64
      }
    }
  }
  impl FromHraw for [f64] {
    fn from_hraw(&mut self, path:&str, file:&str) {
      impl_from_hraw!{
        f64; self, path, file;
        le_u8 be_u8 le_i8 be_i8
        le_u16 be_u16 le_i16 be_i16
        le_u32 be_u32 le_i32 be_i32
        le_u64 be_u64 le_i64 be_i64
        le_f32 be_f32 le_f64 be_f64
      }
    }
  }

}
