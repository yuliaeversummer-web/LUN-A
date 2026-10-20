use serde::Serialize;
#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct MediaError { pub code: String, pub message: String }
impl MediaError { pub fn new(code: &str, message: &str) -> Self { Self { code: code.into(), message: message.into() } } }
#[cfg(windows)] impl From<windows::core::Error> for MediaError {
 fn from(error: windows::core::Error) -> Self { Self::new(&format!("0x{:08X}",error.code().0 as u32), "Windows не предоставил медиаданные. Сессия могла завершиться; обновите обнаружение.") }
}
pub type Result<T> = std::result::Result<T,MediaError>;
pub fn id(value: &str) -> Result<()> {
 if value.is_empty() || value.len()>128 || value.contains('\0') { Err(MediaError::new("invalid_id","Некорректный идентификатор медиасессии.")) } else { Ok(()) }
}
pub fn position(value: f64, duration: i64, min: i64, max: i64) -> Result<i64> {
 if !value.is_finite() || value<0.0 || duration<=0 || value>duration as f64 || value<min as f64 || value>max as f64 {
  Err(MediaError::new("invalid_position","Позиция за пределами доступного диапазона композиции."))
 } else { Ok(value.round() as i64) }
}
#[cfg(test)] mod tests {
 use super::*;
 #[test] fn validates_seek_and_ids() {
  assert_eq!(position(1500.0,10000,0,10000).unwrap(),1500);
  for p in [f64::NAN,f64::INFINITY,-1.0,10001.0] {assert!(position(p,10000,0,10000).is_err());}
  assert!(position(0.0,0,0,0).is_err()); assert!(position(500.0,10000,1000,9000).is_err()); assert!(id("").is_err());
 }
}
