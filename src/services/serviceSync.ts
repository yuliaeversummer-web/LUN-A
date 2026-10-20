import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { snapshotChannel } from './snapshotChannel'

export function synchronizeService<T extends { revision: number }>(event: string, command: string, apply: (value: T, live: boolean) => void, error: (cause: unknown) => void, disposeState: () => void = () => {}) {
  const windowHandle = getCurrentWindow()
  let disposed = false, visibilityEpoch = 0
  const channel = snapshotChannel<T>({
    read: () => invoke<T>(command),
    subscribe: receive => listen<T>(event, event => receive(event.payload)),
    visible: async () => {
      const epoch = visibilityEpoch
      const visible = await windowHandle.isVisible() && !await windowHandle.isMinimized()
      if (!disposed && epoch === visibilityEpoch) document.documentElement.dataset.viewHidden = String(!visible)
      return visible
    },
    apply, error,
  })
  const setVisible = (visible: boolean) => {
    visibilityEpoch++
    document.documentElement.dataset.viewHidden = String(!visible)
    channel.visibility(visible)
  }
  const stops: (() => void)[] = []
  const watch = async () => {
    const stop = await listen<boolean>('lunea:view-visible', event => setVisible(event.payload), { target: windowHandle.label })
    if (disposed) stop(); else stops.push(stop)
  }
  void watch().catch(cause => { if (!disposed) error(cause) })
  const refresh = () => { void channel.refresh() }
  // The initially hidden Tauri windows can finish loading immediately before
  // setup shows main. Recover that startup ordering without a 15-second blank.
  const startupTimers = [setTimeout(refresh, 250), setTimeout(refresh, 1_000)]
  window.addEventListener('focus', refresh)
  document.addEventListener('visibilitychange', refresh)
  const cleanup = () => {
    if (disposed) return
    disposed = true; channel.dispose(); stops.forEach(stop => stop()); disposeState()
    startupTimers.forEach(clearTimeout)
    window.removeEventListener('focus', refresh)
    document.removeEventListener('visibilitychange', refresh)
    window.removeEventListener('pagehide', cleanup)
  }
  window.addEventListener('pagehide', cleanup, { once: true })
  if (import.meta.hot) import.meta.hot.dispose(cleanup)
  return cleanup
}
