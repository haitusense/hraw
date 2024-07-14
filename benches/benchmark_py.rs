#![allow(dead_code, unused_variables)]

use criterion::{Criterion, criterion_group, criterion_main};
use pyo3::prelude::*;

const SIZE :usize = 640*480usize;

fn bench_load(c: &mut Criterion) {
  let src = fake::vec![u8; SIZE * 4];
  let code = indoc::indoc! {r"
    def function(index):
      return index
  "};

  c.bench_function("load py - full", |b| b.iter(|| {
    Python::with_gil(|py| {
      let module = PyModule::from_code_bound(py, code, "", "",).unwrap();
      module.add("src", src.clone() ).unwrap();
      let func = module.getattr("function").unwrap();
      (0..SIZE).for_each(|i|{ let _  = func.call1((i,)).unwrap().extract::<i32>().unwrap(); });
    });
  }));

  Python::with_gil(|py| {
    let module = PyModule::from_code_bound(py, code, "", "",).unwrap();
    let func = module.getattr("function").unwrap();      
    c.bench_function("load py - whitout load code", |b| b.iter(|| {
      module.add("src", src.clone() ).unwrap();
      (0..SIZE).for_each(|i|{ let _  = func.call1((i,)).unwrap().extract::<i32>().unwrap(); });
    }));  
  });

  Python::with_gil(|py| {
    let module = PyModule::from_code_bound(py, code, "", "",).unwrap();
    let func = module.getattr("function").unwrap();
    module.add("src", src.clone() ).unwrap();   
    c.bench_function("load py - whitout load code, set var", |b| b.iter(|| {
      (0..SIZE).for_each(|i|{ let _  = func.call1((i,)).unwrap().extract::<i32>().unwrap(); });
    }));
  });

}

#[pyo3::prelude::pyclass]
struct PyPixel {
  pub data: Vec<u8>
}

#[pyo3::prelude::pymethods]
impl PyPixel {
  fn to_array(&self) -> &[u8] { &self.data }
  fn to_vec(&self) -> Vec<u8> { self.data.clone() }
  fn get(&self, index:usize) -> u8 { self.data[index] }
}

fn bench_call(c: &mut Criterion) {
  let size = 3000;
  let mut dst = vec![0i32; size];

  c.bench_function("py with dic", |b| b.iter(|| {
    Python::with_gil(|py| {
      let code = indoc::indoc! {r"
        def function(index):
          i = index * 4
          dst = bytearray(src[i:i+4])
          return int.from_bytes(dst, 'little', signed=True)
      "};
      let module = PyModule::from_code_bound(py, code, "", "",).unwrap();
      module.add("src", fake::vec![u8; size * 4]).unwrap();
      let func = module.getattr("function").unwrap();
      (0..size).for_each(|i|{ 
        dst[i] = func.call1((i,)).unwrap().extract::<i32>().unwrap();
      });
    });
  }));

  c.bench_function("py with get()", |b| b.iter(|| {
    Python::with_gil(|py| {
      let code = indoc::indoc! {r"
        def function(index):
          i = index * 4
          dst = bytearray([src.get(i), src.get(i+1), src.get(i+2), src.get(i+3)])
          return int.from_bytes(dst, 'little', signed=True)
      "};
      let module = PyModule::from_code_bound(py, code, "", "",).unwrap();
      module.add("src", PyPixel{ data : fake::vec![u8; size * 4] }).unwrap();
      let func = module.getattr("function").unwrap();
      (0..size).for_each(|i|{ 
        dst[i] = func.call1((i,)).unwrap().extract::<i32>().unwrap();
      });
    });
  }));

  c.bench_function("py with array", |b| b.iter(|| {
    Python::with_gil(|py| {
      let code = indoc::indoc! {r"
        def function(index):
          i = index * 4
          dst = bytearray(src.to_array()[i:i+4])
          return int.from_bytes(dst, 'little', signed=True)
      "};
      let module = PyModule::from_code_bound(py, code, "", "",).unwrap();
      module.add("src", PyPixel{ data : fake::vec![u8; size * 4] }).unwrap();
      let func = module.getattr("function").unwrap();
      (0..size).for_each(|i|{ 
        dst[i] = func.call1((i,)).unwrap().extract::<i32>().unwrap();
      });
    });
  }));

  c.bench_function("py with vec", |b| b.iter(|| {
    Python::with_gil(|py| {
      let code = indoc::indoc! {r"
        def function(index):
          i = index * 4
          dst = bytearray(src.to_vec()[i:i+4])
          return int.from_bytes(dst, 'little', signed=True)
      "};
      let module = PyModule::from_code_bound(py, code, "", "",).unwrap();
      module.add("src", PyPixel{ data : fake::vec![u8; size * 4] }).unwrap();
      let func = module.getattr("function").unwrap();
      (0..size).for_each(|i|{ 
        dst[i] = func.call1((i,)).unwrap().extract::<i32>().unwrap();
      });
    });
  }));

}


fn bench_cast(c: &mut Criterion) {
  let mut dst = vec![0i32; SIZE];
  c.bench_function("py cast", |b| b.iter(|| {
    Python::with_gil(|py| {
      let code = indoc::indoc! {r"
        def function(src):
          return int(src)
      "};
      let module = PyModule::from_code_bound(py, code, "", "",).unwrap();
      let func = module.getattr("function").unwrap();
      (0..SIZE).for_each(|i|{ 
        dst[i] = func.call1((i as f64,)).unwrap().extract::<i32>().unwrap();
      });
    });
  }));
}


criterion_group!(benches, bench_load, /*bench_call,*/ bench_cast);
criterion_main!(benches);