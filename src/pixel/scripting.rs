use anyhow::Context;
use serde::{Serialize, de::DeserializeOwned};
use thiserror::Error;
use std::sync::{Arc,Mutex};

use mlua::prelude::*;
use pyo3::{ prelude::*, types::{PyAny, PySlice, /* PySliceIndices */}}; // use pyo3::{Python, Py, Bound};
use numpy::{ prelude::*, PyArray, PyArray2 };
use std::borrow::Cow;
use super::Pixel;

// const ENTRY : &str = "main";
const BUILTIN : &str = "pixel";

/*** thiserror ****/

#[derive(Error, Debug)]
enum ScriptError {
  
  #[error("Failed in lua script")]
  Lua,

  #[error("Failed in py script")]
  Py,

  #[error("Unimplemented")]
  Unimplemented,
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


/*** PixelModify ****/

pub trait PixelModify {
  
  fn modify_with_py<'a, N, T, U>(&self, entry: N, code: &str, args: T) -> anyhow::Result<U> 
    where N: Into<NullableString<'a>>, T: Serialize, U: DeserializeOwned;
  fn modify_with_py_stdout<'a, N, T, F, U>(&self, entry: N, code: &str, args: T, stdout: F) -> anyhow::Result<U> 
    where N: Into<NullableString<'a>>, T: Serialize, U: DeserializeOwned, F: Fn(&str) + Send + Sync + 'static;

  fn modify_with_lua<'a, N, T, U>(&self, entry: N, code: &str, args: T) -> anyhow::Result<U>
    where N: Into<NullableString<'a>>, T : Serialize, U: DeserializeOwned;
  fn modify_with_lua_stdout<'a, N, T, F, U>(&self, entry: N, code: &str, args: T, stdout: F) -> anyhow::Result<U>
    where N: Into<NullableString<'a>>, T: Serialize, U: DeserializeOwned, F: Fn(&str);

}

impl PixelModify for Arc<Mutex<Pixel<i32>>> {

  fn modify_with_py<'a, N, T, U>(&self, entry:N, code:&str, args:T) -> anyhow::Result<U>
  where N: Into<NullableString<'a>>, T : Serialize, U: DeserializeOwned {
    Python::with_gil(|py| -> anyhow::Result<U> {
      let json = py.import_bound("json")?;

      let module = PyModule::from_code_bound(py, code,"", "",)?;
      module.add(BUILTIN, PixelScripting(self.clone()))?;
      let func = module.getattr(entry.into().as_ref_or("main"))?;

      let args_str = serde_json::to_string(&args)?;
      // let args_py = py.eval_bound(args_str.as_str(), None, None)?;
      let args_py = json.call_method1("loads", (args_str,))?;
      let dst = func.call1((args_py,))?;
      let dst_str = json.call_method1("dumps", (dst,))?.extract::<String>()?;
      let dst_obj = serde_json::from_str::<U>(&dst_str)?;
      
      Ok(dst_obj)
    }).context(ScriptError::Py)
  }
  fn modify_with_py_stdout<'a, N, T, F, U>(&self, entry: N, code: &str, args: T, stdout: F) -> anyhow::Result<U> 
  where N: Into<NullableString<'a>>, T: Serialize, U: DeserializeOwned, F: Fn(&str) + Send + Sync + 'static {
    // F: IntoPy<PyObject> -> Fn(String)に変更

    Python::with_gil(|py| -> anyhow::Result<U> {
      let sys = py.import_bound("sys")?;
      let json = py.import_bound("json")?;

      sys.setattr("stdout", PythonStdout::new(stdout).into_py(py))?;

      let module = PyModule::from_code_bound(py, code,"", "",)?;
      module.add(BUILTIN, PixelScripting(self.clone()))?;
      let func = module.getattr(entry.into().as_ref_or("main"))?;

      let args_str = serde_json::to_string(&args)?;
      let args_py = json.call_method1("loads", (args_str,))?;
      let dst = func.call1((args_py,))?;
      let dst_str = json.call_method1("dumps", (dst,))?.extract::<String>()?;
      let dst_obj = serde_json::from_str::<U>(&dst_str)?;
      
      Ok(dst_obj)
    }).context(ScriptError::Py)
  }

  fn modify_with_lua<'a, N, T, U>(&self, entry:N, code: &str, args: T) -> anyhow::Result<U>
  where N: Into<NullableString<'a>>, T: Serialize, U: DeserializeOwned {
    let lua = Lua::new();
    anyhow::Context::context(lua.scope(|_scope| {
      let globals = lua.globals();
      globals.set(BUILTIN, PixelScripting(self.clone()))?;

      let func: mlua::Function = match entry.into().0 {
        Some(e) => {
          lua.load(code).exec()?;
          lua.globals().get(e)?
        }
        None => lua.load(code).eval()?,
      };

      let args_lua = lua.to_value(&args)?;
      let dst = func.call::<_, mlua::Value>((args_lua,))?;
      let dst_obj = lua.from_value(dst)?;

      Ok(dst_obj)
    }), ScriptError::Lua)
  }
  
  fn modify_with_lua_stdout<'a, N, T, F, U>(&self, entry:N, code: &str, args: T, stdout: F) -> anyhow::Result<U>
  where N: Into<NullableString<'a>>, T: Serialize, U: DeserializeOwned, F: Fn(&str) {
    let lua = Lua::new();
    let result = anyhow::Context::context(lua.scope(|scope| {
      lua.globals().set(BUILTIN, PixelScripting(self.clone()))?;
      lua.globals().set("print", scope.create_function(|_, msg: String | {
        stdout(msg.as_str());
        Ok(())
      })?)?;

      let func: mlua::Function = match entry.into().0 {
        Some(e) => {
          lua.load(code).exec()?;
          lua.globals().get(e)?
        }
        None => lua.load(code).eval()?,
      };
      let args_lua = lua.to_value(&args)?;
      let dst = func.call::<_, mlua::Value>((args_lua,))?;
      let dst_obj = lua.from_value(dst)?;

      Ok(dst_obj)
    }), ScriptError::Lua);
    result
  }

}


/*** PixelScripting ****/
/* 
  lua | number : double-precision (64-bit) floating-point 
  py  | number : double-precision (64-bit) floating-point 
      | int    : 上限はなし
      | NumPy  : int32 / int64
      | float  : 64bit
*/

#[pyo3::prelude::pyclass]
pub struct PixelScripting (Arc<Mutex<Pixel<i32>>>);

impl PixelScripting {
  pub fn to_mmf(&mut self, _path:&str) -> anyhow::Result<()>{ panic!("{}", ScriptError::Unimplemented) }
}


/* for lua */
impl mlua::UserData for PixelScripting {
  fn add_fields<'lua, F: mlua::UserDataFields<'lua, Self>>(_fields: &mut F) { }

  fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
    methods.add_method("get", |_, this, (x, y): (usize, usize)| {
      match this.0.lock() {
        Ok(n) => Ok(n.data[x + y * n.width]),
        Err(_e) => Err(mlua::Error::RuntimeError(format!("{}", ScriptError::Lua)))
      }
    });
    methods.add_method_mut("set", |_, this, (x, y, val): (usize, usize, i32)| {
      let width = this.0.lock().unwrap().width;
      this.0.lock().unwrap().data[x + y * width] = val;
      Ok(())
    });

    methods.add_method("width", |_, this, ()| {
      Ok(this.0.lock().unwrap().width)
    });
    methods.add_method("height", |_, this, ()| {
      Ok(this.0.lock().unwrap().height)
    });

    methods.add_method("to_table", |_, this, ()| {
      Ok(this.0.lock().unwrap().data.clone())
    });

    methods.add_meta_method(mlua::MetaMethod::Index, |_, this, index: usize| {
      Ok(this.0.lock().unwrap().data[index])
    });
  }
}

/* for py */
#[pyo3::prelude::pymethods]
impl PixelScripting {

  // indexer
  fn __getitem__<'py>(&self, py: Python<'py>, index: &Bound<'_, PyAny>) -> PyResult<PyObject> {
    let px = self.0.lock().unwrap();
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

  fn get(&self, x:usize, y:usize) -> i32 {
    let px = self.0.lock().unwrap();
    px.data[x + y * px.width] 
  }
  fn set(&mut self, x:usize, y:usize, val:i32) {
    let width = self.0.lock().unwrap().width;
    self.0.lock().unwrap().data[x + y * width] = val;
  }

  fn width(&self) -> usize { self.0.lock().unwrap().width }
  fn height(&self) -> usize { self.0.lock().unwrap().height }

  fn to_array(&self) -> Vec<i32> { self.0.lock().unwrap().data.clone() }
  fn to_np<'py>(&self, py: Python<'py>) -> Py<PyArray2<i32>> {
    let px = self.0.lock().unwrap();
    let arr = PyArray::from_vec_bound(py, px.data.clone());
    let pyarray = arr.reshape([px.height, px.width]).unwrap();
    pyarray.into()
  }
  
  fn from_array(&mut self, src: Vec<i32>) {
    self.0.lock().unwrap().data.clear();
    self.0.lock().unwrap().data.extend(src);
  }  
  fn from_np<'py>(&mut self, src: &Bound<'py, PyArray2<i32>>) {
    self.0.lock().unwrap().data.clear();
    self.0.lock().unwrap().data.extend(src.to_vec().unwrap());
  }

}

/*** stdout ***/
#[pyo3::prelude::pyclass]
pub struct PythonStdout { callback: Arc<Mutex<dyn Fn(&str) + Send + Sync>>, }

impl PythonStdout {
  pub fn new<T : Fn(&str) + Send + Sync + 'static>(f: T) -> Self {
    Self { callback : Arc::new(Mutex::new(f)) }
  }
}

#[pyo3::prelude::pymethods]
impl PythonStdout {
  fn write(&self, msg: &str) {
    let callback = self.callback.clone();
    callback.lock().unwrap()(msg);
  }
}

pub fn eval_py1<T>(code : &str, stdout : T) -> anyhow::Result<String> where T: IntoPy<Py<PyAny>> {
  Python::with_gil(|py| -> anyhow::Result<String> {
    let sys = py.import_bound("sys")?;
    sys.setattr("stdout", stdout.into_py(py))?;
    let dst = py.eval_bound(code, None, None)?;
    // let _ = mutex.lock().unwrap().send_event(huazhi::event_handler::UserEvent::TerminalMessage(format!("{dst}\r\n")));
    Ok(format!("{dst}"))
  })
}

