import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import { invoke, isTauri } from '@tauri-apps/api/core'
import { synchronizeService } from '../services/serviceSync'
import type { AudioSession, AudioSnapshot } from '../types/audio'

export const useAudioStore = defineStore('audio', () => {
  const snapshot = ref<AudioSnapshot>({ revision: -1, status: 'loading', endpoint: null, sessions: [], outputs: [], inputs: [], message: null, warning: null })
  const error = ref(''), refreshing = ref(false)
  const pending = ref<Record<string, { token: number; value: number | boolean; command: string; args: Record<string, unknown> }>>({})
  const running = new Set<string>()
  let sequence = 0, initialized = false, disposed = false
  const ready = computed(() => snapshot.value.status === 'ready' && !!snapshot.value.endpoint)
  const masterKey = (field: string) => `master:${snapshot.value.endpoint?.deviceId}:${field}`
  const sessionKey = (id: string, field: string) => `session:${id}:${field}`
  const volume = computed(() => Number(pending.value[masterKey('volume')]?.value ?? snapshot.value.endpoint?.volume ?? 0))
  const muted = computed(() => Boolean(pending.value[masterKey('muted')]?.value ?? snapshot.value.endpoint?.muted ?? false))
  const sessions = computed(() => snapshot.value.sessions)
  const hasActivePlayback = computed(() => ready.value && !muted.value && volume.value > 0 && sessions.value.some(session => session.state === 'active' && !session.muted && session.volume > 0))
  function sessionVolume(session: AudioSession) { return Number(pending.value[sessionKey(session.id, 'volume')]?.value ?? session.volume) }
  function sessionMuted(session: AudioSession) { return Boolean(pending.value[sessionKey(session.id, 'muted')]?.value ?? session.muted) }
  function apply(next: AudioSnapshot) {
    if (disposed || next.revision < snapshot.value.revision) return
    snapshot.value = next
    for (const [key, job] of Object.entries(pending.value)) {
      if (job.args.deviceId && job.args.deviceId !== next.endpoint?.deviceId || job.args.sessionId && !next.sessions.some(s => s.id === job.args.sessionId)) delete pending.value[key]
    }
  }
  function message(cause: unknown) { return cause && typeof cause === 'object' && 'message' in cause ? String(cause.message) : String(cause) }
  async function enqueue(key: string, command: string, args: Record<string, unknown>, value: number | boolean) {
    error.value = ''
    pending.value[key] = { token: ++sequence, command, args, value }
    if (running.has(key)) return
    running.add(key)
    try {
      while (!disposed && pending.value[key]) {
        // Coalesce input, retaining the latest thumb position across old replies.
        await new Promise(resolve => setTimeout(resolve, 40))
        const job = pending.value[key]; if (!job) break
        try { apply(await invoke<AudioSnapshot>(job.command, job.args)) }
        catch (cause) { error.value = message(cause) }
        if (pending.value[key]?.token === job.token) delete pending.value[key]
      }
    } finally { running.delete(key) }
  }
  function setVolume(value: number) {
    const deviceId = snapshot.value.endpoint?.deviceId
    if (!ready.value || !deviceId) return
    return enqueue(masterKey('volume'), 'set_master_volume', { deviceId, volume: value }, value)
  }
  function setMuted(value: boolean) {
    const deviceId = snapshot.value.endpoint?.deviceId
    if (!ready.value || !deviceId) return
    return enqueue(masterKey('muted'), 'set_master_mute', { deviceId, muted: value }, value)
  }
  function setSessionVolume(id: string, value: number) { return enqueue(sessionKey(id, 'volume'), 'set_session_volume', { sessionId: id, volume: value }, value) }
  function setSessionMuted(id: string, value: boolean) { return enqueue(sessionKey(id, 'muted'), 'set_session_mute', { sessionId: id, muted: value }, value) }
  async function refresh() {
    if (!isTauri() || refreshing.value) return
    refreshing.value = true; error.value = ''
    try { apply(await invoke<AudioSnapshot>('refresh_audio')) } catch (cause) { error.value = message(cause) }
    finally { refreshing.value = false }
  }
  async function openSettings() { try { await invoke('open_sound_settings') } catch (cause) { error.value = message(cause) } }
  async function initialize() {
    if (initialized) return
    initialized = true
    if (!isTauri()) { snapshot.value.status = 'unavailable'; snapshot.value.message = 'Настоящий микшер доступен в приложении LUNÉA для Windows.'; return }
    let syncError = false
    synchronizeService<AudioSnapshot>('lunea:audio-state', 'get_audio_state', next => {
      if (syncError) { error.value = ''; syncError = false }
      apply(next)
    }, cause => {
      syncError = true
      snapshot.value = { ...snapshot.value, status: 'error' }; error.value = message(cause)
    }, () => { disposed = true; pending.value = {} })
  }
  return { snapshot, error, refreshing, ready, volume, muted, sessions, hasActivePlayback, sessionVolume, sessionMuted, setVolume, setMuted, setSessionVolume, setSessionMuted, apply, initialize, refresh, openSettings }
})
