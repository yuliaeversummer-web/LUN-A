import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke, isTauri } from '@tauri-apps/api/core'
import type { DesktopSnapshot, OverlayPreferences } from '../types/desktop'
export const useDesktopStore = defineStore('desktop', () => {
  const native = isTauri()
  const ready = ref(false), error = ref('')
  const preferences = ref<OverlayPreferences>({ shortcut: 'Ctrl+D', pinned: false, autoHide: true, position: null, theme: null, startInTray: false })
  const autostartEnabled = ref<boolean | null>(null), autostartError = ref<string | null>(null)
  const shortcutRegistered = ref(false)
  const shortcutError = ref<string | null>(null), persistenceError = ref<string | null>(null)
  let revision = -1
  function apply(snapshot: DesktopSnapshot) {
    if (snapshot.revision < revision) return
    revision = snapshot.revision
    preferences.value = snapshot.preferences; shortcutRegistered.value = snapshot.shortcutRegistered
    shortcutError.value = snapshot.shortcutError; persistenceError.value = snapshot.persistenceError; ready.value = true
    autostartEnabled.value = snapshot.autostartEnabled; autostartError.value = snapshot.autostartError
  }
  async function command<T>(name: string, args?: Record<string, unknown>): Promise<T | undefined> {
    if (!native) { error.value = 'Управление окнами доступно в приложении Windows.'; return }
    error.value = ''
    try { return await invoke<T>(name, args) }
    catch (cause) { error.value = String(cause); return }
  }
  async function configure(patch: Partial<Pick<OverlayPreferences, 'shortcut' | 'pinned' | 'autoHide'>>) {
    const snapshot = await command<DesktopSnapshot>('configure_overlay', patch)
    if (snapshot) apply(snapshot)
    return !!snapshot
  }
  async function configureStartup(patch: { enabled?: boolean; startInTray?: boolean }) {
    const snapshot = await command<DesktopSnapshot>('configure_startup', patch)
    if (snapshot) apply(snapshot)
    return !!snapshot && !snapshot.autostartError
  }
  async function refresh() {
    const snapshot = await command<DesktopSnapshot>('get_desktop_state')
    if (snapshot) apply(snapshot)
  }
  return { native, ready, error, preferences, shortcutRegistered, shortcutError, persistenceError, autostartEnabled, autostartError, apply, configure, configureStartup, refresh, command }
})
