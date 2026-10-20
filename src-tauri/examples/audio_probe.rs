//! Explicit local QA tool, never launched by the application or bundled.
//! Default is read-only. `fixture` renders silence in two separate sessions.
#[cfg(windows)]
fn main() -> windows::core::Result<()> {
    use windows::{core::{Interface, GUID, w}, Win32::{Media::Audio::{*, Endpoints::*}, System::Com::*}};
    use serde_json::json;
    unsafe {
        CoInitializeEx(None, COINIT_MULTITHREADED).ok()?;
        // Run in an inner scope so COM interfaces drop before CoUninitialize.
        let result = (|| -> windows::core::Result<()> {
            let enumerator: IMMDeviceEnumerator = CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)?;
            let device = enumerator.GetDefaultAudioEndpoint(eRender, eConsole)?;
            let endpoint: IAudioEndpointVolume = device.Activate(CLSCTX_ALL, None)?;
            let args: Vec<String> = std::env::args().collect();
            match args.get(1).map(String::as_str) {
                Some("volume") => {
                    let volume: f32 = args.get(2).and_then(|arg| arg.parse().ok()).filter(|value| (0.0..=100.0).contains(value)).expect("explicit volume 0..100 required");
                    endpoint.SetMasterVolumeLevelScalar(volume / 100.0, std::ptr::null())?;
                }
                Some("mute") => endpoint.SetMute(args.get(2).map(String::as_str) == Some("true"), std::ptr::null())?,
                Some("fixture") => {
                    let mut clients = Vec::new();
                    for (index, guid) in [GUID::from_u128(0x615a8026_5984_43e8_9fb6_f4a120554501), GUID::from_u128(0x615a8026_5984_43e8_9fb6_f4a120554502)].iter().enumerate() {
                        let client: IAudioClient = device.Activate(CLSCTX_ALL, None)?;
                        let format = client.GetMixFormat()?;
                        let result = client.Initialize(AUDCLNT_SHAREMODE_SHARED, 0, 1_000_000, 0, format, Some(guid));
                        CoTaskMemFree(Some(format.cast())); result?;
                        let render: IAudioRenderClient = client.GetService()?;
                        let control: IAudioSessionControl = client.GetService()?;
                        control.SetDisplayName(if index == 0 { w!("LUNÉA QA · сессия A") } else { w!("LUNÉA QA · сессия B") }, std::ptr::null())?;
                        let frames = client.GetBufferSize()?;
                        render.GetBuffer(frames)?; render.ReleaseBuffer(frames, AUDCLNT_BUFFERFLAGS_SILENT.0 as u32)?;
                        client.Start()?; clients.push((client, render, frames));
                    }
                    println!("{}", json!({"fixturePid":std::process::id(),"sessions":2,"audio":"silence"}));
                    let seconds = args.get(2).and_then(|arg| arg.parse::<u64>().ok()).unwrap_or(60).min(300);
                    let until = std::time::Instant::now() + std::time::Duration::from_secs(seconds);
                    while std::time::Instant::now() < until {
                        for (client, render, capacity) in &clients {
                            let available = capacity.saturating_sub(client.GetCurrentPadding()?);
                            if available > 0 { render.GetBuffer(available)?; render.ReleaseBuffer(available, AUDCLNT_BUFFERFLAGS_SILENT.0 as u32)?; }
                        }
                        std::thread::sleep(std::time::Duration::from_millis(20));
                    }
                    for (client, _, _) in clients { client.Stop()?; }
                }
                _ => {},
            }
            let manager: IAudioSessionManager2 = device.Activate(CLSCTX_ALL, None)?;
            let sessions = manager.GetSessionEnumerator()?; let mut rows = Vec::new();
            for index in 0..sessions.GetCount()? {
                let control: IAudioSessionControl2 = sessions.GetSession(index)?.cast()?;
                let volume: ISimpleAudioVolume = control.cast()?;
                rows.push(json!({"pid":control.GetProcessId().unwrap_or(0),"volume":volume.GetMasterVolume().ok().map(|v|v*100.0),"muted":volume.GetMute().ok().map(|v|v.as_bool())}));
            }
            let microphone = enumerator.GetDefaultAudioEndpoint(eCapture, eConsole).ok().and_then(|device| device.Activate::<IAudioEndpointVolume>(CLSCTX_ALL, None).ok());
            println!("{}", json!({"volume":endpoint.GetMasterVolumeLevelScalar()?*100.0,"muted":endpoint.GetMute()?.as_bool(),"sessions":rows,
                "microphone":microphone.map(|mic|json!({"volume":mic.GetMasterVolumeLevelScalar().ok(),"muted":mic.GetMute().ok().map(|v|v.as_bool())}))}));
            Ok(())
        })();
        CoUninitialize(); result
    }
}
#[cfg(not(windows))] fn main() { eprintln!("Windows-only QA"); }
