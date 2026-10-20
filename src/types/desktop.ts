import type { ThemeId } from './theme'
export interface OverlayPreferences {
  shortcut: string; pinned: boolean; autoHide: boolean
  position: [number, number] | null; theme: ThemeId | null
  startInTray: boolean
}
export interface DesktopSnapshot {
  revision: number; preferences: OverlayPreferences
  shortcutRegistered: boolean; shortcutError: string | null; persistenceError: string | null
  autostartEnabled: boolean | null; autostartError: string | null
}
