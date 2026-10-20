fn main() {
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(
        tauri_build::AppManifest::new().commands(&[
            "get_desktop_state", "set_desktop_theme", "configure_overlay",
            "toggle_overlay", "hide_overlay", "open_main", "fit_overlay", "configure_startup",
            "get_audio_state", "get_audio_sessions", "get_audio_devices", "refresh_audio", "set_master_volume",
            "set_master_mute", "set_session_volume", "set_session_mute", "open_sound_settings", "open_mixer",
            "get_media_state", "get_media_artwork", "refresh_media_sessions", "select_media_session",
            "media_play", "media_pause", "media_toggle_playback", "media_next", "media_previous", "media_seek",
            "get_discord_state", "get_discord_settings", "configure_discord", "refresh_discord", "discord_toggle_mute", "discord_toggle_deafen",
        ]),
    )).expect("Unable to build LUNÉA manifest");
}
