// use anyhow::Context as _;
use mlua::prelude::*;

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
