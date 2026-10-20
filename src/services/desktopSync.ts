import { isTauri, invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { getCurrentWindow } from '@tauri-apps/api/window'
import type { Pinia } from 'pinia'
import { nextTick } from 'vue'
import { router } from '../app/router'
import { useThemeStore } from '../stores/theme'
import { useDesktopStore } from '../stores/desktop'
import type { DesktopSnapshot } from '../types/desktop'
import { synchronizeService } from './serviceSync'

// Desktop settings are separate from AudioService, MediaService and DiscordService.
export async function initializeDesktopSync(pinia: Pinia) {
  if (!isTauri()) return
  const theme = useThemeStore(pinia), desktop = useDesktopStore(pinia)
  let applying = false, revision = -1, disposed = false
  const disposers: (() => void)[] = []
  let previousTheme = theme.selected
  function apply(snapshot: DesktopSnapshot) {
    if (disposed || snapshot.revision < revision) return
    revision = snapshot.revision; applying = true
    if (snapshot.preferences.theme) theme.selected = snapshot.preferences.theme
    desktop.apply(snapshot); previousTheme = theme.selected
    applying = false
  }
  async function send(name: string, args: Record<string, unknown>) {
    try { apply(await invoke<DesktopSnapshot>(name, args)) }
    catch (cause) { if (!disposed) desktop.error = String(cause) }
  }
  let syncError = false
  const stopSync = synchronizeService<DesktopSnapshot>('lunea:desktop-state', 'get_desktop_state', value => {
    if (syncError) { desktop.error = ''; syncError = false }
    apply(value)
    if (!value.preferences.theme) void send('set_desktop_theme', { theme: theme.selected })
  }, cause => { syncError = true; desktop.error = String(cause) })
  const cleanup = () => {
    if (disposed) return
    disposed = true; stopSync(); disposers.forEach(stop => stop())
    window.removeEventListener('pagehide', cleanup)
  }
  window.addEventListener('pagehide', cleanup, { once: true })
  if (import.meta.hot) import.meta.hot.dispose(cleanup)
  try {
    if (getCurrentWindow().label === 'main') {
      const stop = await listen<string>('lunea:navigate', event => {
        if (event.payload === '/settings') void router.push('/settings')
        if (event.payload === '/mixer') void router.push('/').then(async () => { await nextTick(); document.getElementById('mixer-title')?.scrollIntoView({ block: 'start' }) })
      })
      if (disposed) { stop(); return }
      disposers.push(stop)
    }
    disposers.push(theme.$subscribe(() => {
      if (applying || disposed || previousTheme === theme.selected) return
      previousTheme = theme.selected
      void send('set_desktop_theme', { theme: theme.selected })
    }, { flush: 'sync', detached: true }))
  } catch (cause) { desktop.error = String(cause) }
}
