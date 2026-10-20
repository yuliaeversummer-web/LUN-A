use super::{devices, events::{Dirty, EndpointEvents}, Endpoint};
use windows::Win32::{Media::Audio::{*, Endpoints::*}, System::Com::CLSCTX_ALL};

pub struct EndpointHandle { pub volume: IAudioEndpointVolume, pub device: IMMDevice, pub id: String, callback: Option<IAudioEndpointVolumeCallback> }
impl EndpointHandle {
    pub fn new(device: IMMDevice, dirty: Dirty) -> windows::core::Result<Self> {
        unsafe {
            let id = devices::id(&device)?;
            let volume: IAudioEndpointVolume = device.Activate(CLSCTX_ALL, None)?;
            let callback: IAudioEndpointVolumeCallback = EndpointEvents(dirty).into();
            let registered = volume.RegisterControlChangeNotify(&callback).is_ok();
            Ok(Self { device, volume, id, callback: registered.then_some(callback) })
        }
    }
    pub fn read(&self) -> windows::core::Result<Endpoint> {
        unsafe { Ok(Endpoint { device_id: self.id.clone(), name: devices::name(&self.device), volume: self.volume.GetMasterVolumeLevelScalar()? * 100.0, muted: self.volume.GetMute()?.as_bool() }) }
    }
    pub fn observing(&self) -> bool { self.callback.is_some() }
}
impl Drop for EndpointHandle {
    fn drop(&mut self) { if let Some(callback) = &self.callback { unsafe { let _ = self.volume.UnregisterControlChangeNotify(callback); } } }
}
