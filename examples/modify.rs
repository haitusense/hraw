/*
  cargo run --example modify
*/

use std::sync::{Arc,Mutex};
use hraw::prelude::*;
use serde_json::json;

const TEST_FILE_I32 :&str = r".\samples\i32.zip";

fn main() -> anyhow::Result<()> {

  let mut hraw = Hraw::new(TEST_FILE_I32)?;

  // PixelへのCast
  println!("{:?}", hraw.to_vec_i32(0)?.into_pixel(8,8));
  println!("{:?}", hraw.to_pixel(0));

  // py
  let pixel = hraw.to_pixel(0)?;
  let data = Arc::new(Mutex::new(pixel));

  let dst: serde_json::Value = data.modify_with_py_stdout(
    None, 
    indoc::indoc! {r"
      def main(arg):
        print(arg['a'])
        len = pixel.width() * pixel.height()
        for i in range(1, len, 1):
          print(i)
        return { 'detail' : 'success' }
    "},
    json!({'a' : 1}), 
    |val| println!("py : {}", val)
  )?;

  println!("{}", dst);

  // lua
  let dst: serde_json::Value = data.modify_with_lua_stdout(
    None,
    indoc::indoc! {r"
      function(args)
        print(args['a'])
        return { result = 100 }
      end
    "},
    json!({'a' : 1}), 
    |val| println!("lua : {}", val)
  )?;
  println!("{}", dst);

  let dst: serde_json::Value = data.modify_with_lua(
    "main",
    indoc::indoc! {r"
      pixeltable = pixel:to_table()
      function main(args)
        local dst = 0
        local len = pixel:width() * pixel:height()
        for i = 0, len - 1, 1 do  -- 0..=size
          dst += pixel[i]
          local s = string.format('%d, %d, %d', i, pixel[i], pixeltable[i + 1]) 
          print(s)
        end
        return { result = dst }
      end
    "},
    json!({'a' : 1})
  )?;
  println!("{}", dst);
  Ok(())
}