import { computed, inject, onMounted, onUnmounted, ref, watch, type InjectionKey } from 'vue'
import { useRoute } from 'vue-router'
import { useAudioStore } from '../stores/audio'
import { useDiscordStore } from '../stores/discord'
import { useMediaStore } from '../stores/media'
import type { MascotState } from '../types/mascot'

export const MASCOT_IDLE_MS = 45_000
export const MASCOT_WELCOME_MS = 3_500
export const MASCOT_NEXT_MS = 1_800
export const MASCOT_ERROR_MS = 3_500

export function createMascotReactions() {
  const route = useRoute()
  const audio = useAudioStore()
  const discord = useDiscordStore()
  const media = useMediaStore()
  const inactive = ref(false)
  const reactions = ref<{ state: MascotState; until: number }[]>([])
  let reactionTimer: ReturnType<typeof setTimeout> | undefined
  let idleTimer: ReturnType<typeof setTimeout> | undefined
  let disposed = false

  const baseState = computed<MascotState>(() => {
    if (route.path === '/settings') return 'curious'
    if (audio.snapshot.status === 'error') return 'error'
    if (audio.ready && (audio.muted || audio.volume === 0)) return 'sleeping'
    if (discord.snapshot.mute === true) return 'mute'
    if (media.playing) return 'listening'
    if (inactive.value) return 'idle'
    if (media.available) return 'idle'
    return audio.hasActivePlayback ? 'listening' : 'idle'
  })
  const state = computed<MascotState>(() => audio.ready && (audio.muted || audio.volume === 0) ? 'sleeping' : reactions.value.at(-1)?.state ?? baseState.value)

  function scheduleReactionEnd() {
    clearTimeout(reactionTimer)
    if (disposed) return
    const now = Date.now()
    reactions.value = reactions.value.filter(reaction => reaction.until > now)
    if (!reactions.value.length) return
    const nearestEnd = Math.min(...reactions.value.map(reaction => reaction.until))
    reactionTimer = setTimeout(scheduleReactionEnd, Math.max(1, nearestEnd - now))
  }

  function clearReactions() { clearTimeout(reactionTimer); reactions.value = [] }
  function react(next: MascotState, duration: number) {
    if (disposed) return
    // A newer reaction temporarily covers an older one. Expired reactions never
    // reappear, and the current base state is restored rather than a stale copy.
    reactions.value.push({ state: next, until: Date.now() + duration })
    scheduleReactionEnd()
  }
  function markActivity() {
    if (disposed) return
    clearTimeout(idleTimer)
    inactive.value = false
    idleTimer = setTimeout(() => { inactive.value = true }, MASCOT_IDLE_MS)
  }
  function nextTrack() { markActivity(); void media.command('media_next') }
  function showError() { markActivity(); react('error', MASCOT_ERROR_MS) }

  watch([() => route.path, () => audio.ready, () => audio.snapshot.status, () => audio.muted, () => audio.volume === 0,
    () => discord.snapshot.mute], (values, previous) => {
      // Loading the real endpoint must not cancel the initial greeting when
      // sound is available. Actual mute/errors still take effect immediately.
      if (values[1] === true && previous[1] === false && !audio.muted && audio.volume > 0 && reactions.value.at(-1)?.state === 'welcome') { markActivity(); return }
      clearReactions(); markActivity()
    })
  // Loading/Playing often alternate during a real track transition. They update
  // the base pose without cancelling the accepted Next reaction or greeting.
  watch([() => audio.hasActivePlayback, () => media.playing], markActivity)
  watch(() => media.reaction, reaction => { if (reaction) { markActivity(); react(reaction.kind, reaction.kind === 'next' ? MASCOT_NEXT_MS : MASCOT_ERROR_MS) } })
  watch(() => discord.reaction, () => { markActivity(); react('error', MASCOT_ERROR_MS) })
  onMounted(() => {
    markActivity()
    if (route.path === '/') react('welcome', MASCOT_WELCOME_MS)
  })
  onUnmounted(() => { disposed = true; clearTimeout(idleTimer); clearReactions() })
  return { state, markActivity, nextTrack, showError }
}

type MascotReactions = ReturnType<typeof createMascotReactions>
export const mascotKey: InjectionKey<MascotReactions> = Symbol('lunea-mascot-reactions')
export function useMascotReactions() {
  const reactions = inject(mascotKey)
  if (!reactions) throw new Error('Mascot reactions require the main window layout')
  return reactions
}
