const ERR_FILE :&str = r".\err.zip";
const TEST_FILE_I32 :&str = r".\ship_i32.zip";
const TEST_FILE_U16 :&str = r".\ship_u16.zip";
const TEST_FILE_UNKNOWN :&str = r".\ship_i32_unknown.zip";

#[test]
fn hraw_header() -> anyhow::Result<()> {
  use crate::*;

  /* ファイル構造, headerのinfomationの表示 */
  let mut hraw = crate::Hraw::new(TEST_FILE_I32)?;
  hraw.info().unwrap();

  /* headerをserde_json::Valueで取得, デシリアライズ*/
  let mut hraw = crate::Hraw::new(TEST_FILE_I32)?;
  let value = hraw.header();
  let width = value["width"].as_u64().unwrap_or_default();
  let height = value["height"].as_u64().unwrap_or_default();
  let list = value["data"].as_array().unwrap();
  println!("value : {:?}", value);
  println!("{} {} {:?} {:?}", width, height, list, list[1].as_str());
  
  /* dataの存在チェック */
  println!("{:?}", hraw.contain_poi("data.raw"));
  println!("{:?}", hraw.contain_poi("subdir/1.raw"));
  println!("{:?}", hraw.contain_poi(0));
  println!("{:?}", hraw.contain_poi(1));
  println!("{:?}", hraw.contain_poi(2));
  println!("{:?}", hraw.contain_poi("hoge.raw"));

  /* headerをstrcutで取得 */
  let mut hraw = crate::Hraw::new(TEST_FILE_I32)?;
  let value = hraw.header().to_struct();
  println!("value : {:?}", value);

  /* headerでのerr */
  let mut hraw = crate::Hraw::new(ERR_FILE)?;
  hraw.info()?;
  let value = hraw.header();
  println!("value : {:?}", value);

  Ok(())
}

#[allow(deprecated)]
#[test]
fn hraw_data() -> anyhow::Result<()> {
  use crate::*;
  use crate::buffer::*;

  let mut hraw = crate::Hraw::new(TEST_FILE_I32).unwrap();

  /* 一括読み込み (u8) */
  let vec = hraw.to_vec("data.raw");
  let vec = hraw.to_vec("data.raw");
  let vec = hraw.to_vec_poi(0);
  
  /* stream 所有権は奪ったままなので使いにくい */
  let mut stream = hraw.to_stream("data.raw")?;
  let mut buf : [u8; 4] = unsafe { std::mem::MaybeUninit::zeroed().assume_init() };
  for _i in 0..3 {
    stream.read_exact(&mut buf)?;
    let value = i32::from_le_bytes(buf);
  }

  /* iterator (BitField指定) */
  let mut hraw = crate::Hraw::new(TEST_FILE_I32).unwrap();
  for (index, dst) in hraw.enumerate_path::<le_i32>("data.raw").map(|n| (n.0, n.1 as i32)) {
    if index < 3 || 640 * 480 - 3 < index { println!("{} : {} {} {}",dst ,index, index/640, index%640 )}
  }

  /* iteratorのwrapper (BitField自動) */
  let mut vec_i32 = vec![0i32; 640*480];
  vec_i32.as_mut_slice().from_hraw(TEST_FILE_I32, "data.raw");

  Ok(())
}

#[test]
fn hraw_data_unknown() -> anyhow::Result<()> {
  use crate::buffer::*;

  let mut vec_i32_1 = vec![0i32; 640*480];
  let mut vec_i32_2 = vec![0i32; 640*480];

  vec_i32_1.as_mut_slice().from_hraw(TEST_FILE_I32, "data.raw");
  vec_i32_2.as_mut_slice().from_hraw(TEST_FILE_UNKNOWN, "data.raw");

  println!("{:?}", &vec_i32_1[0..3]);
  println!("{:?}", &vec_i32_2[0..3]);

  Ok(())
}