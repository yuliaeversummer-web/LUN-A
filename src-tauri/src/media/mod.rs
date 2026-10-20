mod errors;
mod policy;
#[cfg(windows)] mod events;
#[cfg(windows)] mod sessions;
#[cfg(windows)] mod artwork;
#[cfg(windows)] mod metadata;
#[cfg(windows)] mod manager;
use errors::{Result,MediaError};
use serde::{Serialize,Deserialize};
use std::{sync::{Arc,Mutex,mpsc,atomic::{AtomicBool,Ordering}},thread::JoinHandle,time::{Duration,Instant}};
use tauri::{AppHandle,Manager,Emitter};

#[derive(Clone,Serialize,PartialEq)] #[serde(rename_all="camelCase")]
pub struct Candidate {pub id:String,pub source_id:String,pub name:String,pub eligible:bool}
#[derive(Clone,Default,Serialize,PartialEq)]
pub struct Capabilities {pub play:bool,pub pause:bool,pub toggle:bool,pub next:bool,pub previous:bool,pub seek:bool}
#[derive(Clone,Serialize,PartialEq)] #[serde(rename_all="camelCase")]
pub struct Track {pub token:u64,pub title:String,pub artist:String,pub album:String,pub artwork_id:Option<String>}
#[derive(Clone,Serialize,PartialEq)] #[serde(rename_all="camelCase")]
pub struct Timeline {pub position_ms:i64,pub confirmed_position_ms:i64,pub duration_ms:i64,pub min_seek_ms:i64,pub max_seek_ms:i64,pub start_ms:i64,pub estimated:bool}
#[derive(Clone,Serialize,PartialEq)] pub struct Reaction {pub sequence:u64,pub kind:String}
#[derive(Clone,Serialize,PartialEq)] #[serde(rename_all="camelCase")]
pub struct MediaSnapshot {
 pub revision:u64,pub status:String,pub reason:String,pub message:Option<String>,pub warning:Option<String>,
 pub selected_id:Option<String>,pub selected_source:Option<String>,pub selection_mode:String,pub preferred_source:Option<String>,
 pub candidates:Vec<Candidate>,pub track:Option<Track>,pub playback:String,pub capabilities:Capabilities,pub timeline:Option<Timeline>,
 pub reaction:Option<Reaction>,pub command_error:Option<MediaError>,pub persistence_error:Option<String>,
}
impl Default for MediaSnapshot {fn default()->Self {Self {revision:0,status:"loading".into(),reason:"initializing".into(),message:None,warning:None,selected_id:None,selected_source:None,selection_mode:"auto".into(),preferred_source:None,candidates:vec![],track:None,playback:"unavailable".into(),capabilities:Capabilities::default(),timeline:None,reaction:None,command_error:None,persistence_error:None}}}
#[derive(Default,Serialize,Deserialize)] pub struct Preference {pub source_id:Option<String>}
#[derive(Clone)] pub enum Action {Play,Pause,Toggle,Next,Previous,Seek {position:f64,track_token:u64}}
#[derive(Clone)] pub enum Operation {Refresh,Select(Option<String>),Control(String,Action)}
struct Request {operation:Operation,reply:mpsc::Sender<Result<MediaSnapshot>>,deadline:Instant}
#[derive(Default)] struct Cache {snapshot:MediaSnapshot,artwork:Option<(String,String)>}
pub struct MediaService {cache:Arc<Mutex<Cache>>,sender:mpsc::SyncSender<Request>,stop:Arc<AtomicBool>,thread:Mutex<Option<JoinHandle<()>>>}
impl MediaService {
 fn request(&self,operation:Operation)->Result<MediaSnapshot>{
  let(reply,response)=mpsc::channel();self.sender.try_send(Request {operation,reply,deadline:Instant::now()+Duration::from_secs(10)}).map_err(|_|MediaError::new("busy","Медиасервис занят. Повторите действие."))?;
  response.recv_timeout(Duration::from_secs(12)).map_err(|_|MediaError::new("timeout","Медиаплеер не ответил вовремя."))?
 }
 fn shutdown(&self){self.stop.store(true,Ordering::Release);if let Ok(mut thread)=self.thread.lock(){if let Some(thread)=thread.take(){let _=thread.join();}}}
}
impl Drop for MediaService {fn drop(&mut self){self.shutdown();}}
fn publish(app:&AppHandle,cache:&Mutex<Cache>,mut next:MediaSnapshot,artwork:Option<(String,String)>)->MediaSnapshot{
 if let Ok(mut state)=cache.lock(){next.revision=state.snapshot.revision;
  if state.snapshot!=next{next.revision+=1;state.snapshot=next.clone();state.artwork=artwork;let _=app.emit("lunea:media-state",&next);}
  state.snapshot.clone()
 }else{next}
}
pub fn setup(app:&AppHandle)->std::result::Result<(),Box<dyn std::error::Error>>{
 let(sender,receiver)=mpsc::sync_channel(32);let cache=Arc::new(Mutex::new(Cache::default()));let stop=Arc::new(AtomicBool::new(false));
 let handle=app.clone();let state=cache.clone();let stopping=stop.clone();let path=app.path().app_config_dir()?.join("media.json");
 let thread=std::thread::Builder::new().name("lunea-gsmtc".into()).spawn(move||{
  #[cfg(windows)] manager::run(handle,state,stopping,receiver,path);
  #[cfg(not(windows))] {let _=(stopping,receiver,path);publish(&handle,&state,MediaSnapshot {status:"unavailable".into(),reason:"platform".into(),message:Some("GSMTC доступен только в Windows.".into()),..Default::default()},None);}
 })?;
 app.manage(MediaService {cache,sender,stop,thread:Mutex::new(Some(thread))});Ok(())
}
pub fn shutdown(app:&AppHandle){if let Some(service)=app.try_state::<MediaService>(){service.shutdown();}}
async fn perform(app:AppHandle,operation:Operation)->Result<MediaSnapshot>{
 tauri::async_runtime::spawn_blocking(move||app.try_state::<MediaService>().ok_or_else(||MediaError::new("initializing","Медиасервис ещё запускается."))?.request(operation)).await.map_err(|_|MediaError::new("worker","Не удалось выполнить медиакоманду."))?
}
#[tauri::command] pub fn get_media_state(app:AppHandle)->Result<MediaSnapshot>{let state=app.try_state::<MediaService>().ok_or_else(||MediaError::new("initializing","Медиасервис ещё запускается."))?;let result=state.cache.lock().map_err(|_|MediaError::new("state","Медиасостояние недоступно."))?.snapshot.clone();Ok(result)}
#[tauri::command] pub fn get_media_artwork(app:AppHandle,artwork_id:String)->Result<String>{errors::id(&artwork_id)?;let state=app.try_state::<MediaService>().ok_or_else(||MediaError::new("initializing","Медиасервис ещё запускается."))?;let result=state.cache.lock().map_err(|_|MediaError::new("state","Обложка недоступна."))?.artwork.as_ref().filter(|(id,_)|id==&artwork_id).map(|(_,data)|data.clone()).ok_or_else(||MediaError::new("stale_artwork","Обложка этой композиции уже недоступна."));result}
#[tauri::command] pub async fn refresh_media_sessions(app:AppHandle)->Result<MediaSnapshot>{perform(app,Operation::Refresh).await}
#[tauri::command] pub async fn select_media_session(app:AppHandle,session_id:Option<String>)->Result<MediaSnapshot>{if let Some(id)=&session_id{errors::id(id)?;}perform(app,Operation::Select(session_id)).await}
macro_rules! control {($name:ident,$action:expr)=>{#[tauri::command] pub async fn $name(app:AppHandle,session_id:String)->Result<MediaSnapshot>{errors::id(&session_id)?;perform(app,Operation::Control(session_id,$action)).await}};}
control!(media_play,Action::Play);control!(media_pause,Action::Pause);control!(media_toggle_playback,Action::Toggle);control!(media_next,Action::Next);control!(media_previous,Action::Previous);
#[tauri::command] pub async fn media_seek(app:AppHandle,session_id:String,track_token:u64,position_ms:f64)->Result<MediaSnapshot>{errors::id(&session_id)?;if !position_ms.is_finite()||position_ms<0.0{return Err(MediaError::new("invalid_position","Некорректная позиция композиции."));}perform(app,Operation::Control(session_id,Action::Seek {position:position_ms,track_token})).await}
