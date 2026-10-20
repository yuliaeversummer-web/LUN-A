use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf, str::FromStr, sync::{Mutex, atomic::{AtomicBool, Ordering}}, time::Duration};
use tauri::{menu::{Menu, MenuItem, PredefinedMenuItem}, tray::{TrayIconBuilder, TrayIconEvent, MouseButton}, AppHandle, Emitter, Manager, PhysicalPosition, PhysicalRect, State, WindowEvent};
use tauri_plugin_autostart::ManagerExt;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

const EVENT: &str = "lunea:desktop-state";

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Preferences {
    pub shortcut: String,
    pub pinned: bool,
    pub auto_hide: bool,
    pub position: Option<(i32, i32)>,
    pub theme: Option<String>,
    pub start_in_tray: bool,
}
impl Default for Preferences {
    fn default() -> Self { Self { shortcut: "Ctrl+D".into(), pinned: false, auto_hide: true, position: None, theme: None, start_in_tray: false } }
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub revision: u64, pub preferences: Preferences,
    pub shortcut_registered: bool, pub shortcut_error: Option<String>, pub persistence_error: Option<String>,
    pub autostart_enabled: Option<bool>, pub autostart_error: Option<String>,
}
struct Data { snapshot: Snapshot, focus_epoch: u64 }
pub struct DesktopState { data: Mutex<Data>, path: PathBuf, quitting: AtomicBool }

pub const AUTOSTART_ARG: &str = "--autostart";
pub fn is_autostart(args: impl IntoIterator<Item = String>) -> bool { args.into_iter().any(|arg| arg == AUTOSTART_ARG) }
fn start_hidden(automatic: bool, start_in_tray: bool) -> bool { automatic && start_in_tray }
fn read_autostart(app: &AppHandle) -> (Option<bool>, Option<String>) {
    match app.autolaunch().is_enabled() {
        Ok(enabled) => (Some(enabled), None),
        Err(_) => (None, Some("Не удалось проверить автозапуск Windows. Попробуйте снова.".into())),
    }
}

fn valid_theme(value: &str) -> bool { matches!(value, "neon-pink" | "arctic-blue" | "forest-gold" | "crimson-night") }
fn parse_shortcut(value: &str) -> Result<Shortcut, String> {
    let shortcut = Shortcut::from_str(value).map_err(|_| "Некорректное сочетание. Используйте Ctrl или Alt и клавишу.".to_string())?;
    if !shortcut.mods.intersects(Modifiers::CONTROL | Modifiers::ALT) || shortcut.mods.intersects(Modifiers::SUPER | Modifiers::META) {
        return Err("Используйте Ctrl или Alt; клавиша Windows не поддерживается.".into());
    }
    Ok(shortcut)
}
fn snapshot(app: &AppHandle) -> Result<Snapshot, String> {
    let state = app.try_state::<DesktopState>().ok_or("Состояние ещё инициализируется")?;
    let result = state.data.lock().map_err(|_| "Состояние недоступно")?.snapshot.clone();
    Ok(result)
}
pub fn overlay_shortcut(app:&AppHandle)->String {snapshot(app).map(|s|s.preferences.shortcut).unwrap_or_else(|_|"Ctrl+D".into())}
fn publish(app: &AppHandle, data: &mut Data) -> Result<Snapshot, String> {
    data.snapshot.revision += 1;
    app.emit(EVENT, &data.snapshot).map_err(|e| e.to_string())?;
    Ok(data.snapshot.clone())
}
fn save(state: &DesktopState, data: &mut Data) {
    let result = (|| -> Result<(), Box<dyn std::error::Error>> {
        if let Some(parent) = state.path.parent() { fs::create_dir_all(parent)?; }
        let temporary = state.path.with_extension("tmp");
        fs::write(&temporary, serde_json::to_vec_pretty(&data.snapshot.preferences)?)?;
        fs::rename(temporary, &state.path)?;
        Ok(())
    })();
    data.snapshot.persistence_error = result.err().map(|_| "Не удалось сохранить настройки на устройстве.".into());
}

pub fn setup(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let path = app.path().app_config_dir()?.join("desktop.json");
    let mut load_error = None;
    let mut preferences = match fs::read(&path) {
        Ok(bytes) => serde_json::from_slice::<Preferences>(&bytes).unwrap_or_else(|_| { load_error = Some("Файл настроек повреждён; применены значения по умолчанию.".into()); Preferences::default() }),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Preferences::default(),
        Err(_) => { load_error = Some("Не удалось прочитать настройки; применены значения по умолчанию.".into()); Preferences::default() },
    };
    if preferences.theme.as_deref().is_some_and(|theme| !valid_theme(theme)) { preferences.theme = None; }
    if parse_shortcut(&preferences.shortcut).is_err() { preferences.shortcut = "Ctrl+D".into(); }
    let hidden = start_hidden(is_autostart(std::env::args()), preferences.start_in_tray);
    let (autostart_enabled, autostart_error) = read_autostart(app);
    app.manage(DesktopState { path, quitting: AtomicBool::new(false), data: Mutex::new(Data { focus_epoch: 0, snapshot: Snapshot {
        revision: 0, preferences, shortcut_registered: false, shortcut_error: None, persistence_error: load_error,
        autostart_enabled, autostart_error,
    } }) });
    let shortcut = snapshot(app)?.preferences.shortcut;
    let registration = register_shortcut(app, &shortcut);
    {
        let state = app.state::<DesktopState>();
        let mut data = state.data.lock().map_err(|_| "State poisoned")?;
        data.snapshot.shortcut_registered = registration.is_ok();
        data.snapshot.shortcut_error = registration.err();
    }
    let open = MenuItem::with_id(app, "open", "Открыть LUNÉA", true, None::<&str>)?;
    let overlay = MenuItem::with_id(app, "overlay", "Показать/скрыть оверлей", true, None::<&str>)?;
    let settings = MenuItem::with_id(app, "settings", "Настройки", true, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app)?;
    let exit = MenuItem::with_id(app, "exit", "Выход", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&open, &overlay, &settings, &separator, &exit])?;
    let icon = app.default_window_icon().ok_or("Missing app icon")?.clone();
    TrayIconBuilder::with_id("lunea-tray").icon(icon).tooltip("LUNÉA — Audio Control")
        .menu(&menu).show_menu_on_left_click(false)
        .on_tray_icon_event(|tray, event| {
            if matches!(event, TrayIconEvent::DoubleClick { button: MouseButton::Left, .. }) {
                if let Err(error) = open_main_window(tray.app_handle(), false) { eprintln!("LUNÉA tray: {error}"); }
            }
        })
        .on_menu_event(|app, event| {
            let result = match event.id.as_ref() {
                "open" => open_main_window(app, false),
                "settings" => open_main_window(app, true),
                "overlay" => toggle_overlay_window(app),
                "exit" => { exit_app(app); Ok(()) },
                _ => Ok(()),
            };
            if let Err(error) = result { eprintln!("LUNÉA tray: {error}"); }
        }).build(app)?;
    if !hidden { open_main_window(app, false)?; }
    Ok(())
}

fn exit_app(app: &AppHandle) {
    if let Some(state) = app.try_state::<DesktopState>() { state.quitting.store(true, Ordering::SeqCst); }
    save_preferences(app);
    let _ = app.global_shortcut().unregister_all();
    app.exit(0);
}

fn refresh_autostart(app: &AppHandle) {
    let Some(state) = app.try_state::<DesktopState>() else { return };
    let (enabled, error) = read_autostart(app);
    if let Ok(mut data) = state.data.lock() {
        if data.snapshot.autostart_enabled != enabled || data.snapshot.autostart_error != error {
            data.snapshot.autostart_enabled = enabled;
            data.snapshot.autostart_error = error;
            let _ = publish(app, &mut data);
        }
    };
}

fn register_shortcut(app: &AppHandle, value: &str) -> Result<(), String> {
    let shortcut = parse_shortcut(value)?;
    app.global_shortcut().on_shortcut(shortcut, |app, _, event| {
        if event.state() == ShortcutState::Pressed {
            if let Err(error) = toggle_overlay_window(app) { eprintln!("LUNÉA shortcut: {error}"); }
        }
    }).map_err(|_| format!("Сочетание {value} занято или недоступно. Выберите другое в настройках."))
}

pub fn save_preferences(app: &AppHandle) {
    if let Some(state) = app.try_state::<DesktopState>() {
        if let Ok(mut data) = state.data.lock() { save(&state, &mut data); }
    }
}

pub fn open_main_window(app: &AppHandle, settings: bool) -> Result<(), String> {
    let main = app.get_webview_window("main").ok_or("Главное окно недоступно")?;
    main.show().map_err(|e| e.to_string())?;
    main.unminimize().map_err(|e| e.to_string())?;
    let _ = app.emit_to("main", "lunea:view-visible", true);
    if settings { app.emit_to("main", "lunea:navigate", "/settings").map_err(|e| e.to_string())?; }
    main.set_focus().map_err(|e| e.to_string())
}

fn contains(area: &PhysicalRect<i32, u32>, position: (i32, i32), size: tauri::PhysicalSize<u32>) -> bool {
    let (x, y) = (i64::from(position.0), i64::from(position.1));
    x >= i64::from(area.position.x) && y >= i64::from(area.position.y)
        && x + i64::from(size.width) <= i64::from(area.position.x) + i64::from(area.size.width)
        && y + i64::from(size.height) <= i64::from(area.position.y) + i64::from(area.size.height)
}

fn place_overlay(app: &AppHandle) -> Result<(), String> {
    let window = app.get_webview_window("overlay").ok_or("Оверлей недоступен")?;
    let size = window.outer_size().map_err(|e| e.to_string())?;
    let saved = snapshot(app)?.preferences.position;
    let monitors = window.available_monitors().map_err(|e| e.to_string())?;
    let saved = saved.filter(|position| monitors.iter().any(|monitor| contains(monitor.work_area(), *position, size)));
    let position = if let Some(position) = saved { position } else {
        let monitor = app.get_webview_window("main").and_then(|main| main.current_monitor().ok().flatten())
            .or_else(|| window.primary_monitor().ok().flatten()).ok_or("Рабочая область экрана недоступна")?;
        let area = monitor.work_area();
        let gap = (16.0 * monitor.scale_factor()).round() as i32;
        ((i64::from(area.position.x) + i64::from(area.size.width) - i64::from(size.width) - i64::from(gap)).max(i64::from(area.position.x)) as i32,
         (area.position.y + gap).min((i64::from(area.position.y) + i64::from(area.size.height) - i64::from(size.height)).max(i64::from(area.position.y)) as i32))
    };
    window.set_position(PhysicalPosition::new(position.0, position.1)).map_err(|e| e.to_string())
}

pub fn hide_overlay_window(app: &AppHandle) -> Result<(), String> {
    let window = app.get_webview_window("overlay").ok_or("Оверлей недоступен")?;
    window.hide().map_err(|e| e.to_string())?;
    let _ = app.emit_to("overlay", "lunea:view-visible", false);
    save_preferences(app);
    Ok(())
}
pub fn toggle_overlay_window(app: &AppHandle) -> Result<(), String> {
    let window = app.get_webview_window("overlay").ok_or("Оверлей недоступен")?;
    if window.is_visible().map_err(|e| e.to_string())? { hide_overlay_window(app) }
    else {
        place_overlay(app)?;
        window.show().map_err(|e| e.to_string())?;
        let _ = app.emit_to("overlay", "lunea:view-visible", true);
        window.set_focus().map_err(|e| e.to_string())
    }
}

pub fn window_event(window: &tauri::Window, event: &WindowEvent) {
    let app = window.app_handle();
    match event {
        WindowEvent::CloseRequested { api, .. } => {
            if app.try_state::<DesktopState>().is_some_and(|state| state.quitting.load(Ordering::SeqCst)) { return; }
            api.prevent_close();
            let _ = window.hide();
            let _ = app.emit_to(window.label(), "lunea:view-visible", false);
            save_preferences(app);
        }
        WindowEvent::Focused(true) if window.label() == "main" => { refresh_autostart(app); let _ = app.emit_to("main", "lunea:view-visible", true); }
        WindowEvent::Resized(_) => {
            let visible = window.is_visible().unwrap_or(false) && !window.is_minimized().unwrap_or(true);
            let _ = app.emit_to(window.label(), "lunea:view-visible", visible);
        }
        WindowEvent::Moved(position) if window.label() == "overlay" => {
            if let Some(state) = app.try_state::<DesktopState>() {
                if let Ok(mut data) = state.data.lock() { data.snapshot.preferences.position = Some((position.x, position.y)); }
            }
        }
        WindowEvent::Focused(focused) if window.label() == "overlay" => {
            let Some(state) = app.try_state::<DesktopState>() else { return };
            let epoch = if let Ok(mut data) = state.data.lock() { data.focus_epoch += 1; data.focus_epoch } else { return };
            if !focused {
                let app = app.clone();
                // Ignore transient focus bounces; never hide a focused slider/control.
                std::thread::spawn(move || {
                    std::thread::sleep(Duration::from_millis(250));
                    let handle = app.clone();
                    let _ = app.run_on_main_thread(move || {
                        let should_hide = handle.try_state::<DesktopState>().is_some_and(|state| state.data.lock().map(|data|
                            data.focus_epoch == epoch && data.snapshot.preferences.auto_hide && !data.snapshot.preferences.pinned).unwrap_or(false));
                        if should_hide && handle.get_webview_window("overlay").is_some_and(|window| !window.is_focused().unwrap_or(true)) {
                            let _ = hide_overlay_window(&handle);
                        } else { save_preferences(&handle); }
                    });
                });
            }
        }
        _ => {},
    }
}

#[tauri::command]
pub fn get_desktop_state(app: AppHandle) -> Result<Snapshot, String> { refresh_autostart(&app); snapshot(&app) }

#[tauri::command]
pub fn configure_startup(app: AppHandle, enabled: Option<bool>, start_in_tray: Option<bool>, state: State<'_, DesktopState>) -> Result<Snapshot, String> {
    let mut data = state.data.lock().map_err(|_| "Состояние недоступно")?;
    if let Some(enabled) = enabled {
        let result = app.autolaunch().is_enabled().and_then(|current| {
            if current == enabled { Ok(()) }
            else if enabled { app.autolaunch().enable() } else { app.autolaunch().disable() }
        });
        let (actual, query_error) = read_autostart(&app);
        data.snapshot.autostart_enabled = actual;
        data.snapshot.autostart_error = if result.is_err() || actual != Some(enabled) {
            Some("Не удалось изменить автозапуск Windows. Показано фактическое состояние; попробуйте снова.".into())
        } else { query_error };
    }
    if let Some(value) = start_in_tray { data.snapshot.preferences.start_in_tray = value; save(&state, &mut data); }
    publish(&app, &mut data)
}

#[tauri::command]
pub fn set_desktop_theme(app: AppHandle, theme: String, state: State<'_, DesktopState>) -> Result<Snapshot, String> {
    if !valid_theme(&theme) { return Err("Неизвестная тема".into()); }
    let mut data = state.data.lock().map_err(|_| "Состояние недоступно")?;
    data.snapshot.preferences.theme = Some(theme);
    save(&state, &mut data);
    publish(&app, &mut data)
}

#[tauri::command]
pub fn configure_overlay(app: AppHandle, shortcut: Option<String>, pinned: Option<bool>, auto_hide: Option<bool>, state: State<'_, DesktopState>) -> Result<Snapshot, String> {
    if let Some(value)=&shortcut { crate::discord::validate_overlay_shortcut(&app,value)?; }
    let mut data = state.data.lock().map_err(|_| "Состояние недоступно")?;
    if let Some(shortcut) = shortcut {
        let parsed = parse_shortcut(&shortcut)?;
        let old = parse_shortcut(&data.snapshot.preferences.shortcut)?;
        if parsed != old || !data.snapshot.shortcut_registered {
            if let Err(error) = register_shortcut(&app, &shortcut) {
                data.snapshot.shortcut_error = Some(error.clone());
                let _ = publish(&app, &mut data);
                return Err(error);
            }
            if data.snapshot.shortcut_registered && parsed != old {
                if app.global_shortcut().unregister(old).is_err() {
                    let _ = app.global_shortcut().unregister(parsed);
                    return Err("Не удалось снять прежнюю горячую клавишу. Настройки не изменены.".into());
                }
            }
        }
        data.snapshot.preferences.shortcut = shortcut;
        data.snapshot.shortcut_registered = true;
        data.snapshot.shortcut_error = None;
    }
    if let Some(value) = pinned { data.snapshot.preferences.pinned = value; }
    if let Some(value) = auto_hide { data.snapshot.preferences.auto_hide = value; }
    save(&state, &mut data);
    publish(&app, &mut data)
}

#[tauri::command]
pub fn toggle_overlay(app: AppHandle) -> Result<(), String> { toggle_overlay_window(&app) }
#[tauri::command]
pub fn hide_overlay(app: AppHandle) -> Result<(), String> { hide_overlay_window(&app) }
#[tauri::command]
pub fn open_main(app: AppHandle, settings: bool) -> Result<(), String> { open_main_window(&app, settings) }
#[tauri::command]
pub fn fit_overlay(window: tauri::WebviewWindow, height: f64) -> Result<(), String> {
    if window.label() != "overlay" || !height.is_finite() { return Err("Недопустимое окно или размер".into()); }
    let height = height.clamp(300.0, 440.0);
    let current = window.inner_size().map_err(|e| e.to_string())?.to_logical::<f64>(window.scale_factor().map_err(|e| e.to_string())?);
    if (current.height - height).abs() > 1.0 {
        window.set_size(tauri::LogicalSize::new(360.0, height)).map_err(|e| e.to_string())?;
        if window.is_visible().unwrap_or(false) { place_overlay(window.app_handle())?; }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn startup_visibility_requires_both_automatic_launch_and_preference() {
        assert!(!start_hidden(false, false)); assert!(!start_hidden(false, true));
        assert!(!start_hidden(true, false)); assert!(start_hidden(true, true));
        assert!(is_autostart(vec!["lunea.exe".into(), "--autostart".into()]));
        assert!(!is_autostart(vec!["lunea.exe".into(), "--autostart=false".into()]));
        let old: Preferences = serde_json::from_str(r#"{"shortcut":"Ctrl+D"}"#).unwrap();
        assert!(!old.start_in_tray);
    }
    #[test]
    fn validates_full_window_in_negative_monitor_work_area() {
        let area = PhysicalRect { position: PhysicalPosition::new(-1920, 0), size: tauri::PhysicalSize::new(1920, 1040) };
        let size = tauri::PhysicalSize::new(450, 415);
        assert!(contains(&area, (-500, 20), size));
        assert!(!contains(&area, (-100, 20), size));
        assert!(!contains(&area, (-500, 900), size));
    }
    #[test]
    fn shortcuts_require_modifier_and_support_semantic_equality() {
        assert_eq!(parse_shortcut("Ctrl+D").unwrap(), parse_shortcut("Control+KeyD").unwrap());
        assert!(parse_shortcut("D").is_err());
        assert!(parse_shortcut("not-a-shortcut").is_err());
    }
}
