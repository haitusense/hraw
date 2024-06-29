// pyo3 : pythonのモジュールが必要なので可搬性はない
// mmfとVirtualAllocExどっちにしよう

#[test]
fn env() -> anyhow::Result<()> {
  use indoc::indoc;

  let info_lua = indoc! {"
    --!strict
    function(a : number) : number
      print(_VERSION)
      for k, v in pairs(_G) do
        print(k, v)
      end
      return 1
    end
  "};
  crate::rawnumber::scripting::lua_call(info_lua);

  let info_py = indoc! {"
    import sys
    print(sys)
    print(sys.executable) #rustのexe
    print(sys.version)    #install済みのpythonと一致する
    print(sys.path)       #install済みのpythonと一致する
    
    def function():
      return 1
  "};
  crate::rawnumber::scripting::py_call(info_py);

  let info_csx = indoc! {r#"
    using System;
    using System.Collections;
    
    Console.WriteLine($".Net Version: {Environment.Version}");
    Console.WriteLine($"OSVersion: {Environment.OSVersion}");
    Console.WriteLine("GetEnvironmentVariables: ");
    IDictionary environmentVariables = Environment.GetEnvironmentVariables();
    foreach (DictionaryEntry de in environmentVariables) {
      Console.WriteLine($"  {de.Key} = {de.Value}");
    }

    public void function() {
      Console.WriteLine("function");
    }

    function();
  "#};
  crate::rawnumber::scripting::csx_call(info_csx);

  Ok(())
}


#[test]
fn bitwise_operation() {
  use indoc::indoc;
  use fake::{Fake, Faker};
  use mlua::prelude::*;
  use pyo3::prelude::*;

  let bitwize_lua = indoc! {"
    --!strict
    function(a : number, b : number, c : number, d : number) : number
      local buf = buffer.create(4)
      buffer.writeu8(buf, 0, a)
      buffer.writeu8(buf, 1, b)
      buffer.writeu8(buf, 2, c)
      buffer.writeu8(buf, 3, d)
      return buffer.readi32(buf, 0);
    end
  "};

  let bitwize_lua_2 = indoc! {r#"
    --!strict
    function(a : table) : number
      local buf = buffer.create(4)
      print(a, buf)
      buffer.writeu8(buf, 0, a[1])
      buffer.writeu8(buf, 1, a[2])
      buffer.writeu8(buf, 2, a[3])
      buffer.writeu8(buf, 3, a[4])
      return buffer.readi32(buf, 0);
    end
  "#};

  let bitwize_py = indoc! {"
    def func(a, b, c, d):
      a = a << 0
      b = b << 8
      c = c << 16
      d = d << 24
      byts = (a | b | c | d).to_bytes(4, 'little')
      return int.from_bytes(byts, 'little', signed=True)
  "};

  let bitwize_py_2 = indoc! {"
    def func(a, b, c, d):
      dst = bytearray([a,b,c,d])
      return int.from_bytes(dst, 'little', signed=True)
  "};

  let bitwize_py_3 = indoc! {"
    import struct
    def func(a, b, c, d):
      dst = bytearray([a,b,c,d])
      return struct.unpack_from('<i', dst, 0)[0]
  "};
  
  let src = vec![1,2,3,4,5,6,7,8,9,10];

  let list = (0..100)
    .map(|_| Faker.fake::<[u8;4]>())
    .collect::<Vec<_>>();
  list.into_iter().for_each(|n| {
    let src = i32::from_le_bytes(n);
    {
      let lua = Lua::new();
      let func: mlua::Function = lua.load(bitwize_lua).eval().unwrap();
      let dst = func.call::<_, i32>((n[0], n[1], n[2], n[3])).unwrap();
      assert_eq!( src, dst );
    }
    {
      let lua = Lua::new();
      let func: mlua::Function = lua.load(bitwize_lua_2).eval().unwrap();
      let dst = func.call::<_, i32>(n).unwrap();
      assert_eq!( src, dst );
    }
    {
      Python::with_gil(|py| {
        let func = PyModule::from_code_bound(py,bitwize_py,"", "",).unwrap()
          .getattr("func").unwrap();
        let dst = func.call1((n[0], n[1], n[2], n[3])).unwrap().extract::<i32>().unwrap();
        assert_eq!( src, dst );
      });
    }
    {
      Python::with_gil(|py| {
        let func = PyModule::from_code_bound(py,bitwize_py_2,"", "",).unwrap()
          .getattr("func").unwrap();
        let dst = func.call1((n[0], n[1], n[2], n[3])).unwrap().extract::<i32>().unwrap();
        assert_eq!( src, dst );
      });
    }
    {
      use pyo3::prelude::*;
      Python::with_gil(|py| {
        let func = PyModule::from_code_bound(py,bitwize_py_3,"", "",).unwrap()
          .getattr("func").unwrap();
        let dst = func.call1((n[0], n[1], n[2], n[3])).unwrap().extract::<i32>().unwrap();
        assert_eq!( src, dst );
      });
    }
  });
}


#[test]
fn bitwise_operation_from_array() {
  use indoc::indoc;
  // use fake::{Fake, Faker};

  let lua = indoc! {"
    --!strict
    function(index : number) : number
      local i = index * 4 + 1;
      print(src)
      local buf = buffer.create(4)
      buffer.writeu8(buf, 0, src[i])
      buffer.writeu8(buf, 1, src[i + 1])
      buffer.writeu8(buf, 2, src[i + 2])
      buffer.writeu8(buf, 3, src[i + 3])
      return buffer.readi32(buf, 0);
    end
  "};

  let py = indoc! {"
    def function(index):
      i = index * 4;
      print('addr :', id(src))
      dst = bytearray([src[i], src[i+1], src[i+2], src[i+3]])
      return int.from_bytes(dst, 'little', signed=True)
  "};

  use crate::*;
  let src = vec![1,2,3,4,5,6,7,8,9,10,11,12];
  let mut dst = vec![0i32;src.len()];
  dst.from_lua_script(lua, src.as_slice(), 3, 1);
  println!("{:?}", dst);

  dst.from_py_script(py, src.as_slice(), 3, 1);
  println!("{:?}", dst);

}


#[test]
fn plot() {
  use indoc::indoc;
  
  let plot_py = indoc! {r#"
    import seaborn as sns
    import numpy as np
    import pandas as pd
    import matplotlib.pyplot as plt

    def func(a, b, c, d):
      iris = sns.load_dataset("iris")
      sns.pairplot(iris)
      plt.show()
  "#};
  
  let src = vec![1,2,3,4,5,6,7,8,9,10];
  {
    use pyo3::prelude::*;
    Python::with_gil(|py| {
      let func = PyModule::from_code_bound(py,plot_py,"", "",).unwrap()
        .getattr("func").unwrap();
      let dst = func.call1((src[0], src[1], src[2], src[3])).unwrap().extract::<i32>().unwrap();
    });
  }
}

/*
mlua
*/
#[test]
fn mlua() -> anyhow::Result<()> {
  use mlua::prelude::*;

  let src = vec![1,2,3,4,5,6,7,8,9,10];
  let mut dst = vec![0i32;10];
  
  let lua = Lua::new();
  lua.scope(|scope| {
    let src = src.as_slice();
    let dst = dst.as_mut_slice();

    let globals = lua.globals();
    let print : mlua::Function = globals.get("print")?;
    print.call::<_, ()>("hello from rust")?;

    lua.globals().set("src", src)?;
    // lua.create_function(func)
    // lua.load("var = 123").exec().unwrap();
    // println!("var: {:?}", lua.globals().get::<_, Option<i32>>("var").unwrap());

    globals.set("get", scope.create_function(|_, index: usize | { Ok(src[index]) })?)?;
    globals.set("set", scope.create_function(|_, (x, y): (usize, usize)| { /* mut *dst[x] = y as i32; */ Ok(()) })?)?;

    let dst = lua.load("func(1, 2)").eval::<i32>();
    println!("var: {:?}", dst);

    Ok(())
  })?;
  println!("{:?}", dst);
  
  Ok(())
}

/*
eval_bound
run_bound
from_code_bound
*/
#[test]
fn pyo3() {
  use indoc::indoc;
  let py_eval = indoc! {"
    lambda x, y: x + y
  "};

  let py_code_bound = indoc! {r#"
    print("call func in py")
    def func(bytes):
      print(bytes)
      print(src)
      bytes[1] = 3
      return 1
  "#};

  let py_code_run = indoc! {r#"
    print("run_bound in py")
    "a"
  "#};


  let src = [1u8, 2u8, 3u8, 4u8];
  let src2 = [3u8, 4u8, 5u8, 6u8];
  use pyo3::prelude::*;
  println!("Python::with_gil");
  Python::with_gil(|py| {

    /*** eval_bound ***/
    let eval_bound = py.eval_bound(py_eval, None, None).unwrap();
    let eval_bound_result = eval_bound.call1((1, 2));
    println!("eval_bound_result : {:?}", eval_bound_result);

    /*** run_bound ***/
    println!("run_bound : called func");
    let dst = py.run_bound(py_code_run, None, None);
    println!("run_bound : {:?}", dst);

    /*** from_code_bound ***/
    let module = PyModule::from_code_bound(py,py_code_bound,"", "",).unwrap();
    module.add("src", src2).unwrap();
    let func = module.getattr("func").unwrap();
    println!("from_code_bound : called func");
    let dst = func.call1((src,)).unwrap().extract::<i32>().unwrap();
    println!("from_code_bound : {:?}", dst);

    /*** eval_bound with stdout ***/
    let sys = py.import_bound("sys").unwrap();
    sys.setattr("stdout", LoggingStdout.into_py(py)).unwrap();
    println!("from_code_bound with stdout : called func");
    let dst = func.call1((src,)).unwrap().extract::<i32>().unwrap();
    println!("from_code_bound with stdout :{:?}", dst);

    /*** run_bound ***/
    println!("run_bound with stdout : called func");
    let dst = py.run_bound(py_code_run, None, None);
    println!("run_bound : {:?}", dst);
  });

}

#[pyo3::prelude::pyclass]
struct LoggingStdout;

#[pyo3::prelude::pymethods]
impl LoggingStdout {
  fn write(&self, src: &str) { 
    match src {
      "\n" => println!(""),
      _ => print!("\x1b[43m{}\x1b[49m", src.replace('\n', "\r\n"))
    }
  }
}

#[test]
fn csx() {
  use indoc::indoc;
  use std::io::Write;

  // let tempfile = tempfile::Builder::new().suffix(".csx").tempfile().unwrap();
  // let file_name = tempfile.path().to_string_lossy();

  let temp = tempfile::tempdir().unwrap();
  let path = temp.path().to_string_lossy().into_owned();
  let file_path = format!(r"{path}\temp.csx");

  let csx_code = indoc! {r#"
    Console.WriteLine("a");
  "#};
  {
    let mut buf_writer = std::io::BufWriter::new(std::fs::File::create(&file_path).unwrap());
    buf_writer.write(csx_code.as_bytes()).unwrap();
    buf_writer.flush().unwrap();
  }
  let dst = std::process::Command::new("powershell")
    .args(&["dotnet", "script", &file_path])
    .output().expect("failed to execute process");
  //   let dst = std::process::Command::new("powershell")
  //     .args(&["-ExecutionPolicy", "Bypass", "-File", file_path.as_str()])
  //     .spawn().unwrap()
  //     .wait().unwrap();
  println!("{:?}", dst);
}


/*
scripting in hraw
*/
#[test]
fn hraw_read_with_scripting() -> anyhow::Result<()> {
  use crate::*;
  {
    const TEST_FILE_I32 :&str = r".\src\test\pics\lua.zip";
    let mut hraw = crate::Hraw::new(TEST_FILE_I32).unwrap();
    let header  = hraw.header().to_struct();
    let vec = hraw.to_vec_poi(0)?;
    let mut dst = vec![0i32; header.width * header.height];
    let decoder = header.decoder.unwrap();
    println!("lang : {}", decoder.lang);
    println!("code : \r\n{}", decoder.code);
    dst.from_lua_script(decoder.code.as_str(), vec.as_slice(), header.width, header.height);
    println!("{:?}", &dst);
  }
  {
    const TEST_FILE_I32 :&str = r".\src\test\pics\py.zip";
    let mut hraw = crate::Hraw::new(TEST_FILE_I32).unwrap();
    let header  = hraw.header().to_struct();
    let vec = hraw.to_vec_poi(0)?;
    let mut dst = vec![0i32; header.width * header.height];
    let decoder = header.decoder.unwrap();
    println!("lang : {}", decoder.lang);
    println!("code : \r\n{}", decoder.code);
  
    dst.from_py_script(decoder.code.as_str(), vec.as_slice(), header.width, header.height);
    println!("{:?}", &dst);
  }

  {
    use crate::buffer::FromHraw;

    // const TEST_FILE_I32 :&str = r".\src\test\pics\py.zip";
    // let mut hraw = crate::Hraw::new(TEST_FILE_I32).unwrap();
    // let header  = hraw.header().to_struct();
    // let mut dst = vec![0i32; header.width * header.height];

    // dst.as_mut_slice().from_hraw(TEST_FILE_I32, 0);
    // println!("0 : {:?}", &dst[0..2]);
    // dst.as_mut_slice().from_hraw(TEST_FILE_I32, 1);
    // println!("1 : {:?}", &dst[0..2]);
    // dst.as_mut_slice().from_hraw(TEST_FILE_I32, "data.raw");
    // println!("data.raw : {:?}", &dst[0..2]);
    // dst.as_mut_slice().from_hraw(TEST_FILE_I32, "subdir/1.raw");
    // println!("subdir/1.raw : {:?}", &dst[0..2]);

  }
  Ok(())
}


#[test]
fn powershell() -> anyhow::Result<()> {
  use std::process::Stdio;
  let outputs = std::fs::File::create("out.txt")?;
  let errors = outputs.try_clone()?;

  let dst = std::process::Command::new("powershell")
    .args(&["ls"])
    .stdout(Stdio::from(outputs))
    .stderr(Stdio::from(errors))
    .spawn().unwrap()
    .wait().unwrap();
  
  println!("{:?}", dst);
  
  Ok(())
}



