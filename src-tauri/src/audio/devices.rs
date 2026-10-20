use super::Device;
use windows::{core::PWSTR, Win32::{Devices::FunctionDiscovery::PKEY_Device_FriendlyName, Media::Audio::*, System::Com::{CoTaskMemFree, STGM_READ, StructuredStorage::{PropVariantClear, PropVariantToStringAlloc}}}};

// Every PWSTR returned by Core Audio is task-allocated, including failed/empty
// metadata. Free it on this same COM worker after converting to owned text.
pub unsafe fn take_string(value: PWSTR) -> String {
    let text = unsafe { value.to_string() }.unwrap_or_default();
    unsafe { CoTaskMemFree(Some(value.0.cast())) }; text
}
pub fn id(device: &IMMDevice) -> windows::core::Result<String> { unsafe { Ok(take_string(device.GetId()?)) } }
pub fn name(device: &IMMDevice) -> String {
    unsafe {
        let result = (|| -> windows::core::Result<String> {
            let properties = device.OpenPropertyStore(STGM_READ)?;
            let mut value = properties.GetValue(&PKEY_Device_FriendlyName)?;
            let result = PropVariantToStringAlloc(&value).map(|text| take_string(text));
            let _ = PropVariantClear(&mut value);
            result
        })();
        result.ok().filter(|text| !text.is_empty()).unwrap_or_else(|| "Аудиоустройство Windows".into())
    }
}
pub fn default_id(enumerator: &IMMDeviceEnumerator, flow: EDataFlow, role: ERole) -> Option<String> {
    unsafe { enumerator.GetDefaultAudioEndpoint(flow, role).ok().and_then(|device| id(&device).ok()) }
}
pub fn list(enumerator: &IMMDeviceEnumerator, flow: EDataFlow) -> windows::core::Result<Vec<Device>> {
    let defaults = [eConsole, eMultimedia, eCommunications].map(|role| default_id(enumerator, flow, role));
    unsafe {
        let collection = enumerator.EnumAudioEndpoints(flow, DEVICE_STATE(DEVICE_STATEMASK_ALL))?;
        let mut devices = Vec::new();
        for index in 0..collection.GetCount()? {
            let Ok(device) = collection.Item(index) else { continue };
            let Ok(id) = id(&device) else { continue };
            let state = device.GetState().unwrap_or(DEVICE_STATE_NOTPRESENT);
            devices.push(Device { name: name(&device), default_console: defaults[0].as_ref() == Some(&id),
                default_multimedia: defaults[1].as_ref() == Some(&id), default_communications: defaults[2].as_ref() == Some(&id), id,
                state: if state == DEVICE_STATE_ACTIVE { "active" } else if state == DEVICE_STATE_DISABLED { "disabled" }
                    else if state == DEVICE_STATE_UNPLUGGED { "unplugged" } else { "notPresent" }.into() });
        }
        devices.sort_by(|a,b| b.default_console.cmp(&a.default_console).then_with(|| a.name.cmp(&b.name)));
        Ok(devices)
    }
}
