use super::{*, devices, endpoint::EndpointHandle, sessions::Sessions, events::{Dirty, DeviceEvents}, errors::AudioError};
use std::sync::mpsc::{Receiver, RecvTimeoutError};
use windows::{core::PCWSTR, Win32::{Media::Audio::*, System::Com::{CoInitializeEx, CoUninitialize, CoCreateInstance, COINIT_MULTITHREADED, CLSCTX_ALL}}};

struct Apartment;
impl Apartment { fn new() -> windows::core::Result<Self> { unsafe { CoInitializeEx(None, COINIT_MULTITHREADED).ok()?; } Ok(Self) } }
impl Drop for Apartment { fn drop(&mut self) { unsafe { CoUninitialize(); } } }
struct Engine {
    enumerator: IMMDeviceEnumerator, notification: Option<IMMNotificationClient>, dirty: Dirty,
    endpoint: Option<EndpointHandle>, sessions: std::collections::HashMap<String, Sessions>,
}
impl Engine {
    fn new(dirty: Dirty) -> windows::core::Result<Self> {
        unsafe {
            let enumerator: IMMDeviceEnumerator = CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)?;
            let notification: IMMNotificationClient = DeviceEvents(dirty.clone()).into();
            let registered = enumerator.RegisterEndpointNotificationCallback(&notification).is_ok();
            Ok(Self { enumerator, notification: registered.then_some(notification), dirty, endpoint: None, sessions: std::collections::HashMap::new() })
        }
    }
    fn read(&mut self) -> windows::core::Result<AudioSnapshot> {
        let outputs = devices::list(&self.enumerator, eRender)?;
        let inputs = devices::list(&self.enumerator, eCapture)?;
        self.sessions.retain(|id, _| outputs.iter().any(|device| device.id == *id && device.state == "active"));
        let active_id = devices::default_id(&self.enumerator, eRender, eConsole);
        if self.endpoint.as_ref().map(|endpoint| &endpoint.id) != active_id.as_ref() {
            self.endpoint = None;
        }
        let mut snapshot = AudioSnapshot { outputs, inputs, ..Default::default() };
        let Some(id) = active_id else {
            snapshot.status = "unavailable".into(); snapshot.message = Some("Нет активного устройства вывода. Подключите наушники или проверьте параметры Windows.".into());
            return Ok(snapshot);
        };
        unsafe {
            if self.endpoint.is_none() {
                let wide: Vec<u16> = id.encode_utf16().chain(Some(0)).collect();
                self.endpoint = Some(EndpointHandle::new(self.enumerator.GetDevice(PCWSTR(wide.as_ptr()))?, self.dirty.clone())?);
            }
            let endpoint = self.endpoint.as_ref().ok_or_else(windows::core::Error::empty)?;
            snapshot.endpoint = Some(endpoint.read()?);
            // Applications may be explicitly routed to a different output in
            // Windows. Enumerate each active render endpoint once, not just the
            // default. A session instance ID includes its endpoint identity.
            for device in snapshot.outputs.iter().filter(|device| device.state == "active") {
                if !self.sessions.contains_key(&device.id) {
                    let wide: Vec<u16> = device.id.encode_utf16().chain(Some(0)).collect();
                    if let Ok(handle) = self.enumerator.GetDevice(PCWSTR(wide.as_ptr())).and_then(|handle| Sessions::new(&handle, self.dirty.clone())) {
                        self.sessions.insert(device.id.clone(), handle);
                    }
                }
                let rows = self.sessions.get_mut(&device.id).map(|sessions| sessions.read());
                match rows {
                    Some(Ok(rows)) => snapshot.sessions.extend(rows),
                    _ => { snapshot.warning = Some("Часть сессий Windows недоступна; повторная проверка выполняется автоматически.".into()); self.sessions.remove(&device.id); }
                }
            }
            snapshot.sessions.sort_by(|a,b| a.state.cmp(&b.state).then_with(|| a.name.cmp(&b.name)).then_with(|| a.id.cmp(&b.id)));
            if self.notification.is_none() || !endpoint.observing() || self.sessions.values().any(|sessions| !sessions.observing()) {
                snapshot.warning = Some("Часть уведомлений Windows недоступна; используется дополнительная проверка раз в 2 секунды.".into());
            }
        }
        snapshot.status = "ready".into(); Ok(snapshot)
    }
    fn execute(&mut self, operation: &Operation) -> Result<()> {
        unsafe {
            match operation {
                Operation::Refresh => {},
                Operation::MasterVolume(id, _) | Operation::MasterMute(id, _) => {
                    if devices::default_id(&self.enumerator, eRender, eConsole).as_ref() != Some(id) {
                        return Err(AudioError::new("device_changed", "Устройство вывода изменилось. Дождитесь обновления и повторите действие."));
                    }
                    let endpoint = self.endpoint.as_ref().filter(|endpoint| &endpoint.id == id).ok_or_else(|| AudioError::new("unavailable", "Устройство вывода недоступно."))?;
                    match operation {
                        Operation::MasterVolume(_, value) => endpoint.volume.SetMasterVolumeLevelScalar(*value, std::ptr::null())?,
                        Operation::MasterMute(_, muted) => endpoint.volume.SetMute(*muted, std::ptr::null())?,
                        _ => {},
                    }
                }
                Operation::SessionVolume(id, _) | Operation::SessionMute(id, _) => {
                    let (device_id, session) = self.sessions.iter().find_map(|(device_id, sessions)| sessions.handles.get(id).map(|session| (device_id, session)))
                        .ok_or_else(|| AudioError::new("session_closed", "Эта аудиосессия уже завершена. Обновите микшер."))?;
                    let wide: Vec<u16> = device_id.encode_utf16().chain(Some(0)).collect();
                    if self.enumerator.GetDevice(PCWSTR(wide.as_ptr()))?.GetState()? != DEVICE_STATE_ACTIVE {
                        return Err(AudioError::new("device_changed", "Устройство этой сессии отключено. Дождитесь обновления микшера."));
                    }
                    if session.control.GetState()? == AudioSessionStateExpired { return Err(AudioError::new("session_closed", "Аудиосессия завершена.")); }
                    match operation {
                        Operation::SessionVolume(_, volume) => session.volume.SetMasterVolume(*volume, std::ptr::null())?,
                        Operation::SessionMute(_, muted) => session.volume.SetMute(*muted, std::ptr::null())?,
                        _ => {},
                    }
                }
            }
        }
        Ok(())
    }
}
impl Drop for Engine {
    fn drop(&mut self) {
        self.sessions.clear(); self.endpoint = None;
        if let Some(notification) = &self.notification { unsafe { let _ = self.enumerator.UnregisterEndpointNotificationCallback(notification); } }
    }
}
fn failed(error: AudioError) -> AudioSnapshot {
    AudioSnapshot { status: "error".into(), message: Some(format!("{} ({})", error.message, error.code)), ..Default::default() }
}
pub(super) fn run(app: AppHandle, cache: Arc<Mutex<AudioSnapshot>>, stop: Arc<AtomicBool>, receiver: Receiver<Request>) {
    #[cfg(debug_assertions)] eprintln!("LUNÉA Windows Audio: worker started");
    struct Lifetime;
    impl Drop for Lifetime { fn drop(&mut self) { #[cfg(debug_assertions)] eprintln!("LUNÉA Windows Audio: worker stopped"); } }
    let _lifetime = Lifetime;
    // Guard is declared before interfaces so all registrations and COM objects
    // are dropped before CoUninitialize on every return path.
    let _apartment = match Apartment::new() {
        Ok(guard) => guard,
        Err(error) => { publish(&app, &cache, failed(error.into()), true); return; }
    };
    let dirty = Dirty(Arc::new(AtomicBool::new(true)));
    let mut engine: Option<Engine> = None; let mut last_refresh = Instant::now() - Duration::from_secs(3);
    while !stop.load(Ordering::Acquire) {
        let request = match receiver.recv_timeout(Duration::from_millis(40)) {
            Ok(request) => Some(request), Err(RecvTimeoutError::Timeout) => None, Err(RecvTimeoutError::Disconnected) => break,
        };
        if stop.load(Ordering::Acquire) { break; }
        if engine.is_none() && last_refresh.elapsed() >= Duration::from_secs(2) {
            match Engine::new(dirty.clone()) {
                Ok(next) => engine = Some(next),
                Err(error) => { publish(&app, &cache, failed(error.into()), false); last_refresh = Instant::now(); }
            }
        }
        let result = request.as_ref().map(|request| {
            if request.deadline < Instant::now() { return Err(AudioError::new("expired", "Аудиокоманда устарела. Повторите действие.")); }
            engine.as_mut().ok_or_else(|| AudioError::new("unavailable", "Служба Windows Audio недоступна."))?.execute(&request.operation)
        });
        // Callbacks coalesce into one atomic flag; 2s reconciliation also covers
        // session-creation races, failed registrations and AudioSrv restarts.
        if request.is_some() || dirty.0.swap(false, Ordering::AcqRel) || last_refresh.elapsed() >= Duration::from_secs(2) {
            if let Some(active) = engine.as_mut() {
                let next = match active.read() {
                    Ok(snapshot) => snapshot,
                    Err(error) => { engine = None; failed(error.into()) },
                };
                let snapshot = publish(&app, &cache, next, request.is_some());
                if let Some(request) = request { let _ = request.reply.send(result.unwrap_or(Ok(())).map(|_| snapshot)); }
            } else if let Some(request) = request { let _ = request.reply.send(Err(AudioError::new("unavailable", "Служба Windows Audio недоступна."))); }
            last_refresh = Instant::now();
        }
    }
    drop(engine); // unregister all watchers once, on the COM worker
}
