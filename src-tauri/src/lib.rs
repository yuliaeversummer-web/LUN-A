mod commands;
mod desktop;
mod audio;
mod media;
mod discord;
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, args, _| {
            if !desktop::is_autostart(args) { let _ = desktop::open_main_window(app, false); }
        }))
        .plugin(tauri_plugin_autostart::Builder::new().args([desktop::AUTOSTART_ARG]).build())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .invoke_handler(tauri::generate_handler![desktop::get_desktop_state,
            desktop::set_desktop_theme, desktop::configure_overlay, desktop::toggle_overlay,
            desktop::hide_overlay, desktop::open_main, desktop::fit_overlay, desktop::configure_startup,
            audio::get_audio_state, audio::get_audio_sessions, audio::get_audio_devices, audio::refresh_audio,
            audio::set_master_volume, audio::set_master_mute, audio::set_session_volume, audio::set_session_mute, audio::open_sound_settings, audio::open_mixer,
            media::get_media_state, media::get_media_artwork, media::refresh_media_sessions, media::select_media_session,
            media::media_play, media::media_pause, media::media_toggle_playback, media::media_next, media::media_previous, media::media_seek,
            discord::get_discord_state, discord::get_discord_settings, discord::configure_discord, discord::refresh_discord, discord::discord_toggle_mute, discord::discord_toggle_deafen])
        .setup(|app| { audio::setup(app.handle())?; media::setup(app.handle())?; desktop::setup(app.handle())?; discord::setup(app.handle()) })
        .on_window_event(desktop::window_event)
        .build(tauri::generate_context!())
        .expect("Unable to launch LUNÉA")
        .run(|app, event| { if matches!(event, tauri::RunEvent::Exit) { discord::shutdown(app); media::shutdown(app); audio::shutdown(app); } });
}
