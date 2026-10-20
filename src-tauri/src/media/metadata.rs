use std::sync::atomic::AtomicBool;
use windows::Media::Control::*;
use super::{Track,Timeline,Capabilities,errors::Result,events::wait,artwork};
pub fn read(session:&GlobalSystemMediaTransportControlsSession,token:u64,stop:&AtomicBool)->Result<(Track,Option<String>)>{
 let operation=session.TryGetMediaPropertiesAsync()?;
 let properties=wait(&operation,stop,||operation.GetResults())?;
 let cover=artwork::read(&properties,stop).ok();
 let track=Track {token,title:properties.Title().map(|s|s.to_string()).unwrap_or_default(),artist:properties.Artist().map(|s|s.to_string()).unwrap_or_default(),album:properties.AlbumTitle().map(|s|s.to_string()).unwrap_or_default(),artwork_id:cover.as_ref().map(|_|format!("cover-{token}"))};
 Ok((track,cover))
}
pub fn playback(session:&GlobalSystemMediaTransportControlsSession)->Result<(String,Capabilities,f64)>{
 let info=session.GetPlaybackInfo()?;let controls=info.Controls()?;let status=info.PlaybackStatus()?;
 let playback=if status==GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing{"playing"}else if status==GlobalSystemMediaTransportControlsSessionPlaybackStatus::Paused{"paused"}else if status==GlobalSystemMediaTransportControlsSessionPlaybackStatus::Changing{"changing"}else{"stopped"};
 Ok((playback.into(),Capabilities {play:controls.IsPlayEnabled().unwrap_or(false),pause:controls.IsPauseEnabled().unwrap_or(false),toggle:controls.IsPlayPauseToggleEnabled().unwrap_or(false),next:controls.IsNextEnabled().unwrap_or(false),previous:controls.IsPreviousEnabled().unwrap_or(false),seek:controls.IsPlaybackPositionEnabled().unwrap_or(false)},info.PlaybackRate().ok().and_then(|r|r.Value().ok()).filter(|r|r.is_finite()&&*r>0.0&&*r<=8.0).unwrap_or(1.0)))
}
fn project(confirmed:i64,duration:i64,elapsed:i64,playing:bool,rate:f64)->(i64,bool){
 let estimated=playing&&(1..=300_000).contains(&elapsed)&&duration>0&&rate.is_finite()&&rate>0.0;
 (if estimated{(confirmed as f64+elapsed as f64*rate).min(duration as f64)as i64}else{confirmed.clamp(0,duration)},estimated)
}
pub fn timeline(session:&GlobalSystemMediaTransportControlsSession,playing:bool,rate:f64)->Result<Timeline>{
 let info=session.GetTimelineProperties()?;let start=info.StartTime()?.Duration/10000;
 let duration=(info.EndTime()?.Duration/10000).saturating_sub(start).max(0);
 let confirmed=(info.Position()?.Duration/10000).saturating_sub(start).clamp(0,duration);
 let min=(info.MinSeekTime()?.Duration/10000).saturating_sub(start).clamp(0,duration);
 let max=(info.MaxSeekTime()?.Duration/10000).saturating_sub(start).clamp(min,duration);
 let now=std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis() as i64+11_644_473_600_000;
 let updated=info.LastUpdatedTime()?.UniversalTime/10000;
 let elapsed=now.saturating_sub(updated);
 let(position,estimated)=project(confirmed,duration,elapsed,playing,rate);
 Ok(Timeline {position_ms:position,confirmed_position_ms:confirmed,duration_ms:duration,min_seek_ms:min,max_seek_ms:max,start_ms:start,estimated})
}
#[cfg(test)]mod tests{use super::*;
 #[test]fn progress_is_bounded_and_does_not_advance_on_pause(){
  assert_eq!(project(1000,5000,2000,true,1.0),(3000,true));
  assert_eq!(project(4000,5000,2000,true,1.0),(5000,true));
  assert_eq!(project(1000,5000,2000,false,1.0),(1000,false));
  assert_eq!(project(1000,5000,-100,true,1.0),(1000,false));
  assert_eq!(project(1000,5000,600000,true,1.0),(1000,false));
 }
}
