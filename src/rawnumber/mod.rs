mod clamp;
pub mod scripting;
pub use scripting::*;
pub use clamp::*;

#[allow(non_camel_case_types)]
#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub enum BitField {
  le_u8, be_u8, le_i8, be_i8,
  le_u16, be_u16, le_i16, be_i16,
  le_u32, be_u32, le_i32, be_i32,
  le_u64, be_u64, le_i64, be_i64,
  le_f32, be_f32, le_f64, be_f64,
  unknown
}

pub trait RawNumber { }
macro_rules! impl_rawnum_strcut { ($($t:ident)*) => {
  $(
    #[allow(non_camel_case_types)]
    pub struct $t{ }
    impl RawNumber for $t{}
  )*
}}
impl_rawnum_strcut!{
  le_u8 be_u8 le_i8 be_i8
  le_u16 be_u16 le_i16 be_i16
  le_u32 be_u32 le_i32 be_i32
  le_u64 be_u64 le_i64 be_i64
  le_f32 be_f32 le_f64 be_f64
}
