#![allow(non_snake_case)]
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use windows::{core::{implement, Ref, PCWSTR, GUID, BOOL, Result}, Win32::{Foundation::PROPERTYKEY, Media::Audio::{*, Endpoints::*}}};

// Core Audio invokes these on its own threads. Never block, query COM, unregister,
// emit UI events, or move a supplied session pointer out of a callback.
#[derive(Clone)]
pub struct Dirty(pub Arc<AtomicBool>);
impl Dirty { pub fn mark(&self) -> Result<()> { self.0.store(true, Ordering::Release); Ok(()) } }

#[implement(IAudioEndpointVolumeCallback)]
pub struct EndpointEvents(pub Dirty);
impl IAudioEndpointVolumeCallback_Impl for EndpointEvents_Impl {
    fn OnNotify(&self, _: *mut AUDIO_VOLUME_NOTIFICATION_DATA) -> Result<()> { self.0.mark() }
}
#[implement(IAudioSessionNotification)]
pub struct SessionNotification(pub Dirty);
impl IAudioSessionNotification_Impl for SessionNotification_Impl {
    fn OnSessionCreated(&self, _: Ref<IAudioSessionControl>) -> Result<()> { self.0.mark() }
}
#[implement(IAudioSessionEvents)]
pub struct SessionEvents(pub Dirty);
impl IAudioSessionEvents_Impl for SessionEvents_Impl {
    fn OnDisplayNameChanged(&self, _: &PCWSTR, _: *const GUID) -> Result<()> { self.0.mark() }
    fn OnIconPathChanged(&self, _: &PCWSTR, _: *const GUID) -> Result<()> { self.0.mark() }
    fn OnSimpleVolumeChanged(&self, _: f32, _: BOOL, _: *const GUID) -> Result<()> { self.0.mark() }
    fn OnChannelVolumeChanged(&self, _: u32, _: *const f32, _: u32, _: *const GUID) -> Result<()> { self.0.mark() }
    fn OnGroupingParamChanged(&self, _: *const GUID, _: *const GUID) -> Result<()> { self.0.mark() }
    fn OnStateChanged(&self, _: AudioSessionState) -> Result<()> { self.0.mark() }
    fn OnSessionDisconnected(&self, _: AudioSessionDisconnectReason) -> Result<()> { self.0.mark() }
}
#[implement(IMMNotificationClient)]
pub struct DeviceEvents(pub Dirty);
impl IMMNotificationClient_Impl for DeviceEvents_Impl {
    fn OnDeviceStateChanged(&self, _: &PCWSTR, _: DEVICE_STATE) -> Result<()> { self.0.mark() }
    fn OnDeviceAdded(&self, _: &PCWSTR) -> Result<()> { self.0.mark() }
    fn OnDeviceRemoved(&self, _: &PCWSTR) -> Result<()> { self.0.mark() }
    fn OnDefaultDeviceChanged(&self, _: EDataFlow, _: ERole, _: &PCWSTR) -> Result<()> { self.0.mark() }
    fn OnPropertyValueChanged(&self, _: &PCWSTR, _: &PROPERTYKEY) -> Result<()> { self.0.mark() }
}
