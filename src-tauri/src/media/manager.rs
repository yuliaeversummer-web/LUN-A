use super::{*,sessions::{Manager,Session,SessionEntry,ManagerEvents},events::{Signals,wait},metadata,policy};
use std::{path::PathBuf,fs,sync::mpsc::{Receiver,RecvTimeoutError}};
use windows::{Win32::{System::{WinRT::{RoInitialize,RoUninitialize,RO_INIT_MULTITHREADED},Diagnostics::ToolHelp::*},Foundation::{CloseHandle,HANDLE}},Media::Control::GlobalSystemMediaTransportControlsSessionPlaybackStatus as PlaybackStatus};

struct Apartment;
impl Drop for Apartment{fn drop(&mut self){unsafe{RoUninitialize();}}}
fn desktop_running()->bool{
 unsafe{
  let Ok(handle)=CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS,0)else{return false;};
  struct Snapshot(HANDLE);impl Drop for Snapshot{fn drop(&mut self){unsafe{let _=CloseHandle(self.0);}}}
  let handle=Snapshot(handle);let mut entry=PROCESSENTRY32W {dwSize:std::mem::size_of::<PROCESSENTRY32W>()as u32,..Default::default()};
  if Process32FirstW(handle.0,&mut entry).is_err(){return false;}
  loop{let end=entry.szExeFile.iter().position(|c|*c==0).unwrap_or(entry.szExeFile.len());
   // This executable name was observed on this PC with the desktop GSMTC ID.
   if String::from_utf16_lossy(&entry.szExeFile[..end]).eq_ignore_ascii_case("Яндекс Музыка.exe"){return true;}
   if Process32NextW(handle.0,&mut entry).is_err(){break;}
  }false
 }
}
struct Engine {
 manager:Manager,observer:ManagerEvents,signals:Signals,entries:Vec<SessionEntry>,counter:u64,epoch:u128,
 preference:Preference,path:PathBuf,manual_runtime:Option<String>,snapshot:MediaSnapshot,cover:Option<(String,String)>,
 track_counter:u64,last_metadata:Instant,last_discovery:Instant,timeline_stamp:Option<i64>,timeline_barrier:Option<i64>,timeline_previous:Option<(i64,i64)>,
}
impl Engine {
 fn new(stop:&AtomicBool,path:PathBuf)->Result<Self>{
  let request=Manager::RequestAsync()?;let manager=wait(&request,stop,||request.GetResults())?;let signals=Signals::new();let observer=ManagerEvents::new(&manager,&signals);
  let mut persistence_error=None;
  let preference=match fs::read(&path){Ok(bytes)=>serde_json::from_slice(&bytes).unwrap_or_else(|_|{persistence_error=Some("Настройки музыки повреждены; включён автоматический выбор.".into());Preference::default()}),Err(e)if e.kind()==std::io::ErrorKind::NotFound=>Preference::default(),Err(_)=>{persistence_error=Some("Не удалось прочитать выбор медиасессии.".into());Preference::default()}};
  Ok(Self {manager,observer,signals,entries:vec![],counter:0,epoch:std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_nanos(),preference,path,manual_runtime:None,snapshot:MediaSnapshot {persistence_error,..Default::default()},cover:None,track_counter:0,last_metadata:Instant::now()-Duration::from_secs(10),last_discovery:Instant::now()-Duration::from_secs(10),timeline_stamp:None,timeline_barrier:None,timeline_previous:None})
 }
 fn clear_track(&mut self){if self.timeline_stamp.is_some(){self.timeline_barrier=self.timeline_stamp;}self.cover=None;self.snapshot.track=None;self.snapshot.timeline=None;self.snapshot.capabilities=Capabilities::default();}
 fn unavailable(&mut self,reason:&str,message:&str){self.clear_track();self.timeline_previous=None;self.snapshot.status="unavailable".into();self.snapshot.reason=reason.into();self.snapshot.message=Some(message.into());self.snapshot.playback="unavailable".into();self.snapshot.command_error=None;}
 fn discover(&mut self)->Result<()> {
  let view=self.manager.GetSessions()?;let mut live=Vec::new();
  for index in 0..view.Size()?.min(256){let session=view.GetAt(index)?;
   if let Some(index)=self.entries.iter().position(|entry|sessions::same(&entry.session,&session)){live.push(self.entries.remove(index));}
   else{self.counter+=1;if let Ok(entry)=SessionEntry::new(session,format!("media-{}-{}",self.epoch,self.counter),&self.signals){live.push(entry);}}
  }
  self.entries=live;self.snapshot.candidates=self.entries.iter().map(|entry|entry.dto.clone()).collect();
  let candidates:Vec<String>=self.entries.iter().filter(|e|e.dto.eligible&&self.preference.source_id.as_ref().is_none_or(|source|source==&e.dto.source_id)).map(|e|e.dto.id.clone()).collect();
  let current=self.manager.GetCurrentSession().ok().and_then(|current|self.entries.iter().find(|entry|entry.dto.eligible&&sessions::same(&entry.session,&current)).map(|entry|entry.dto.id.clone()));
  let choice=policy::choose(&candidates,self.preference.source_id.is_some(),self.manual_runtime.as_deref(),current.as_deref());
  if choice!=self.snapshot.selected_id{self.clear_track();Signals::mark(&self.signals.metadata);}
  self.snapshot.selected_source=choice.as_ref().and_then(|id|self.entries.iter().find(|e|&e.dto.id==id)).map(|entry|entry.dto.source_id.clone());
  self.snapshot.selected_id=choice;self.snapshot.preferred_source=self.preference.source_id.clone();self.snapshot.selection_mode=if self.preference.source_id.is_some(){"manual"}else{"auto"}.into();
  if self.snapshot.selected_id.is_none(){
   if candidates.len()>1{self.unavailable("ambiguous","Найдено несколько сессий Яндекс Музыки. Выберите источник в настройках.");}
   else if self.preference.source_id.is_some(){self.unavailable("preferred_missing","Выбранная сессия Яндекс Музыки недоступна. Ожидается её восстановление.");}
   else if desktop_running(){self.unavailable("no_session","Яндекс Музыка запущена, но не опубликовала GSMTC-сессию. Начните воспроизведение в приложении.");}
   else{self.unavailable("not_running","Яндекс Музыка не обнаружена. Запустите установленное приложение и включите композицию.");}
  }
  self.snapshot.warning=if !self.observer.fully_observed()||self.entries.iter().any(|e|!e.fully_observed()){Some("Часть событий Windows недоступна; выполняется дополнительная сверка состояния.".into())}else{None};
  self.last_discovery=Instant::now();Ok(())
 }
 fn selected(&self)->Option<Session>{self.snapshot.selected_id.as_ref().and_then(|id|self.entries.iter().find(|e|&e.dto.id==id&&e.dto.eligible)).map(|e|e.session.clone())}
 fn update(&mut self,app:&AppHandle,cache:&Mutex<Cache>,stop:&AtomicBool,force:bool)->Result<()> {
  if force||Signals::take(&self.signals.sessions)||self.last_discovery.elapsed()>=Duration::from_secs(2){self.discover()?;}
  let Some(session)=self.selected()else{return Ok(());};
  let dirty=Signals::take(&self.signals.metadata);
  if dirty||self.snapshot.track.is_none()||self.last_metadata.elapsed()>=Duration::from_secs(5){
   // Clear the previous cover before any potentially slow WinRT stream access.
   let previous=self.snapshot.track.clone();let previous_cover=self.cover.clone();let previous_barrier=self.timeline_barrier;let previous_timeline=self.snapshot.timeline.clone();
   if dirty||previous.is_none(){self.clear_track();self.snapshot.status="loading".into();self.snapshot.message=Some("Чтение композиции Windows…".into());publish(app,cache,self.snapshot.clone(),None);}
   self.track_counter+=1;
   let (mut track,cover)=metadata::read(&session,self.track_counter,stop)?;
   let same=previous.as_ref().is_some_and(|old|old.title==track.title&&old.artist==track.artist&&old.album==track.album);
   if same{self.timeline_barrier=previous_barrier;}
   else if previous.is_some(){self.timeline_previous=previous_timeline.map(|t|(t.duration_ms,t.position_ms));}
   if same&&!dirty {track.token=previous.as_ref().map(|t|t.token).unwrap_or(track.token);track.artwork_id=previous.as_ref().and_then(|t|t.artwork_id.clone());self.cover=previous_cover;}
   else{self.cover=track.artwork_id.clone().zip(cover);}
   self.snapshot.track=Some(track);self.last_metadata=Instant::now();
  }
  let (playback,capabilities,rate)=metadata::playback(&session)?;
  if session.GetPlaybackInfo()?.PlaybackStatus()?==PlaybackStatus::Closed{self.unavailable("session_closed","Сессия Яндекс Музыки завершилась. Ожидается её восстановление.");return Ok(());}
  // Yandex can publish the new title before replacing the previous track's
  // timeline. Hide that old position until Windows supplies a new timestamp.
  let stamp=session.GetTimelineProperties().ok().and_then(|t|t.LastUpdatedTime().ok()).map(|t|t.UniversalTime);
  let timeline=metadata::timeline(&session,playback=="playing",rate).ok();
  let previous_position=timeline.as_ref().is_some_and(|t|self.timeline_previous.is_some_and(|(duration,position)|t.duration_ms==duration&&t.confirmed_position_ms>=position.saturating_sub(1000).max(0)));
  self.snapshot.timeline=if (self.timeline_barrier.is_some()&&stamp==self.timeline_barrier)||previous_position{None}else{self.timeline_barrier=None;self.timeline_previous=None;timeline};
  self.timeline_stamp=stamp;
  if self.snapshot.command_error.is_some(){if let Some(timeline)=self.snapshot.timeline.as_mut(){timeline.position_ms=timeline.confirmed_position_ms;timeline.estimated=false;}}
  if self.snapshot.timeline.is_none(){self.snapshot.warning=Some("Windows не предоставил позицию композиции; прогресс недоступен.".into());}
  self.snapshot.status=if playback=="paused"{"paused"}else{"ready"}.into();self.snapshot.playback=playback;
  self.snapshot.capabilities=capabilities;self.snapshot.reason.clear();self.snapshot.message=None;
  let _=Signals::take(&self.signals.playback);let _=Signals::take(&self.signals.timeline);Ok(())
 }
 fn reaction(&mut self,kind:&str){let sequence=self.snapshot.reaction.as_ref().map(|r|r.sequence+1).unwrap_or(1);self.snapshot.reaction=Some(Reaction {sequence,kind:kind.into()});}
 fn save(&mut self){let result=(||->std::io::Result<()>{if let Some(parent)=self.path.parent(){fs::create_dir_all(parent)?;}let bytes=serde_json::to_vec(&self.preference)?;let temporary=self.path.with_extension("tmp");fs::write(&temporary,bytes)?;fs::rename(temporary,&self.path)})();self.snapshot.persistence_error=result.err().map(|_|"Не удалось сохранить выбор медиасессии.".into());}
 fn execute(&mut self,operation:&Operation,stop:&AtomicBool)->Result<()> {
  self.discover()?;self.snapshot.command_error=None;
  match operation{
   Operation::Refresh=>{Signals::mark(&self.signals.metadata);},
   Operation::Select(id)=>{
    if let Some(id)=id{let entry=self.entries.iter().find(|entry|&entry.dto.id==id&&entry.dto.eligible).ok_or_else(||MediaError::new("source_forbidden","Можно выбрать только обнаруженную сессию десктопной Яндекс Музыки."))?;self.preference.source_id=Some(entry.dto.source_id.clone());self.manual_runtime=Some(id.clone());}
    else{self.preference.source_id=None;self.manual_runtime=None;}
    self.save();self.discover()?;Signals::mark(&self.signals.metadata);
   },
   Operation::Control(expected,action)=>{
    if self.snapshot.selected_id.as_ref()!=Some(expected){return Err(MediaError::new("session_changed","Целевая сессия исчезла или изменилась. Повторите действие после обновления."));}
    let session=self.selected().ok_or_else(||MediaError::new("unavailable","Нет доступной сессии Яндекс Музыки."))?;
    // Fresh discovery above verifies identity, not the OS-wide current player.
    if !policy::eligible(&session.SourceAppUserModelId()?.to_string()){return Err(MediaError::new("source_forbidden","Управление этим источником запрещено."));}
    let (playback,capabilities,_)=metadata::playback(&session)?;
    let unsupported=||MediaError::new("unsupported","Яндекс Музыка не поддерживает это действие в текущем состоянии.");
    let op=match action{
     Action::Play if capabilities.play=>session.TryPlayAsync()?,Action::Pause if capabilities.pause=>session.TryPauseAsync()?,
     Action::Toggle if capabilities.toggle=>session.TryTogglePlayPauseAsync()?,
     Action::Toggle if playback=="playing"&&capabilities.pause=>session.TryPauseAsync()?,
     Action::Toggle if playback!="playing"&&capabilities.play=>session.TryPlayAsync()?,
     Action::Next if capabilities.next=>session.TrySkipNextAsync()?,Action::Previous if capabilities.previous=>session.TrySkipPreviousAsync()?,
     Action::Seek {position,track_token} if capabilities.seek=>{
      if self.snapshot.track.as_ref().map(|track|track.token)!=Some(*track_token)||self.signals.metadata.load(Ordering::Acquire){return Err(MediaError::new("track_changed","Композиция изменилась; повторите перемотку."));}
      let operation=session.TryGetMediaPropertiesAsync()?;let properties=wait(&operation,stop,||operation.GetResults())?;
      if !self.snapshot.track.as_ref().is_some_and(|track|track.title==properties.Title().map(|s|s.to_string()).unwrap_or_default()&&track.artist==properties.Artist().map(|s|s.to_string()).unwrap_or_default()&&track.album==properties.AlbumTitle().map(|s|s.to_string()).unwrap_or_default()){
       Signals::mark(&self.signals.metadata);return Err(MediaError::new("track_changed","Композиция изменилась; повторите перемотку."));
      }
      let timeline=metadata::timeline(&session,false,1.0)?;let relative=errors::position(*position,timeline.duration_ms,timeline.min_seek_ms,timeline.max_seek_ms)?;
      let ticks=relative.checked_add(timeline.start_ms).and_then(|value|value.checked_mul(10000)).ok_or_else(||MediaError::new("invalid_position","Позиция выходит за диапазон Windows."))?;
      session.TryChangePlaybackPositionAsync(ticks)?
     },_=>return Err(unsupported()),
    };
    if !wait(&op,stop,||op.GetResults())?{return Err(MediaError::new("rejected","Яндекс Музыка отклонила команду. Состояние не подменено."));}
    if matches!(action,Action::Next|Action::Previous){self.reaction("next");Signals::mark(&self.signals.metadata);}
    // No optimistic playing/seek/track assignment: next read is authoritative.
   }
  }Ok(())
 }
}
impl Drop for Engine{fn drop(&mut self){self.entries.clear();}}
pub(super) fn run(app:AppHandle,cache:Arc<Mutex<Cache>>,stop:Arc<AtomicBool>,receiver:Receiver<Request>,path:PathBuf){
 #[cfg(debug_assertions)] eprintln!("LUNÉA GSMTC: worker started");
 struct Lifetime;impl Drop for Lifetime{fn drop(&mut self){#[cfg(debug_assertions)] eprintln!("LUNÉA GSMTC: worker stopped");}}
 let _lifetime=Lifetime;
 if let Err(error)=unsafe{RoInitialize(RO_INIT_MULTITHREADED)}{publish(&app,&cache,MediaSnapshot {status:"error".into(),reason:"winrt".into(),message:Some(MediaError::from(error).message),..Default::default()},None);return;}
 let _apartment=Apartment;let mut engine:Option<Engine>=None;let mut retry=Instant::now()-Duration::from_secs(5);let mut read_failures=0u8;
 while !stop.load(Ordering::Acquire){
  let request=match receiver.recv_timeout(Duration::from_millis(250)){Ok(request)=>Some(request),Err(RecvTimeoutError::Timeout)=>None,Err(RecvTimeoutError::Disconnected)=>break};
  if stop.load(Ordering::Acquire){break;}
  if engine.is_none()&&retry.elapsed()>=Duration::from_secs(3){match Engine::new(&stop,path.clone()){Ok(next)=>engine=Some(next),Err(error)=>{publish(&app,&cache,MediaSnapshot {status:"error".into(),reason:"gsmtc".into(),message:Some(error.message),..Default::default()},None);retry=Instant::now();}}}
  let Some(active)=engine.as_mut()else{if let Some(request)=request{let _=request.reply.send(Err(MediaError::new("unavailable","Windows GSMTC пока недоступен.")));}continue;};
  let result=request.as_ref().map(|request|if Instant::now()>request.deadline{Err(MediaError::new("expired","Медиакоманда устарела."))}else{active.execute(&request.operation,&stop)});
  if let Some(Err(error))=&result{active.snapshot.command_error=Some(error.clone());active.reaction("error");}
  if let Err(error)=active.update(&app,&cache,&stop,matches!(request.as_ref().map(|r|&r.operation),Some(Operation::Refresh))){
   read_failures=read_failures.saturating_add(1);
   active.clear_track();active.snapshot.status="error".into();active.snapshot.reason="read_error".into();active.snapshot.message=Some(error.message);
  }else{read_failures=0;}
  let snapshot=publish(&app,&cache,active.snapshot.clone(),active.cover.clone());
  if let Some(request)=request{let _=request.reply.send(result.unwrap_or(Ok(())).map(|_|snapshot));}
  // Recreate a closed/disconnected WinRT manager after repeated read failures.
  // Dropping the old engine removes its observer tokens before subscribing again.
  if read_failures>=3{engine=None;retry=Instant::now();read_failures=0;}
 }
 drop(engine);
}
