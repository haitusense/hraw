

pub mod prelude;

pub mod display;
pub mod rawnumber;   // cast from u8, with/without scripting
pub mod conversion;  // casting from hraw to vec<T>, iter... 
pub mod processing;  // color processing

#[cfg(feature = "pixel")]
pub mod pixel;
#[cfg(feature = "open-cv")]
pub mod opencv;

use anyhow::Context;
use thiserror::Error;
use std::io::{BufReader, Read};
use rawnumber::*;
use std::collections::HashMap;
use std::borrow::Cow;

#[allow(unused)]
const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const HEADER_LIST : [&str; 3]= ["header.yaml", "header.yml", "header.json"];

/*** thiserror ***/

#[derive(Error, Debug)]
pub enum HrawError<'a> {
  
  #[error("Failed to load script")]
  HeaderLoad,

  #[error("Failed to call script")]
  Call,

  #[error("{}", format!("{} {}:{}",.0,.1,.2))]
  TLoad(&'a str, &'a str, u32),

  #[error("Failed to call script")]
  Header,

  // #[error("{}", .0.yellow())]  
  // Warning(String),
}

/*** Hraw : Raw image format ***/

/* StringNumber */

pub enum StringNumber<'a> {
  Number(usize),
  String(Cow<'a, str>)
}
impl<'a> From<String> for StringNumber<'a> {
  fn from(n: String) -> Self { Self::String(Cow::Owned(n)) }
}
impl<'a> From<&'a str> for StringNumber<'a> {
  fn from(n: &'a str) -> Self { Self::String(Cow::Borrowed(n)) }
}
impl<'a> From<usize> for StringNumber<'a> {
  fn from(n: usize) -> Self { Self::Number(n) }
}


/* header */

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy, PartialEq, strum::EnumString)]
#[strum(serialize_all = "lowercase")]
pub enum ScriptEnum{
  Lua,
  Py,
  Wasm
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct HeaderDecoder {
  #[serde(default = "default_lang")]
  pub lang: ScriptEnum,
  #[serde(default)]
  pub code: String
}
fn default_lang() -> ScriptEnum { ScriptEnum::Lua }

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct Header {
  pub width     : usize,
  pub height    : usize,

  #[serde(skip)]
  pub total     : usize,
  #[serde(skip)]
  pub stride    : usize,
  #[serde(default)]
  pub offset    : usize,
  #[serde(default = "default_bitfield")]
  pub bitfield  : BitField,

  #[serde(default = "default_data" )]
  pub data      : Vec<String>,

  #[serde(default)]
  pub decoder   : Option<HeaderDecoder>,

  #[serde(skip)]
  pub raw : serde_json::Value,

  #[serde(skip)]
  pub headerpath : String
}
fn default_bitfield() -> BitField { BitField::le_i32 }
fn default_data() -> Vec<String> { vec!["data.raw".to_string()] }
// fn default_data() -> Vec<String> { serde_json::json!([DEFAULT_DATA]).as_array().unwrap().to_owned() }

impl Header {
  pub fn to_size(&self) -> (usize, usize, usize) { (self.width, self.height, self.total) }
  pub fn to_subpath(&self) -> Vec<String> { self.data.clone() }

  // Vec<Value>にして、ファイル毎の設定をいれるか
  pub fn get_subpath(&self, index:usize) -> anyhow::Result<String> {
    let dst = match self.data.get(index) {
      // Some(n) => n.as_str().context("path not found")?.to_owned(), // data : Vec<Value>の場合
      Some(n) => n.clone(),
      None => anyhow::bail!("path not found"),
    };
    Ok(dst)
  }
}


/* hraw */

#[derive(Debug)]
pub struct Hraw {
  path   : std::path::PathBuf,
  header : Header,
  zip    : Option<zip::ZipArchive<std::io::BufReader<std::fs::File>>>,
}



impl Hraw {

  pub fn new(path: &str) -> anyhow::Result<Hraw> {
    let target = std::path::PathBuf::from(path);
    let is_dir = target.is_dir();
    let header_value = Hraw::open_header_value(path, is_dir)?;

    match is_dir {
      true => Ok(Hraw{
        path : target,
        header : Hraw::value_to_header(header_value.0.as_str(), header_value.1)?,
        zip: None
      }),
      false => Ok(Hraw{
        path : target,
        header : Hraw::value_to_header(header_value.0.as_str(), header_value.1)?,
        zip: Some(zip::ZipArchive::new(std::io::BufReader::new(std::fs::File::open(path)?))?)
      }),
    }
  
  }

  /* private */
  fn open_header_value(path: &str, is_dir: bool) -> anyhow::Result<(String, serde_json::Value)> {
    match is_dir {
      true => {
        let root = std::path::PathBuf::from(path);
        for i in crate::HEADER_LIST.iter() {
          let path_buf = root.join(i);
          if path_buf.is_file() {
            let key = i.to_string();
            let buf = std::fs::read_to_string(path_buf)?;
            let value: serde_json::Value = serde_yaml::from_str(buf.as_str())?;
            return Ok((key, value));
          }
        }
      },
      false => {
        let mut zip = zip::ZipArchive::new(std::fs::File::open(path)?)?;
        for i in crate::HEADER_LIST.iter() {
          if let Ok(mut file) = zip.by_name(i) {
            let mut buf = String::new();
            let _ = file.read_to_string(&mut buf);
            let key = i.to_string();
            let value: serde_json::Value = serde_yaml::from_str(buf.as_str())?;
            return Ok((key, value));
          }
        }
      }
    };
    anyhow::bail!("header not found")
  }

  fn value_to_header(filename: &str, value: serde_json::Value) -> anyhow::Result<Header> {
    let mut dst = serde_json::from_value::<Header>(value.clone())?;
    dst.total = dst.width * dst.height;
    dst.stride = dst.width;
    dst.raw = value;
    dst.headerpath = filename.to_string();
    Ok(dst)
  }
  
  fn get_path<'a, T>(&self, value: T) -> Option<Cow<'a, str>>
  where T: Into<StringNumber<'a>> {
    match value.into() {
      StringNumber::String(c) => Some(c),
      StringNumber::Number(n) => {
        let list = &self.header.data;
        match list.get(n) {
          Some(m) => Some(Cow::Owned(m.to_owned())),
          None => None
        }
      },
    }
  }

  /* info and header */
  
  pub fn path_str(&self) -> &str { self.path.to_str().unwrap_or_default() }

  pub fn info(&mut self) -> anyhow::Result<HashMap::<String, String>> {
    let mut dic = HashMap::<String, String>::new();
    match self.zip.as_mut() {
      None => {
        walkdir::WalkDir::new(self.path.as_path())
          .into_iter()
          .filter_map(|v| v.ok())
          .for_each(|x| {
            let dir_kind = match x.path() {
              n if n.is_file() => "file",
              n if n.is_dir() => "dir",
              _ => "unknown" 
            };

            let target = x.path().canonicalize().unwrap_or_default();
            let root = self.path.canonicalize().unwrap_or_default();
            let dir_path = match target.strip_prefix(root) {
              Ok(n) => n.display().to_string(),
              Err(_) => "".to_string()
            };
            if dir_path.as_str() != "" { dic.insert(dir_path.to_owned(), dir_kind.to_owned() ); };
          });
      },
      Some(z) => {
        for i in 0..z.len() {
          let zip_path = z.by_index(i)?;
          let zip_kind = match &zip_path {
            n if n.is_file() => "file",
            n if n.is_dir() => "dir",
            _=> "unknown" 
          };
          dic.insert(zip_path.name().to_owned(), zip_kind.to_owned() );
        }
      }
    }
    Ok(dic)
  }

  pub fn header(&self) -> Header { self.header.clone() }

  /* data */
  pub fn contain<'a, T: Into<StringNumber<'a>>>(&mut self, subpath: T) -> anyhow::Result<String> {
    let subpath = self.get_path(subpath).context(HrawError::Header)?.to_string();
    let zip = self.zip.as_mut();
    match zip {
      None => {
        let path_buf = self.path.join(subpath.as_str());
        match path_buf.is_file() {
          true => Ok(subpath.to_string()),
          false => Ok(subpath.to_string()),
        }
      },
      Some(z) => {
        let _ = z.by_name(subpath.as_str())?;
        Ok(subpath.to_string())
      }
    }
  }


  /* get reader */
  // FileはSeekあるけど、ZipFileはSeekがない
  pub fn to_handle<'a, 'b,  T: Into<StringNumber<'b>>>(&'a mut self, subpath: T) -> anyhow::Result<Box<dyn Read + 'a>>
    where 'a : 'b
    {
    let subpath = self.get_path(subpath).context(HrawError::Header)?.into_owned();
    match self.zip.as_mut() {
      None => {
        let path_buf = self.path.join(subpath);
        let handle = std::fs::File::open(path_buf)?;
        Ok(Box::new(handle))
      },
      Some(z) => {
        let handle = z.by_name(&subpath)?;
        Ok(Box::new(handle))
      }
    }
  }

  pub fn to_bufread<'a, 'b, T>(&'a mut self, subpath: T) -> anyhow::Result<BufReader<Box<dyn Read + 'a>>>
  where T: Into<StringNumber<'b>>, 'a : 'b {
    Ok(BufReader::new(self.to_handle(subpath)?))
  }


  /* get byte */
  pub fn to_bytes<'a, 'b, T>(&'a mut self, subpath: T) -> anyhow::Result<Vec<u8>> where T: Into<StringNumber<'b>>, 'a : 'b {
    let handle = self.to_handle(subpath)?;
    let mut reader = std::io::BufReader::new(handle);

    let mut dst: Vec<u8> = Vec::new();
    reader.read_to_end(&mut dst)?;
    Ok(dst)
  }

}
