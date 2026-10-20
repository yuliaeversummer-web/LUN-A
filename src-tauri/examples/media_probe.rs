//! Read-only GSMTC discovery for QA. Never started by LUNÉA or bundled.
#[cfg(windows)]
fn main() -> windows::core::Result<()> {
 use windows::{Media::Control::GlobalSystemMediaTransportControlsSessionManager as Manager, Win32::System::WinRT::{RoInitialize,RoUninitialize,RO_INIT_MULTITHREADED}};
 use serde_json::json;
 unsafe { RoInitialize(RO_INIT_MULTITHREADED)?; }
 let result=(|| -> windows::core::Result<()> {
  let manager=Manager::RequestAsync()?.join()?; let sessions=manager.GetSessions()?;
  let mut rows=Vec::new();
  for i in 0..sessions.Size()? {
   let session=sessions.GetAt(i)?; let playback=session.GetPlaybackInfo()?; let controls=playback.Controls()?;
   let properties=session.TryGetMediaPropertiesAsync()?.join()?; let timeline=session.GetTimelineProperties()?;
   let thumbnail=properties.Thumbnail().ok().and_then(|r|r.OpenReadAsync().ok()?.join().ok()).map(|s|json!({"size":s.Size().ok(),"type":s.ContentType().ok().map(|s|s.to_string())}));
   rows.push(json!({"source":session.SourceAppUserModelId()?.to_string(),"title":properties.Title()?.to_string(),"artist":properties.Artist()?.to_string(),"album":properties.AlbumTitle()?.to_string(),"playback":playback.PlaybackStatus()?.0,
    "position":timeline.Position()?.Duration,"start":timeline.StartTime()?.Duration,"end":timeline.EndTime()?.Duration,"lastUpdated":timeline.LastUpdatedTime()?.UniversalTime,
    "controls":{"play":controls.IsPlayEnabled()?,"pause":controls.IsPauseEnabled()?,"toggle":controls.IsPlayPauseToggleEnabled()?,"next":controls.IsNextEnabled()?,"previous":controls.IsPreviousEnabled()?,"seek":controls.IsPlaybackPositionEnabled()?},"thumbnail":thumbnail}));
  }
  println!("{}",json!({"current":manager.GetCurrentSession().ok().and_then(|s|s.SourceAppUserModelId().ok()).map(|s|s.to_string()),"sessions":rows})); Ok(())
 })();
 unsafe {RoUninitialize();} result
}
#[cfg(not(windows))] fn main() {}
