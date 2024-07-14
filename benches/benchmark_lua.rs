use criterion::{Criterion, criterion_group, criterion_main};
use mlua::prelude::*;

const SIZE :usize = 640*480usize;

#[pyo3::prelude::pyclass]
pub struct LuaStrcut {
  pub data : Vec<u8>
}

impl mlua::UserData for LuaStrcut {
  fn add_fields<'lua, F: mlua::UserDataFields<'lua, Self>>(_fields: &mut F) {
  }
  fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
    methods.add_meta_method(mlua::MetaMethod::Index, |_, this, index: usize| {
      Ok(this.data[index])
    });
  }
}

/*
  setのコストとcallのコストはほぼ同等。単純にキャスト分
*/
fn bench_load(c: &mut Criterion) {
  let src = fake::vec![u8; SIZE * 4];

  c.bench_function("load lua - full", |b| b.iter(|| {
    let lua = Lua::new();
    lua.globals().set("src", src.clone()).unwrap();
    let func: mlua::Function = lua.load(indoc::indoc! {r"
      function(index)
        return index
      end
    "}).eval().unwrap();
    (0..SIZE).for_each(|i|{ let _ = func.call::<_, i32>(i).unwrap(); });
  }));

  c.bench_function("load lua - full call func", |b| b.iter(|| {
    let lua = Lua::new();
    lua.globals().set("src", src.clone()).unwrap();
    lua.load(indoc::indoc! {r"
      function main(index)
        return index
      end
    "}).exec().unwrap();
    (0..SIZE).for_each(|i|{ let _ = lua.globals().call_function::<_, i32>("main", i).unwrap(); });
  }));

  c.bench_function("load lua - full call func2", |b| b.iter(|| {
    let lua = Lua::new();
    lua.globals().set("src", src.clone()).unwrap();
    lua.load(indoc::indoc! {r"
      function main(index)
        return index
      end
    "}).exec().unwrap();
    let func: mlua::Function = lua.globals().get("main").unwrap();
    (0..SIZE).for_each(|i|{ let _ = func.call::<_, i32>(i).unwrap(); });
  }));

  let lua = Lua::new();
  let func: mlua::Function = lua.load(indoc::indoc! {r"
    function(index)
      return index
    end
  "}).eval().unwrap();
  c.bench_function("load lua - whitout load code", |b| b.iter(|| {
    lua.globals().set("src", src.clone()).unwrap();
    (0..SIZE).for_each(|i|{ let _ = func.call::<_, i32>(i).unwrap(); });
  }));

  lua.globals().set("src", src.clone()).unwrap();
  c.bench_function("load lua - whitout load code, set var", |b| b.iter(|| {
    (0..SIZE).for_each(|i|{ let _ = func.call::<_, i32>(i).unwrap(); });
  }));

}

/*
  strcut > function > table
  30msずつ
  indexのcast, 戻り値のcast分
*/
fn bench_call(c: &mut Criterion) {
  let src = fake::vec![u8; SIZE * 4];
  let mut dst = vec![0i32; SIZE];
  
  let mut memory = 0usize;
  c.bench_function("lua with table", |b| b.iter(|| {
    let lua = Lua::new();
    let globals = lua.globals();
    globals.set("src", src.as_slice()).unwrap();
    let func: mlua::Function = lua.load(indoc::indoc! {r"
      function(index)
        buf = buffer.create(4)
        for n = 0, 3 do
          buffer.writeu8(buf, n, src[index * 4 + n + 1])
        end
        return buffer.readi32(buf, 0)
      end
    "}).eval().unwrap();
    let m = lua.used_memory();
    memory = if m > memory { m } else { memory };
    (0..SIZE).for_each(|i|{ dst[i] = func.call::<_, i32>(i).unwrap(); });
  }));
  println!("memory : {}", memory);

  c.bench_function("lua with table, global buffer", |b| b.iter(|| {
    let lua = Lua::new();
    lua.globals().set("src", src.as_slice()).unwrap();
    lua.globals().set("buf", lua.create_buffer(vec![0u8;4]).unwrap()).unwrap();
    let func: mlua::Function = lua.load(indoc::indoc! {r"
      function(index)
        for n = 0, 3 do
          buffer.writeu8(buf, n, src[index * 4 + n + 1])
        end
        return buffer.readi32(buf, 0)
      end
    "}).eval().unwrap();
    let m = lua.used_memory();
    memory = if m > memory { m } else { memory };
    (0..SIZE).for_each(|i|{ dst[i] = func.call::<_, i32>(i).unwrap(); });
  }));

  let mut memory = 0usize;
  c.bench_function("lua with buffer direct", |b| b.iter(|| {
    let lua = Lua::new();
    lua.globals().set("src", lua.create_buffer(fake::vec![u8; SIZE * 4]).unwrap()).unwrap();
    let func: mlua::Function = lua.load(indoc::indoc! {r"
      function(index)
        return buffer.readi32(src, index * 4)
      end
    "}).eval().unwrap();
    let m = lua.used_memory();
    memory = if m > memory { m } else { memory };
    (0..SIZE).for_each(|i|{ dst[i] = func.call::<_, i32>(i).unwrap(); });
  }));
  println!("memory : {}", memory);

  let mut memory = 0usize;
  c.bench_function("lua with function", |b| b.iter(|| {
    let lua = Lua::new();
    lua.scope(|scope| {
      lua.globals().set("get", scope.create_function(|_, index: usize | { 
        let dst = src[index]; // let dst = slice.borrow()[index]; +6% 誤差範囲
        Ok(dst)
      })?)?;
      let func: mlua::Function = lua.load(indoc::indoc! {r"
        function(index)
          local buf = buffer.create(4)
          for n = 0, 3 do
            buffer.writeu8(buf, n, get(index * 4 + n))
          end
          return buffer.readi32(buf, 0)
        end
      "}).eval().unwrap();
      let m = lua.used_memory();
      memory = if m > memory { m } else { memory };
      (0..SIZE).for_each(|i|{ dst[i] = func.call::<_, i32>(i).unwrap(); });
      Ok(())
    }).unwrap();
  }));
  println!("memory : {}", memory);

  let mut memory = 0usize;
  c.bench_function("lua with buffer", |b| b.iter(|| {
    let lua = Lua::new();
    lua.globals().set("src", lua.create_buffer(fake::vec![u8; SIZE * 4]).unwrap()).unwrap();
    let func: mlua::Function = lua.load(indoc::indoc! {r"
      function(index)
        local buf = buffer.create(4)
        for n = 0, 3 do
          buffer.writeu8(buf, n, buffer.readu8(src, index * 4 + n))
        end
        return buffer.readi32(buf, 0)
      end
    "}).eval().unwrap();
    let m = lua.used_memory();
    memory = if m > memory { m } else { memory };
    (0..SIZE).for_each(|i|{ dst[i] = func.call::<_, i32>(i).unwrap(); });
  }));
  println!("memory : {}", memory);

  let mut memory = 0usize;
  c.bench_function("lua with struct", |b| b.iter(|| {
    let lua = Lua::new();
    let globals = lua.globals();
    globals.set("src", LuaStrcut { data: fake::vec![u8; SIZE * 4] }).unwrap();
    let func: mlua::Function = lua.load(indoc::indoc! {r"
      function(index)
        local buf = buffer.create(4)
        for n = 0, 3 do
          buffer.writeu8(buf, n, src[index * 4 + n])
        end
        return buffer.readi32(buf, 0)
      end
    "}).eval().unwrap();
    let m = lua.used_memory();
    memory = if m > memory { m } else { memory };
    (0..SIZE).for_each(|i|{ dst[i] = func.call::<_, i32>(i).unwrap(); });
  }));
  println!("memory : {}", memory);
}


fn bench_cast(c: &mut Criterion) {
  c.bench_function("lua cast", |b| b.iter(|| {
    let lua = Lua::new();
    let func: mlua::Function = lua.load(indoc::indoc! {r"
      function(src)
        return math.floor(src * 100)
      end
    "}).eval().unwrap();
    (0..SIZE).for_each(|i|{ let _ = func.call::<_, i32>(i as f64).unwrap(); });
  }));
}



criterion_group!(benches, bench_load, bench_call, bench_cast);
criterion_main!(benches);