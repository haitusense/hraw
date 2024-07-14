/*
  cargo run --example hraw
*/

use byteorder::{LittleEndian, ReadBytesExt};
use display::Indent;
use hraw::prelude::*;
use std::io::Read;

const TEST_DIR :&str = r".\samples\i32";
const TEST_FILE_I32 :&str = r".\samples\i32.zip";
const TEST_FILE_LUA :&str = r".\samples\lua.zip";

fn main() -> anyhow::Result<()> {

  // info
  let mut hraw = Hraw::new(TEST_DIR)?;
  let info = display::obj_to_yaml(hraw.info()?)?;
  println!("---info ({})---\r\n{}", TEST_DIR, info.indent(2));

  let mut hraw = Hraw::new(TEST_FILE_I32)?;
  let info = display::obj_to_yaml(hraw.info()?)?;
  println!("---info ({})---\r\n{}", hraw.path_str(), info.indent(2));

  // header
  let hraw = Hraw::new(TEST_FILE_LUA)?;
  println!("---header---\r\n{:?}", hraw.header());
  println!("---header---\r\n{}", display::obj_to_yaml(hraw.header())?.indent(2));
  println!("---header raw---\r\n{}", display::obj_to_json(hraw.header().raw)?);

  // dataの存在確認
  let mut hraw = Hraw::new(TEST_FILE_I32)?;

  println!("{:?}", hraw.contain("000.raw")?);
  println!("{:?}", hraw.contain(0)?);

  // Vecへのdata読み込み
  println!("{:?}", hraw.to_bytes(0));
  println!("{:?}", hraw.to_vec_i32(0));

  // 手動cast
  let mut reader = hraw.to_bufread(0)?;
  let mut buf : [u8; std::mem::size_of::<i64>()] = unsafe { std::mem::MaybeUninit::zeroed().assume_init() };
  for _ in 0..2 { let _ = reader.read_exact(&mut buf); }
  for _ in 0..3 {
    let n = reader.read_i32::<LittleEndian>()?;
    println!("{}", n);
  }

  let mut hraw = Hraw::new(TEST_FILE_I32)?;
  for i in hraw.enumerate::<_, le_i32>(0)? {
    println!("{} : {}", i.0, i.1);
  }
  Ok(())
}