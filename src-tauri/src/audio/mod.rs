mod errors;
#[cfg(windows)] mod devices;
#[cfg(windows)] mod endpoint;
#[cfg(windows)] mod sessions;
#[cfg(windows)] mod events;
#[cfg(windows)] mod icons;
#[cfg(windows)] mod worker;

use serde::Serialize;
use std::{sync::{Arc, Mutex, mpsc, atomic::{AtomicBool, Ordering}}, thread::JoinHandle, time::{Duration, Instant}};
use tauri::{AppHandle, Emitter, Manager, State};
use errors::{AudioError, Result};

#[derive(Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Device { pub id: String, pub name: String, pub state: String, pub default_console: bool, pub default_multimedia: bool, pub default_communications: bool }
#[derive(Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Endpoint { pub device_id: String, pub name: String, pub volume: f32, pub muted: bool }
#[derive(Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Session { pub id: String, pub device_id: String, pub device_name: String, pub pid: u32, pub name: String, pub icon: Option<String>, pub volume: f32, pub muted: bool, pub state: String, pub system: bool }
#[derive(Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AudioSnapshot {
    pub revision: u64, pub status: String, pub endpoint: Option<Endpoint>, pub sessions: Vec<Session>,
    pub outputs: Vec<Device>, pub inputs: Vec<Device>, pub message: Option<String>, pub warning: Option<String>,
}
impl Default for AudioSnapshot {
    fn default() -> Self { Self { revision: 0, status: "loading".into(), endpoint: None, sessions: vec![], outputs: vec![], inputs: vec![], message: None, warning: None } }
}
#[derive(Clone)]
pub enum Operation { Refresh, MasterVolume(String, f32), MasterMute(String, bool), SessionVolume(String, f32), SessionMute(String, bool) }
struct Request { operation: Operation, reply: mpsc::Sender<Result<AudioSnapshot>>, deadline: Instant }
pub struct AudioService {
    sender: mpsc::SyncSender<Request>, snapshot: Arc<Mutex<AudioSnapshot>>, stop: Arc<AtomicBool>, thread: Mutex<Option<JoinHandle<()>>>,
}
impl AudioService {
    fn request(&self, operation: Operation) -> Result<AudioSnapshot> {
        let (reply, response) = mpsc::channel();
        self.sender.try_send(Request { operation, reply, deadline: Instant::now() + Duration::from_secs(4) })
            .map_err(|_| AudioError::new("busy", "Аудиомикшер занят. Повторите действие."))?;
        response.recv_timeout(Duration::from_secs(5)).map_err(|_| AudioError::new("timeout", "Windows Audio не ответил вовремя. Повторите действие."))?
    }
    pub fn shutdown(&self) {
        self.stop.store(true, Ordering::Release);
        if let Ok(mut handle) = self.thread.lock() { if let Some(handle) = handle.take() { let _ = handle.join(); } }
    }
}
impl Drop for AudioService { fn drop(&mut self) { self.shutdown(); } }
fn publish(app: &AppHandle, cache: &Mutex<AudioSnapshot>, mut next: AudioSnapshot, force: bool) -> AudioSnapshot {
    if let Ok(mut current) = cache.lock() {
        next.revision = current.revision;
        if force || *current != next {
            next.revision += 1; *current = next.clone();
            let _ = app.emit("lunea:audio-state", &next);
        }
        current.clone()
    } else { next }
}
pub fn setup(app: &AppHandle) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let (sender, receiver) = mpsc::sync_channel(128);
    let snapshot = Arc::new(Mutex::new(AudioSnapshot::default())); let stop = Arc::new(AtomicBool::new(false));
    let handle = app.clone(); let cache = snapshot.clone(); let stopping = stop.clone();
    let thread = std::thread::Builder::new().name("lunea-core-audio".into()).spawn(move || {
        #[cfg(windows)] worker::run(handle, cache, stopping, receiver);
        #[cfg(not(windows))] {
            let _ = (stopping, receiver);
            publish(&handle, &cache, AudioSnapshot { status: "unavailable".into(), message: Some("Аудиомикшер доступен только в Windows.".into()), ..Default::default() }, true);
        }
    })?;
    app.manage(AudioService { sender, snapshot, stop, thread: Mutex::new(Some(thread)) }); Ok(())
}
pub fn shutdown(app: &AppHandle) { if let Some(service) = app.try_state::<AudioService>() { service.shutdown(); } }

async fn perform(app: AppHandle, operation: Operation) -> Result<AudioSnapshot> {
    // Only the bounded channel and owned JSON cross threads, never COM pointers.
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.try_state::<AudioService>().ok_or_else(|| AudioError::new("initializing", "Аудиомикшер ещё запускается."))?;
        state.request(operation)
    }).await.map_err(|_| AudioError::new("worker", "Не удалось выполнить аудиокоманду."))?
}
#[tauri::command]
pub fn get_audio_state(app: AppHandle) -> Result<AudioSnapshot> {
    let service = app.try_state::<AudioService>().ok_or_else(|| AudioError::new("initializing", "Аудиомикшер ещё запускается."))?;
    let result = service.snapshot.lock().map_err(|_| AudioError::new("state", "Аудиосостояние недоступно."))?.clone(); Ok(result)
}
#[tauri::command]
pub fn get_audio_sessions(state: State<'_, AudioService>) -> Result<Vec<Session>> { Ok(state.snapshot.lock().map_err(|_| AudioError::new("state", "Сессии недоступны."))?.sessions.clone()) }
#[tauri::command]
pub fn get_audio_devices(state: State<'_, AudioService>) -> Result<(Vec<Device>, Vec<Device>)> {
    let state = state.snapshot.lock().map_err(|_| AudioError::new("state", "Устройства недоступны."))?; Ok((state.outputs.clone(), state.inputs.clone()))
}
#[tauri::command]
pub async fn refresh_audio(app: AppHandle) -> Result<AudioSnapshot> { perform(app, Operation::Refresh).await }
#[tauri::command]
pub async fn set_master_volume(app: AppHandle, device_id: String, volume: f32) -> Result<AudioSnapshot> {
    errors::identifier(&device_id)?; let volume = errors::volume(volume)?;
    perform(app, Operation::MasterVolume(device_id, volume)).await
}
#[tauri::command]
pub async fn set_master_mute(app: AppHandle, device_id: String, muted: bool) -> Result<AudioSnapshot> {
    errors::identifier(&device_id)?; perform(app, Operation::MasterMute(device_id, muted)).await
}
#[tauri::command]
pub async fn set_session_volume(app: AppHandle, session_id: String, volume: f32) -> Result<AudioSnapshot> {
    errors::identifier(&session_id)?; let volume = errors::volume(volume)?;
    perform(app, Operation::SessionVolume(session_id, volume)).await
}
#[tauri::command]
pub async fn set_session_mute(app: AppHandle, session_id: String, muted: bool) -> Result<AudioSnapshot> {
    errors::identifier(&session_id)?; perform(app, Operation::SessionMute(session_id, muted)).await
}
#[tauri::command]
pub async fn open_sound_settings() -> Result<()> {
    tauri::async_runtime::spawn_blocking(|| {
        #[cfg(windows)] unsafe {
            use windows::{core::w, Win32::{UI::{Shell::ShellExecuteW, WindowsAndMessaging::SW_SHOWNORMAL}}};
            if ShellExecuteW(None, w!("open"), w!("ms-settings:sound"), None, None, SW_SHOWNORMAL).0 as isize <= 32 {
                return Err(AudioError::new("settings", "Не удалось открыть параметры звука Windows."));
            } Ok(())
        }
        #[cfg(not(windows))] { Err(AudioError::new("platform", "Параметры звука доступны в Windows.")) }
    }).await.map_err(|_| AudioError::new("settings", "Не удалось открыть параметры звука."))?
}

#[tauri::command]
pub fn open_mixer(app: AppHandle) -> Result<()> {
    crate::desktop::open_main_window(&app, false).map_err(|_| AudioError::new("main", "Не удалось открыть главный микшер."))?;
    app.emit_to("main", "lunea:navigate", "/mixer").map_err(|_| AudioError::new("main", "Не удалось открыть главный микшер."))
}
