// Independent of Vue/Tauri so delayed replies, dropped events and teardown can
// be exercised without sending real audio or keyboard commands.
export function snapshotChannel<T extends { revision: number }>(options: {
  read: () => Promise<T>
  subscribe: (receive: (value: T) => void) => Promise<() => void>
  visible: () => Promise<boolean>
  apply: (value: T, live: boolean) => void
  error: (cause: unknown) => void
  intervalMs?: number
  timeoutMs?: number
}) {
  let disposed = false, active = false, reading = false, subscribing = false
  let stop: (() => void) | undefined, latest: T | undefined
  let revision = -1, failed = false
  let visibilityEpoch = 0
  const timers = new Set<ReturnType<typeof setTimeout>>()
  async function bounded<R>(operation: Promise<R>): Promise<R> {
    let timer: ReturnType<typeof setTimeout> | undefined
    try {
      return await Promise.race([operation, new Promise<never>((_, reject) => {
        timer = setTimeout(() => { timers.delete(timer!); reject(new Error('Сервис не ответил вовремя. Выполняется повторная сверка.')) }, options.timeoutMs ?? 5_000)
        timers.add(timer)
      })])
    } finally { if (timer) { clearTimeout(timer); timers.delete(timer) } }
  }
  function receive(value: T, live = true) {
    if (disposed || value.revision < (latest?.revision ?? revision)) return
    latest = value
    if (!active || (!failed && value.revision === revision)) return
    options.apply(value, live); revision = value.revision; failed = false
  }
  async function reconcile() {
    if (disposed || reading) return
    reading = true
    try {
      const epoch = visibilityEpoch
      const visible = await bounded(options.visible())
      if (epoch === visibilityEpoch) active = visible
      if (disposed) return
      if (active && latest) receive(latest, false)
      // Cached DTO reads only: never retry a mutating command or a Toggle.
      receive(await bounded(options.read()), false)
    } catch (cause) { if (!disposed && (active || revision < 0)) { failed = true; options.error(cause) } }
    finally { reading = false }
  }
  async function connect() {
    if (disposed || stop || subscribing) return
    subscribing = true
    try {
      const unsubscribe = await options.subscribe(receive)
      if (disposed) unsubscribe(); else stop = unsubscribe
    } catch (cause) { if (!disposed && active) { failed = true; options.error(cause) } }
    finally { subscribing = false }
  }
  async function refresh() { void connect(); await reconcile() }
  const interval = setInterval(() => { void refresh() }, options.intervalMs ?? 15_000)
  void refresh()
  return {
    refresh,
    // Native show/hide notifications avoid waiting for the watchdog interval.
    visibility(visible: boolean) {
      if (disposed) return
      visibilityEpoch++
      active = visible
      if (visible) { if (latest) receive(latest, false); void refresh() }
    },
    dispose() {
      if (disposed) return
      disposed = true; clearInterval(interval); stop?.(); latest = undefined
      for (const timer of timers) clearTimeout(timer)
      timers.clear()
    },
  }
}
