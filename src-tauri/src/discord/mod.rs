mod errors;
mod hotkeys;
#[cfg(windows)]mod detection;
#[cfg(windows)]mod input;
mod sessions;
use errors::{DiscordError,Result};
use serde::{Deserialize,Serialize};
use std::{sync::{Arc,Mutex,mpsc,atomic::{AtomicBool,Ordering}},thread::JoinHandle,time::{Duration,Instant},path::PathBuf};
use tauri::{AppHandle,Manager,Emitter};
#[derive(Clone,Serialize,Deserialize,PartialEq)]#[serde(rename_all="camelCase",default,deny_unknown_fields)]
pub struct Preferences {pub client:String,pub enabled:bool,pub confirmed:bool,pub mute:String,pub deafen:String}
impl Default for Preferences{fn default()->Self{Self{client:"stable".into(),enabled:false,confirmed:false,mute:String::new(),deafen:String::new()}}}
#[derive(Clone,Serialize,PartialEq)]#[serde(rename_all="camelCase")]
pub struct Client{pub variant:String,pub installed:bool,pub running:bool,pub pids:Vec<u32>,pub privilege_compatible:bool}
#[derive(Clone,Serialize,PartialEq)]#[serde(rename_all="camelCase")]
pub struct Attempt{pub sequence:u64,pub action:String,pub status:String,pub message:String,pub timestamp_ms:u64,pub code:Option<String>}
#[derive(Clone,Serialize,PartialEq)]#[serde(rename_all="camelCase")]
pub struct Snapshot{pub revision:u64,pub configuration_revision:u64,pub preferences:Preferences,pub clients:Vec<Client>,pub status:String,pub reason:String,pub message:String,pub can_send:bool,pub mode:String,pub mute:Option<bool>,pub deafen:Option<bool>,pub audio_sessions:Vec<crate::audio::Session>,pub last_attempt:Option<Attempt>,pub persistence_error:Option<String>}
impl Default for Snapshot{fn default()->Self{Self{revision:0,configuration_revision:1,preferences:Preferences::default(),clients:vec![],status:"loading".into(),reason:"initializing".into(),message:"Обнаружение Discord…".into(),can_send:false,mode:"hotkeys".into(),mute:None,deafen:None,audio_sessions:vec![],last_attempt:None,persistence_error:None}}}
enum Operation{Refresh,Configure(Preferences),Action{kind:&'static str,client:String,configuration_revision:u64}}
struct Request{operation:Operation,reply:mpsc::Sender<Result<Snapshot>>,deadline:Instant}
pub struct DiscordService{cache:Arc<Mutex<Snapshot>>,sender:mpsc::SyncSender<Request>,stop:Arc<AtomicBool>,thread:Mutex<Option<JoinHandle<()>>>}
impl DiscordService{
 fn shutdown(&self){self.stop.store(true,Ordering::Release);if let Ok(mut handle)=self.thread.lock(){if let Some(handle)=handle.take(){let _=handle.join();}}}
 fn request(&self,operation:Operation)->Result<Snapshot>{let(reply,response)=mpsc::channel();self.sender.try_send(Request{operation,reply,deadline:Instant::now()+Duration::from_secs(3)}).map_err(|_|DiscordError::new("busy","Discord-команда уже выполняется."))?;response.recv_timeout(Duration::from_secs(5)).map_err(|_|DiscordError::new("timeout","Сервис Discord не ответил вовремя."))?}
}
impl Drop for DiscordService{fn drop(&mut self){self.shutdown();}}
fn publish(app:&AppHandle,cache:&Mutex<Snapshot>,mut next:Snapshot)->Snapshot{if let Ok(mut current)=cache.lock(){next.revision=current.revision;if next!=*current{next.revision+=1;*current=next.clone();let _=app.emit("lunea:discord-state",&next);}current.clone()}else{next}}
fn inspect(app:&AppHandle,s:&mut Snapshot){
 #[cfg(windows)]match detection::scan(){Ok(clients)=>s.clients=clients,Err(error)=>{s.can_send=false;s.status="error".into();s.reason=error.code;s.message=error.message;return;}}
 s.audio_sessions=crate::audio::get_audio_state(app.clone()).ok().map(|audio|sessions::matching(&audio.sessions,&s.clients,&s.preferences.client)).unwrap_or_default();for session in &mut s.audio_sessions{session.icon=None;}
 let selected=s.clients.iter().find(|c|c.variant==s.preferences.client);
 let(reason,message)=if !selected.is_some_and(|c|c.running){("unavailable",if selected.is_some_and(|c|c.installed){"Discord установлен, но не запущен."}else{"Выбранный клиент Discord не обнаружен в стандартных каталогах."})}
 else if s.clients.iter().filter(|c|c.running).count()>1{("multiple_clients","Запущено несколько вариантов Discord. Глобальный ввод нельзя направить только одному: закройте остальные клиенты.")}
 else if !selected.is_some_and(|c|c.privilege_compatible){("uipi","Уровень доступа Discord выше LUNÉA или не удалось его проверить. Команды недоступны; повышение прав автоматически не выполняется.")}
 else if !s.preferences.enabled||!s.preferences.confirmed{("setup","Назначьте и подтвердите горячие клавиши в настройках LUNÉA.")}
 else if hotkeys::validate(&s.preferences,&crate::desktop::overlay_shortcut(app)).is_err(){("conflict","Сохранённые сочетания невалидны или конфликтуют с оверлеем. Исправьте настройки.")}
 else{("ready","Discord запущен. Сочетания доступны; состояния микрофона и Deafen неизвестны.")};
 s.reason=reason.into();s.message=message.into();s.can_send=reason=="ready";s.status=if reason=="unavailable"{"unavailable"}else if reason=="ready"{"ready"}else{"setup"}.into();
}
fn save(path:&PathBuf,p:&Preferences)->Result<()>{let result=(||->std::io::Result<()>{if let Some(parent)=path.parent(){std::fs::create_dir_all(parent)?;}let tmp=path.with_extension("tmp");std::fs::write(&tmp,serde_json::to_vec(p)?)?;std::fs::rename(tmp,path)})();result.map_err(|_|DiscordError::new("persistence","Не удалось сохранить настройки Discord. Прежняя конфигурация сохранена."))}
fn execute(app:&AppHandle,s:&mut Snapshot,operation:Operation,path:&PathBuf,last:&mut Option<Instant>)->Result<()>{
 inspect(app,s);
 match operation{
 Operation::Refresh=>Ok(()),
 Operation::Configure(mut p)=>{hotkeys::validate(&p,&crate::desktop::overlay_shortcut(app))?;if !p.mute.is_empty(){p.mute=hotkeys::parse(&p.mute)?.canonical;}if !p.deafen.is_empty(){p.deafen=hotkeys::parse(&p.deafen)?.canonical;}save(path,&p)?;s.preferences=p;s.configuration_revision+=1;s.persistence_error=None;inspect(app,s);Ok(())},
 Operation::Action{kind,client,configuration_revision}=>{
  let result=(||->Result<()>{if last.is_some_and(|at|at.elapsed()<Duration::from_millis(1200)){return Err(DiscordError::new("cooldown","Повторное нажатие пропущено. Предыдущее действие могло быть отправлено; проверьте Discord."));}*last=Some(Instant::now());
   if configuration_revision!=s.configuration_revision||client!=s.preferences.client{return Err(DiscordError::new("configuration_changed","Настройки Discord изменились. Повторите действие после обновления."));}
   hotkeys::validate(&s.preferences,&crate::desktop::overlay_shortcut(app))?;
   if !s.can_send{return Err(DiscordError::new(&s.reason,&s.message));}
   let chord=hotkeys::parse(if kind=="mute"{&s.preferences.mute}else{&s.preferences.deafen})?;
   #[cfg(windows)]input::send(&chord)?;
   #[cfg(not(windows))]{let _=chord;return Err(DiscordError::new("platform","Отправка сочетаний доступна только в Windows."));}
   Ok(())
  })();
  let sequence=s.last_attempt.as_ref().map(|a|a.sequence+1).unwrap_or(1);let sent=result.is_ok();s.last_attempt=Some(Attempt{sequence,action:kind.into(),status:if sent{"sent"}else{"not_sent"}.into(),message:result.as_ref().err().map(|e|e.message.clone()).unwrap_or_else(||"Сочетание отправлено в Windows. Переключение Discord не подтверждено.".into()),timestamp_ms:std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis()as u64,code:result.as_ref().err().map(|e|e.code.clone())});result
 }
 }
}
pub fn setup(app:&AppHandle)->std::result::Result<(),Box<dyn std::error::Error>>{
 let cache=Arc::new(Mutex::new(Snapshot::default()));let stop=Arc::new(AtomicBool::new(false));let(sender,receiver)=mpsc::sync_channel::<Request>(8);let path=app.path().app_config_dir()?.join("discord.json");let app_handle=app.clone();let state=cache.clone();let stopping=stop.clone();
 let thread=std::thread::Builder::new().name("lunea-discord".into()).spawn(move||{
  #[cfg(debug_assertions)]eprintln!("LUNÉA Discord: worker started");
  let mut snapshot=Snapshot::default();match std::fs::read(&path){Ok(bytes)=>match serde_json::from_slice::<Preferences>(&bytes){Ok(p)=>{if hotkeys::validate(&p,&crate::desktop::overlay_shortcut(&app_handle)).is_ok(){snapshot.preferences=p;}else{snapshot.persistence_error=Some("Сохранённые настройки Discord не прошли проверку; управление отключено.".into());}},Err(_)=>snapshot.persistence_error=Some("Настройки Discord повреждены; управление отключено.".into())},Err(e)if e.kind()==std::io::ErrorKind::NotFound=>{},Err(_)=>snapshot.persistence_error=Some("Не удалось прочитать настройки Discord.".into())}
  let mut last=None;let mut polled=Instant::now()-Duration::from_secs(2);
  while !stopping.load(Ordering::Acquire){let request=match receiver.recv_timeout(Duration::from_millis(250)){Ok(r)=>Some(r),Err(mpsc::RecvTimeoutError::Timeout)=>None,Err(_)=>break};if stopping.load(Ordering::Acquire){break;}
   let result=if let Some(request)=request{let result=if Instant::now()>request.deadline{Err(DiscordError::new("expired","Команда Discord устарела и не отправлена."))}else{execute(&app_handle,&mut snapshot,request.operation,&path,&mut last)};Some((request.reply,result))}else{None};
   if polled.elapsed()>=Duration::from_secs(1){inspect(&app_handle,&mut snapshot);polled=Instant::now();}let current=publish(&app_handle,&state,snapshot.clone());if let Some((reply,result))=result{let _=reply.send(result.map(|_|current));}
  }
  #[cfg(debug_assertions)]eprintln!("LUNÉA Discord: worker stopped");
 })?;app.manage(DiscordService{cache,sender,stop,thread:Mutex::new(Some(thread))});Ok(())
}
pub fn shutdown(app:&AppHandle){if let Some(service)=app.try_state::<DiscordService>(){service.shutdown();}}
pub fn validate_overlay_shortcut(app:&AppHandle,shortcut:&str)->std::result::Result<(),String>{if let Some(service)=app.try_state::<DiscordService>(){let s=service.cache.lock().map_err(|_|"Настройки Discord недоступны")?;if s.preferences.enabled&&[&s.preferences.mute,&s.preferences.deafen].iter().any(|key|hotkeys::same(key,shortcut)){return Err("Сочетание занято действием Discord. Сначала измените настройки Discord в LUNÉA.".into());}}Ok(())}
async fn perform(app:AppHandle,operation:Operation)->Result<Snapshot>{tauri::async_runtime::spawn_blocking(move||app.try_state::<DiscordService>().ok_or_else(||DiscordError::new("initializing","Сервис Discord запускается."))?.request(operation)).await.map_err(|_|DiscordError::new("worker","Не удалось выполнить команду Discord."))?}
#[tauri::command]pub fn get_discord_state(app:AppHandle)->Result<Snapshot>{let state=app.try_state::<DiscordService>().ok_or_else(||DiscordError::new("initializing","Сервис Discord запускается."))?;let snapshot=state.cache.lock().map_err(|_|DiscordError::new("state","Состояние Discord недоступно."))?.clone();Ok(snapshot)}
#[tauri::command]pub fn get_discord_settings(app:AppHandle)->Result<Preferences>{Ok(get_discord_state(app)?.preferences)}
#[tauri::command]pub async fn refresh_discord(app:AppHandle)->Result<Snapshot>{perform(app,Operation::Refresh).await}
#[tauri::command]pub async fn configure_discord(app:AppHandle,preferences:Preferences)->Result<Snapshot>{perform(app,Operation::Configure(preferences)).await}
#[tauri::command]pub async fn discord_toggle_mute(app:AppHandle,client:String,configuration_revision:u64)->Result<Snapshot>{perform(app,Operation::Action{kind:"mute",client,configuration_revision}).await}
#[tauri::command]pub async fn discord_toggle_deafen(app:AppHandle,client:String,configuration_revision:u64)->Result<Snapshot>{perform(app,Operation::Action{kind:"deafen",client,configuration_revision}).await}
