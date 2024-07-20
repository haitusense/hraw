
use anyhow::Context as _;
use thiserror::Error;
use mlua::prelude::*;
use pyo3::{prelude::*, Python, types::{PyAny, PySlice, /* PySliceIndices */}/*, Py, types::PyAny  Bound*/};
use extism::*;
use std::borrow::Cow;

const BUILDIN : &str = "rawbytes";

/*** thiserror ***/

#[derive(Error, Debug)]
enum ScriptError<'a> {
  
  #[error("Failed to load script")]
  Load,

  #[error("Failed to call script")]
  Call,

  #[allow(unused)]
  #[error("{}", format!("{} {}:{}",.0,.1,.2))]
  TLoad(&'a str, &'a str, u32),

  // #[error("{}", .0.yellow())]  
  // Warning(String),
}

/*** NullableString ****/

pub struct NullableString<'a> (Option<Cow<'a, str>>);

impl<'a> NullableString<'a> {
  pub fn as_ref_or(&'a self, default: &'a str) -> &'a str {
    let cow = &self.0;
    match cow {
      Some(n) => n.as_ref(),
      None => default
    }
  }
}

impl<'a> From<String> for NullableString<'a> {
  fn from(n: String) -> Self { Self(Some(Cow::Owned(n))) }
}
impl<'a> From<&'a str> for NullableString<'a> {
  fn from(n: &'a str) -> Self { Self(Some(Cow::Borrowed(n))) }
}
impl<'a> From<Option<Cow<'a, str>>> for NullableString<'a> {
  fn from(n: Option<Cow<'a, str>>) -> Self { Self(n) }
}


/*** call with strcut ***/

/* 共通 */

#[pyo3::prelude::pyclass]
pub struct RawBytes {
  pub path : String,
  pub width : usize,
  pub height : usize,
  pub data : Vec<u8>
}

// [derive(mlua::FromLua)]でもいい?
impl mlua::UserData for RawBytes {
  
  fn add_fields<'lua, F: mlua::UserDataFields<'lua, Self>>(fields: &mut F) {
    fields.add_field_method_get("path", |_, this| {
      Ok(this.path.clone())
    });
    fields.add_field_method_get("width", |_, this| {
      Ok(this.width)
    });
    fields.add_field_method_get("height", |_, this| {
      Ok(this.height)
    });
  }

  fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
    methods.add_method("get", |_, this, index: usize| {
      Ok(this.data[index])
    });
    methods.add_method("to_table", |_, this, ()| {
      Ok(this.data.clone())
    });
    methods.add_meta_method(mlua::MetaMethod::Index, |_, this, index: usize| {
      Ok(this.data[index])
    });

  }

}

#[pyo3::prelude::pymethods]
impl RawBytes {
  // indexer
  fn __getitem__<'py>(&self, py: Python<'py>, index: &Bound<'_, PyAny>) -> PyResult<PyObject> {
    let px = self;
    let len = px.data.len() as isize;
    if let Ok(idx) = index.extract::<isize>() {
      let idx = if idx < 0 { len + idx } else { idx };
      if idx < 0 || len <= idx {
        Err(pyo3::exceptions::PyIndexError::new_err("Index out of range"))
      } else {
        Ok(px.data[idx as usize].into_py(py))
      }
    } else if let Ok(slice) = index.downcast::<PySlice>() {
      let indices = slice.indices(len as i32)?;
      let (start, stop, step) = (indices.start, indices.stop, indices.step);
      let mut result = Vec::new();
      let mut i = start;
      while if step > 0 { i < stop } else { i > stop } {
        result.push(px.data[i as usize]);
        i += step;
      }
      Ok(result.into_py(py))
    } else {
      Err(pyo3::exceptions::PyTypeError::new_err("Invalid index type"))
    }
  }

  fn path(&self) -> String { self.path.clone() }
  fn width(&self) -> i32 { self.width as i32 }
  fn height(&self) -> i32 { self.height as i32 }

  fn get(&self, index:usize) -> u8 { self.data[index] }
  fn to_array(&self) -> Vec<u8> { self.data.clone() }
}


/*** u8 to i32/f32/f64 ***/
// ランダムアクセスさせるので一度全部読む

pub trait BytesScripting {
  
  fn to_vec_i32_with_lua(&mut self, entry:&str, code:&str, width:usize, height:usize) -> anyhow::Result<Vec<i32>>;
  fn to_vec_f32_with_lua(&mut self, entry:&str, code:&str, width:usize, height:usize) -> anyhow::Result<Vec<f32>>;
  fn to_vec_f64_with_lua(&mut self, entry:&str, code:&str, width:usize, height:usize) -> anyhow::Result<Vec<f64>>;

  fn to_vec_i32_with_py(&mut self, entry:&str, code:&str, width:usize, height:usize) -> anyhow::Result<Vec<i32>>;
  fn to_vec_f32_with_py(&mut self, entry:&str, code:&str, width:usize, height:usize) -> anyhow::Result<Vec<f32>>;
  fn to_vec_f64_with_py(&mut self, entry:&str, code:&str, width:usize, height:usize) -> anyhow::Result<Vec<f64>>;
  
  fn to_vec_i32_with_wasm(&mut self, entry:&str, code:&str, width:usize, height:usize) -> anyhow::Result<Vec<i32>>;
  fn to_vec_f32_with_wasm(&mut self, entry:&str, code:&str, width:usize, height:usize) -> anyhow::Result<Vec<f32>>;
  fn to_vec_f64_with_wasm(&mut self, entry:&str, code:&str, width:usize, height:usize) -> anyhow::Result<Vec<f64>>;

}

trait LuaEx {
  fn call_lua<'lua, 'a, S, T>(&'lua self, src:&[u8], entry: S, code: &str, width: usize, height: usize) -> anyhow::Result<Vec<T>>
  where S: Into<NullableString<'a>>, T : mlua::FromLuaMulti<'lua> + Default + Clone;
}
impl LuaEx for Lua {
  fn call_lua<'lua, 'a, S, T>(&'lua self, src:&[u8], entry: S, code: &str, width: usize, height: usize) -> anyhow::Result<Vec<T>>
  where S: Into<NullableString<'a>>, T : mlua::FromLuaMulti<'lua> + Default + Clone {
    let mut dst : Vec<T> = vec![Default::default(); width*height];
    let globals = self.globals();
    anyhow::Context::context(globals.set(BUILDIN, RawBytes{
      path : "".to_string(),
      width : width,
      height : height,
      data : src.to_vec(),
    }), ScriptError::Load)?;
    

    let func: mlua::Function = match entry.into().0 {
      Some(e) => {
        anyhow::Context::context(self.load(code).exec(), ScriptError::Load)?;
        anyhow::Context::context(self.globals().get(e), ScriptError::Load)?
      },
      None => anyhow::Context::context(self.load(code).eval(), ScriptError::Load)?
    };

    for i in 0..width*height {
      dst[i] = anyhow::Context::context(func.call::<_, T>(i), ScriptError::Call)?;
    }
    Ok(dst)
  }
}

macro_rules! macro_py { ($t:tt; $self:ident, $entry:ident, $code:ident, $width:ident, $height:ident) => {
  Python::with_gil(|py| -> anyhow::Result<Vec<$t>> {
    let mut dst : Vec<$t> = vec![Default::default(); $width*$height];
    let module = PyModule::from_code_bound(py, $code, "", "",).context(ScriptError::Load)?;
    module.add(BUILDIN, RawBytes{
      path : "".to_string(),
      width : $width,
      height : $height,
      data : $self.to_vec(),
    }).context(ScriptError::Load)?;
    let func = module.getattr($entry).context(ScriptError::Load)?;

    for i in 0..$width*$height {
      dst[i] = func.call1((i,)).context(ScriptError::Call)?.extract::<$t>()?;
    }
    Ok(dst)
  })
}}

impl BytesScripting for [u8] {

  fn to_vec_i32_with_lua(&mut self, entry:&str, code:&str, width:usize, height:usize) -> anyhow::Result<Vec<i32>> {
    let lua = Lua::new();
    lua.call_lua(self, entry, code, width, height)
  }
  fn to_vec_f32_with_lua(&mut self, entry:&str, code:&str, width:usize, height:usize) -> anyhow::Result<Vec<f32>> {
    let lua = Lua::new();
    lua.call_lua(self, entry, code, width, height)
  }
  fn to_vec_f64_with_lua(&mut self, entry:&str, code:&str, width:usize, height:usize) -> anyhow::Result<Vec<f64>> {
    let lua = Lua::new();
    lua.call_lua(self, entry, code, width, height)
  }

  fn to_vec_i32_with_py(&mut self, entry:&str, code:&str, width:usize, height:usize) -> anyhow::Result<Vec<i32>> {
    macro_py!(i32; self, entry, code, width, height)
  }
  fn to_vec_f32_with_py(&mut self, entry:&str, code:&str, width:usize, height:usize) -> anyhow::Result<Vec<f32>> {
    macro_py!(f32; self, entry, code, width, height)
  }
  fn to_vec_f64_with_py(&mut self, entry:&str, code:&str, width:usize, height:usize) -> anyhow::Result<Vec<f64>> {
    macro_py!(f64; self, entry, code, width, height)
  }

  fn to_vec_i32_with_wasm(&mut self, entry:&str, code:&str, width:usize, height:usize) -> anyhow::Result<Vec<i32>> {
    panic!("Unimplemented")
  }
  fn to_vec_f32_with_wasm(&mut self, entry:&str, code:&str, width:usize, height:usize) -> anyhow::Result<Vec<f32>> {
    panic!("Unimplemented")
  }
  fn to_vec_f64_with_wasm(&mut self, entry:&str, code:&str, width:usize, height:usize) -> anyhow::Result<Vec<f64>> {
    panic!("Unimplemented")
  }
}
