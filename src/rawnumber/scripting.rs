// use anyhow::Context as _;
use mlua::prelude::*;
use pyo3::prelude::*;
// use numpy::{PyArray, PyArray2};

pub trait LuaEx {
  fn call_func<'lua, T : mlua::FromLuaMulti<'lua>>(&'lua self, src:&[u8], dst:&mut [T], width:usize, height:usize, code:&'lua str) -> anyhow::Result<()>;    
}

impl LuaEx for Lua {
  fn call_func<'lua, T : mlua::FromLuaMulti<'lua>>(&'lua self, src:&[u8], dst:&mut [T], width:usize, height:usize, code:&'lua str) -> anyhow::Result<()> {
    let globals = self.globals();
    globals.set("src", src)?;
    let func: mlua::Function = self.load(code).eval()?;
    (0..width*height).for_each(|i|{ 
      dst[i] = func.call::<_, T>(i).unwrap();
    });
    Ok(())
  }
}


pub fn lua_call(code:&str) {
  use mlua::prelude::*;
  let lua = Lua::new();
  let func: mlua::Function = lua.load(code).eval().unwrap();
  println!("{:?}", func.call::<_, ()>(()).unwrap());
}

pub fn py_call(code:&str) {
  Python::with_gil(|py| {
    let func = PyModule::from_code_bound(py,code,"", "",)
      .unwrap()
      .getattr("function").unwrap();
    println!("{:?}", func.call0().unwrap());
  });
}


pub fn csx_call(code:&str) {
  let temp = tempfile::tempdir().unwrap();
  let path = temp.path().to_string_lossy().into_owned();
  let file_path = format!(r"{path}\temp.csx");
  {  
    use std::io::Write;
    let mut buf_writer = std::io::BufWriter::new(std::fs::File::create(&file_path).unwrap());
    buf_writer.write(code.as_bytes()).unwrap();
    buf_writer.flush().unwrap();
  } 
  let dst = std::process::Command::new("powershell")
    .args(&["dotnet", "script", &file_path])
    .stdout(std::process::Stdio::inherit())
    .output().expect("failed to execute process");

  match dst.status.success() {
    true => { println!("success"); },
    false => { println!("err {:?}", dst); }
  };
  
}

/*** scripting***/

pub trait HrawScripting {
  fn from_lua_script(&mut self, code:&str, src:&[u8], width:usize, height:usize);
  fn from_py_script(&mut self, code:&str, src:&[u8], width:usize, height:usize);
}
impl HrawScripting for [i32] {
  fn from_lua_script(&mut self, code:&str, src:&[u8], width:usize, height:usize){
    let lua = Lua::new();
    lua.call_func(src, self, width, height, code).unwrap(); // Tをジェネリクスにするため
    /* 
      let globals = lua.globals();
      globals.set("src", src).unwrap();
      let func: mlua::Function = lua.load(code).eval().unwrap();
      (0..width*height).for_each(|i|{ 
        dst[i] = func.call::<_, i32>(i).unwrap();
      });
    */
  }
  fn from_py_script(&mut self, code:&str, src:&[u8], width:usize, height:usize){
    Python::with_gil(|py| {
      let module = PyModule::from_code_bound(py, code, "", "",).unwrap();
      module.add("src", src).unwrap();
      let func = module.getattr("function").unwrap();
      (0..width*height).for_each(|i|{ 
        self[i] = func.call1((i,)).unwrap().extract::<i32>().unwrap();
      });
    });
  }
}
impl HrawScripting for [f32] {
  fn from_lua_script(&mut self, code:&str, src:&[u8], width:usize, height:usize){
    let lua = Lua::new();
    lua.call_func(src, self, width, height, code).unwrap(); // Tをジェネリクスにするため
  }
  fn from_py_script(&mut self, code:&str, src:&[u8], width:usize, height:usize){
    Python::with_gil(|py| {
      let module = PyModule::from_code_bound(py, code, "", "",).unwrap();
      module.add("src", src).unwrap();
      let func = module.getattr("function").unwrap();
      (0..width*height).for_each(|i|{ 
        self[i] = func.call1((i,)).unwrap().extract::<f32>().unwrap();
      });
    });
  }
}
impl HrawScripting for [f64] {
  fn from_lua_script(&mut self, code:&str, src:&[u8], width:usize, height:usize){
    let lua = Lua::new();
    lua.call_func(src, self, width, height, code).unwrap(); // Tをジェネリクスにするため
  }
  fn from_py_script(&mut self, code:&str, src:&[u8], width:usize, height:usize){
    Python::with_gil(|py| {
      let module = PyModule::from_code_bound(py, code, "", "",).unwrap();
      module.add("src", src).unwrap();
      let func = module.getattr("function").unwrap();
      (0..width*height).for_each(|i|{ 
        self[i] = func.call1((i,)).unwrap().extract::<f64>().unwrap();
      });
    });
  }
}



pub fn csx_call_array(code:&str, src:&[u8], _dst:&mut [i32], _width:usize, _height:usize) {
  use indoc::formatdoc;
  let len = src.len();
  let csx_code = formatdoc! {r#"
    {code}

    for(var i =0;i<{len};i++){{
      function();
    }}

    MemoryMapReader.Accessor("wuzeiMemoryMapped_header", (src)=>{{
      accessor.ReadArray(0, buf, 0, buf.Length);
    }});

    public class MemoryMapReader {{
      public static void Accessor(string addr, Action<System.IO.MemoryMappedFiles.MemoryMappedViewAccessor> act){{
        try{{
          using var mmf = System.IO.MemoryMappedFiles.MemoryMappedFile.OpenExisting(addr);
          using var accessor = mmf.CreateViewAccessor();
          act(accessor);
        }} catch ( Exception e ) {{ Console.WriteLine(e.ToString()); }}
      }}
    }}
  "#};
  csx_call(csx_code.as_str());
}



