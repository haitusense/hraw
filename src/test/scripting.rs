
// pyo3
// pythonのモジュールが必要なので可搬性はない
#[test]
fn pyo3() -> anyhow::Result<()> {
  use pyo3::prelude::*;
  Python::with_gil(|py| {
    let hoge = PyModule::from_code_bound(py,r#"
import sys
print(sys)
print(sys.executable) #rustのexe
print(sys.version) #install済みのpythonと一致する
print(sys.path) #install済みのpythonと一致する

def hoge(n):
  return n + 1
"#,"", "",).unwrap()
    .getattr("hoge").unwrap();

    let a = hoge.call1((3,)).unwrap();
    println!("{:?}", a);
  });
  Ok(())
}


#[test]
fn mlua() -> anyhow::Result<()> {
  use mlua::prelude::*;
  let lua = Lua::new();

  let globals = lua.globals();
  let print : mlua::Function = globals.get("print")?;
  print.call::<_, ()>("hello from rust")?;

  let src = vec![1,2,3,4,5,6,7,8,9,10];
  let mut dst = vec![0i32;10];
  lua.scope(|scope| {
    let src = src.as_slice();
    let dst = dst.as_mut_slice();

    globals.set("get", scope.create_function(|_, index: usize | {
      Ok(src[index])
    })?)?;
    // globals.set("set", scope.create_function(|_, (x, y): (usize, usize)| {
    //  mut *dst[x] = y as i32;
    //   Ok(())
    // })?)?;

    let dst = lua.load("func(1, 2)").eval::<i32>();
    println!("var: {:?}", dst);


    Ok(())
  })?;
  println!("{:?}", dst);

  // lua.globals().set("src", &mut src)?;
  // lua.create_function(func)
  // lua.load("var = 123").exec().unwrap();
  // println!("var: {:?}", lua.globals().get::<_, Option<i32>>("var").unwrap());
  
  Ok(())
}


#[test]
fn mlua2() -> anyhow::Result<()> {
  use mlua::prelude::*;
  let lua = Lua::new();
  let globals = lua.globals();
  let src = vec![1,2,3,4,5,6,7,8,9,10];
  let src = src.as_slice();

  globals.set("data", src)?;
  let func: mlua::Function = lua.load(r#"
    function(a, b)
      c = data[2]
      return a + b + c
    end
  "#).eval()?;

  println!("{:?}", func.call::<_, u32>((3, 4))?);
  Ok(())
}

const TEST_FILE_I32 :&str = r".\ship_i32_unknown.zip";

#[test]
fn hraw_read_with_lua() -> anyhow::Result<()> {
  use crate::*;
  use mlua::prelude::*;
  let mut hraw = crate::Hraw::new(TEST_FILE_I32).unwrap();
  let header  = hraw.header().to_struct();
  let vec = hraw.to_vec_poi(0)?;

  let lue = Lua::new();
  let mut dst = vec![0i32; header.width * header.height];
  lue.call_func(vec.as_slice(), dst.as_mut_slice(), header.width, header.height, header.decoder.as_str());

  println!("{:?}", &dst[0..2]);
  Ok(())
}


#[test]
fn mlua_type_inferencepermalink() {
  use mlua::prelude::*;
  let lua = Lua::new();
  let globals = lua.globals();
  let src = vec![1,2,3,4,5,6,7,8,9,10];
  let src = src.as_slice();

  globals.set("data", src).unwrap();
  let func: mlua::Function = lua.load(r#"
    --!strict
    function(a : number, b : number) : number
      local val1 = bit32.lshift(0xFF, 24)
      local val2 = bit32.lshift(0xAF, 16)
      local val3 = bit32.lshift(0x45, 8)
      local val4 = bit32.lshift(0x32, 0)
      
      local value1 = bit32.bor(val1, val2, val3, val4)
      print(string.format("0x%016X", value1))
      if value1 > 0x7FFF_FFFF then
        value1 = -(0xFFFF_FFFF - value1 + 1)
      end
      print(string.format("0x%016X", value1))
      
      local value2 = bit32.bor(val1, val2, val3, val4)
      value2 = bit32.lshift(value2, 12)
      print(string.format("0x%016X", value2))
      value2 = bit32.rshift(value2, 12)
      print(string.format("0x%016X", value2))

      local buf = buffer.create(4)
      print(string.format("len %d", buffer.len(buf)))
      buffer.writeu8(buf, 3, 0xFF)
      buffer.writeu8(buf, 2, 0xAF)
      buffer.writeu8(buf, 1, 0x45)
      buffer.writeu8(buf, 0, 0x32)

      print( string.format("i32 %d", buffer.readi32(buf, 0)) )
      print( string.format("u32 %d", buffer.readu32(buf, 0)) )
      print( string.format("0x%X", buffer.readi32(buf, 0)) )
      print( string.format("%d", buffer.readf32(buf, 0)) )
      return value1;
    end
  "#).eval().unwrap();

  println!("{:?}", func.call::<_, i32>((3, 4)).unwrap());
}


