// Native integration fixture, not bundled: reserve Ctrl+Alt+F8 for 30 seconds.
// It sends no input and does not change Discord or Windows settings.
#[cfg(windows)]
fn main() -> windows::core::Result<()> {
    use windows::Win32::UI::Input::KeyboardAndMouse::{RegisterHotKey, UnregisterHotKey, MOD_ALT, MOD_CONTROL, MOD_NOREPEAT};
    unsafe { RegisterHotKey(None, 71, MOD_CONTROL | MOD_ALT | MOD_NOREPEAT, 0x77)?; }
    struct Registration;
    impl Drop for Registration {
        fn drop(&mut self) { unsafe { let _ = UnregisterHotKey(None, 71); } }
    }
    let _registration = Registration;
    println!("Ctrl+Alt+F8 reserved; no keyboard input is sent.");
    std::thread::sleep(std::time::Duration::from_secs(30));
    Ok(())
}
#[cfg(not(windows))]
fn main() {}
