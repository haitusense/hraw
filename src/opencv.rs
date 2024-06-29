use opencv::{core, highgui, imgcodecs, imgproc, prelude::*};

fn run() -> anyhow::Result<()> {
  let image = imgcodecs::imread("test.jpg", 0)?;
  highgui::named_window("hello opencv!", 0)?;
  highgui::imshow("hello opencv!", &image)?;
  highgui::wait_key(100000)?;
  Ok(())
}

const TEST_FILE_I32 :&str = r".\ship_i32_unknown.zip";

#[cfg(test)]
pub mod test {

  // cargo test opencv --all-features
  #[test]
  fn opencv() -> anyhow::Result<()> {
    use crate::buffer::*;;
    use crate::opencv::*;

    let mut hraw = crate::Hraw::new(TEST_FILE_I32).unwrap();
    let value = hraw.header();
    let width = value["width"].as_u64().unwrap_or_default();
    let height = value["height"].as_u64().unwrap_or_default();

    let mut vec_i32 = vec![0i32; 640*480];
    let mut vec_u8 = vec![0u8; 640*480];
    vec_i32.as_mut_slice().from_hraw(TEST_FILE_I32, "data.raw");

    vec_i32.iter().zip(vec_u8.iter_mut()).for_each(|(a, b)|{ 
      *b = num::clamp(*a >> 8 , u8::MIN as i32, u8::MAX as i32) as u8; 
    });

    let src = unsafe {
      Mat::new_rows_cols_with_data( height as i32, width as i32, u8::opencv_type(), 
      vec_u8.as_mut_ptr().cast::<std::ffi::c_void>(), core::Mat_AUTO_STEP
      )?
    };
  
    let mut dst = Mat::default();
    imgproc::cvt_color(&src, &mut dst, imgproc::COLOR_BayerBG2BGR, 0)?;
    dst.

    Ok(())
  }

}