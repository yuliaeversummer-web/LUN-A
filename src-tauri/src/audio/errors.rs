use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioError { pub code: String, pub message: String }
impl AudioError {
    pub fn new(code: &str, message: &str) -> Self { Self { code: code.into(), message: message.into() } }
}
#[cfg(windows)]
impl From<windows::core::Error> for AudioError {
    fn from(error: windows::core::Error) -> Self {
        Self::new(&format!("0x{:08X}", error.code().0 as u32), "Windows Audio недоступен. Устройство могло отключиться; обновите список и повторите действие.")
    }
}
pub type Result<T> = std::result::Result<T, AudioError>;
pub fn volume(value: f32) -> Result<f32> {
    if !value.is_finite() || !(0.0..=100.0).contains(&value) {
        Err(AudioError::new("invalid_volume", "Громкость должна быть от 0 до 100%."))
    } else { Ok(value / 100.0) }
}
pub fn identifier(value: &str) -> Result<()> {
    if value.is_empty() || value.len() > 4096 || value.contains('\0') {
        Err(AudioError::new("invalid_id", "Некорректный идентификатор устройства или сессии."))
    } else { Ok(()) }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn validates_native_arguments() {
        assert_eq!(volume(25.0).unwrap(), 0.25);
        for value in [-1.0, 101.0, f32::NAN, f32::INFINITY] { assert!(volume(value).is_err()); }
        assert!(identifier("").is_err()); assert!(identifier("a\0b").is_err());
    }
}
