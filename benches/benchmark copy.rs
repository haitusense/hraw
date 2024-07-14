// cargo bench --bench benchmark -- group1
// cargo bench benchmark_lua で実行

use criterion::{Criterion, criterion_group, criterion_main};
use mlua::prelude::*;
use pyo3::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

fn bench(c: &mut Criterion) {
  let mut dst = vec![0i32; SIZE];
  
  c.bench_function("rust", |b| b.iter(|| {
    let src = fake::vec![u8; SIZE * 4];
    (0..SIZE).for_each(|i|{ 
      dst[i] = i32::from_be_bytes([src[i*4], src[i*4+1], src[i*4+2], src[i*4+3]]);
    });
  }));

  c.bench_function("lua", |b| b.iter(|| {
    let lua = Lua::new();
    let globals = lua.globals();
    globals.set("src", ScriptStrcut{ data: fake::vec![u8; SIZE * 4] }).unwrap();
    let func: mlua::Function = lua.load(indoc::indoc! {r"
      function(index)
        buf = buffer.create(4)
        for n = 0, 3 do
          buffer.writeu8(buf, n, src[index * 4 + n])
        end
        return buffer.readi32(buf, 0)
      end
    "}).eval().unwrap();
    (0..SIZE).for_each(|i|{ dst[i] = func.call::<_, i32>(i).unwrap(); });
  }));

  c.bench_function("py", |b| b.iter(|| {
    Python::with_gil(|py| {
      let module = PyModule::from_code_bound(py, indoc::indoc! {r"
        def function(index):
          i = index * 4
          dst = bytearray(src.data[i:i+4])
          return int.from_bytes(dst, 'little', signed=True)
      "}, "", "",).unwrap();
      module.add("src", ScriptStrcut{ data: fake::vec![u8; SIZE * 4] }).unwrap();
      let func = module.getattr("function").unwrap();
      (0..SIZE).for_each(|i|{ 
        dst[i] = func.call1((i,)).unwrap().extract::<i32>().unwrap();
      });
    });
  }));

}


#[allow(dead_code)]
fn bench_lua3(c: &mut Criterion) {
  let size = 640*480usize;
  let src = fake::vec![u8; size * 4];
  let mut dst = vec![0i32; size];
  let slice_dst = Rc::new(RefCell::new(dst.as_mut_slice()));
  let code = indoc::indoc! {r"
    function(size)
      local buf = buffer.create(4)
      for index = 0, size - 1 do
        for m = 0, 3 do
          buffer.writeu8(buf, m, get(index * 4 + m))
        end
        set(index, buffer.readi32(buf, 0))
      end
      return 0
    end
  "};
  c.bench_function("lua with get/set()", |b| b.iter(|| {
    let lua = Lua::new();
    lua.scope(|scope| {
      let globals = lua.globals();
      globals.set("get", scope.create_function(|_, index: usize | { 
        let dst = src[index];
        Ok(dst)
      })?)?;
      globals.set("set", scope.create_function(|_, (index, val): (usize, i32)| { 
        slice_dst.borrow_mut()[index] = val;
        Ok(())
      })?)?;
      let func: mlua::Function = lua.load(code).eval().unwrap();
      let _ = func.call::<_, i32>(size).unwrap();
      Ok(())
    }).unwrap();
  }));
}



criterion_group!(benches, bench);
criterion_main!(benches);