use super::{Session, devices, events::{Dirty, SessionEvents, SessionNotification}, icons};
use std::collections::HashMap;
use windows::{core::Interface, Win32::{Media::Audio::*, System::Com::CLSCTX_ALL}};

pub struct SessionHandle { pub control: IAudioSessionControl2, pub volume: ISimpleAudioVolume, callback: Option<IAudioSessionEvents> }
impl Drop for SessionHandle { fn drop(&mut self) { if let Some(callback) = &self.callback { unsafe { let _ = self.control.UnregisterAudioSessionNotification(callback); } } } }
pub struct Sessions {
    manager: IAudioSessionManager2, callback: Option<IAudioSessionNotification>, dirty: Dirty,
    device_id: String, device_name: String,
    pub handles: HashMap<String, SessionHandle>, metadata: HashMap<u32, (Option<String>, Option<String>)>,
}
impl Sessions {
    pub fn new(device: &IMMDevice, dirty: Dirty) -> windows::core::Result<Self> {
        unsafe {
            let device_id = devices::id(device)?;
            let device_name = devices::name(device);
            let manager: IAudioSessionManager2 = device.Activate(CLSCTX_ALL, None)?;
            let callback: IAudioSessionNotification = SessionNotification(dirty.clone()).into();
            let registered = manager.RegisterSessionNotification(&callback).is_ok();
            Ok(Self { manager, callback: registered.then_some(callback), dirty, device_id, device_name, handles: HashMap::new(), metadata: HashMap::new() })
        }
    }
    pub fn observing(&self) -> bool { self.callback.is_some() && self.handles.values().all(|session| session.callback.is_some()) }
    pub fn read(&mut self) -> windows::core::Result<Vec<Session>> {
        unsafe {
            let enumerator = self.manager.GetSessionEnumerator()?;
            let mut rows = Vec::new(); let mut live = std::collections::HashSet::new();
            for index in 0..enumerator.GetCount()? {
                let row = (|| -> windows::core::Result<Session> {
                    let control: IAudioSessionControl2 = enumerator.GetSession(index)?.cast()?;
                    let state = control.GetState()?;
                    if state == AudioSessionStateExpired { return Err(windows::core::Error::empty()); }
                    let id = devices::take_string(control.GetSessionInstanceIdentifier()?);
                    if id.is_empty() { return Err(windows::core::Error::empty()); }
                    live.insert(id.clone());
                    if !self.handles.contains_key(&id) {
                        let volume: ISimpleAudioVolume = control.cast()?;
                        let callback: IAudioSessionEvents = SessionEvents(self.dirty.clone()).into();
                        let registered = control.RegisterAudioSessionNotification(&callback).is_ok();
                        self.handles.insert(id.clone(), SessionHandle { control: control.clone(), volume, callback: registered.then_some(callback) });
                    }
                    let session = &self.handles[&id];
                    let pid = control.GetProcessId().unwrap_or(0);
                    let system = control.IsSystemSoundsSession() == windows::core::HRESULT(0);
                    let metadata = self.metadata.entry(pid).or_insert_with(|| {
                        let path = icons::process_path(pid);
                        let name = path.as_ref().and_then(|path| std::path::Path::new(path).file_stem()).map(|name| name.to_string_lossy().into_owned());
                        let icon = path.as_ref().and_then(|path| icons::app_icon(path)); (name, icon)
                    });
                    let display = control.GetDisplayName().ok().map(|text| devices::take_string(text)).filter(|text| !text.is_empty() && !text.starts_with('@'));
                    Ok(Session { id, device_id: self.device_id.clone(), device_name: self.device_name.clone(), pid, name: if system { "Системные звуки".into() } else { display.or_else(|| metadata.0.clone()).unwrap_or_else(|| if pid > 0 { format!("Приложение · PID {pid}") } else { "Аудиосессия Windows".into() }) },
                        icon: metadata.1.clone(), volume: session.volume.GetMasterVolume()? * 100.0,
                        muted: session.volume.GetMute()?.as_bool(), state: if state == AudioSessionStateActive { "active" } else { "inactive" }.into(), system })
                })();
                if let Ok(row) = row { rows.push(row); }
            }
            self.handles.retain(|id, _| live.contains(id));
            self.metadata.retain(|pid, _| rows.iter().any(|row| row.pid == *pid));
            rows.sort_by(|a,b| a.state.cmp(&b.state).then_with(|| a.name.cmp(&b.name)).then_with(|| a.id.cmp(&b.id)));
            Ok(rows)
        }
    }
}
impl Drop for Sessions {
    fn drop(&mut self) {
        self.handles.clear();
        if let Some(callback) = &self.callback { unsafe { let _ = self.manager.UnregisterSessionNotification(callback); } }
    }
}
