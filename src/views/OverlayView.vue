<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { isTauri } from '@tauri-apps/api/core'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { Pin, PinOff, ArrowUpRight, X, Volume2, VolumeX } from '@lucide/vue'
import BrandMark from '../components/ui/BrandMark.vue'
import MediaCard from '../components/ui/MediaCard.vue'
import DiscordActions from '../components/ui/DiscordActions.vue'
import { useMediaStore } from '../stores/media'
import MascotDisplay from '../components/ui/MascotDisplay.vue'
import { useAudioStore } from '../stores/audio'
import { useDiscordStore } from '../stores/discord'
import { useDesktopStore } from '../stores/desktop'
import type { MascotState } from '../types/mascot'
const audio = useAudioStore(), discord = useDiscordStore(), desktop = useDesktopStore()
const media = useMediaStore()
const card = ref<HTMLElement>()
const transient = ref<MascotState | null>(null)
let timer: ReturnType<typeof setTimeout> | undefined
let observer: ResizeObserver | undefined
const state = computed<MascotState>(() => audio.ready && (audio.muted || audio.volume === 0) ? 'sleeping' : transient.value ?? (audio.snapshot.status === 'error' ? 'error' : discord.snapshot.mute === true ? 'mute' : media.playing || (!media.available && audio.hasActivePlayback) ? 'listening' : 'idle'))
watch([() => audio.muted, () => audio.volume === 0, () => discord.snapshot.mute], () => { clearTimeout(timer); transient.value = null })
watch(() => media.reaction, reaction => {
  if (!reaction) return
  clearTimeout(timer); transient.value = reaction.kind
  timer = setTimeout(() => { transient.value = null }, reaction.kind === 'next' ? 1800 : 3500)
})
watch(() => discord.reaction, () => { clearTimeout(timer); transient.value = 'error'; timer = setTimeout(() => { transient.value = null }, 3500) })
function drag(event: MouseEvent) {
  if (isTauri() && event.button === 0) void getCurrentWindow().startDragging().catch(cause => { desktop.error = String(cause) })
}
function fitContents() {
  if (isTauri() && desktop.ready && card.value) {
    void desktop.command('fit_overlay', { height: Math.ceil(card.value.getBoundingClientRect().height + 16) })
  }
}
watch(() => desktop.ready, ready => { if (ready) fitContents() })
onMounted(() => {
  if (!isTauri() || !card.value) return
  observer = new ResizeObserver(fitContents)
  observer.observe(card.value)
})
onUnmounted(() => { observer?.disconnect(); clearTimeout(timer) })
</script>
<template>
  <main class="overlay-shell" aria-label="Компактная панель LUNÉA">
    <section ref="card" class="overlay-card">
      <header class="overlay-header">
        <div class="overlay-drag" @mousedown="drag"><BrandMark/><strong>LUNÉA</strong><span>AUDIO</span></div>
        <button class="overlay-icon" :aria-pressed="desktop.preferences.pinned" :aria-label="desktop.preferences.pinned ? 'Открепить оверлей' : 'Закрепить оверлей'" :title="desktop.preferences.pinned ? 'Закреплён: не скрывается при потере фокуса' : 'Закрепить: запретить автоскрытие'" :disabled="!desktop.native" @click="desktop.configure({ pinned: !desktop.preferences.pinned })"><Pin v-if="!desktop.preferences.pinned" :size="15" aria-hidden="true"/><PinOff v-else :size="15" aria-hidden="true"/></button>
        <button class="overlay-icon" aria-label="Открыть LUNÉA" title="Открыть главное окно" :disabled="!desktop.native" @click="desktop.command('open_main', { settings: false })"><ArrowUpRight :size="17" aria-hidden="true"/></button>
        <button class="overlay-icon overlay-close" aria-label="Скрыть оверлей" title="Скрыть оверлей" :disabled="!desktop.native" @click="desktop.command('hide_overlay')"><X :size="16" aria-hidden="true"/></button>
      </header>
      <section class="overlay-volume" aria-label="Системная громкость Windows">
        <div class="overlay-volume-label"><span>Громкость <small>· Windows</small></span><output>{{ audio.ready ? Math.round(audio.volume) : '—' }}<small>%</small></output></div>
        <div class="overlay-slider-row"><button class="overlay-icon" :aria-label="audio.muted ? 'Включить системный звук' : 'Выключить системный звук'" :aria-pressed="audio.muted" :disabled="!audio.ready" @click="audio.setMuted(!audio.muted)"><VolumeX v-if="audio.muted" :size="18" aria-hidden="true"/><Volume2 v-else :size="18" aria-hidden="true"/></button><input :value="audio.volume" type="range" min="0" max="100" :disabled="!audio.ready" aria-label="Системная громкость" @input="audio.setVolume(Number(($event.target as HTMLInputElement).value))"/></div>
        <p class="overlay-native-label" :title="audio.snapshot.endpoint?.name">{{ audio.ready ? audio.muted ? 'Системный звук выключен' : audio.snapshot.endpoint?.name : audio.snapshot.status === 'loading' ? 'Подключение Windows Audio…' : 'Windows Audio недоступен' }}</p>
        <button class="overlay-mixer-link" :disabled="!desktop.native" @click="desktop.command('open_mixer')">Полный микшер · {{ audio.sessions.length }} сессий →</button>
      </section>
      <MediaCard compact/>
      <div class="overlay-companion"><MascotDisplay :state="state" size="compact"/><DiscordActions compact/></div>
      <p v-if="discord.snapshot.lastAttempt || discord.error" class="overlay-error" role="status">{{ discord.error || (discord.snapshot.lastAttempt?.status==='sent'?'Сочетание отправлено. Состояние неизвестно.':discord.snapshot.lastAttempt?.message) }}</p>
      <p v-if="desktop.error || desktop.persistenceError" class="overlay-error" role="status">{{ desktop.error || desktop.persistenceError }}</p>
      <p v-if="audio.error || audio.snapshot.message" class="overlay-error" role="status">{{ audio.error || audio.snapshot.message }}</p>
      <p class="overlay-footnote">{{ desktop.preferences.pinned ? 'Закреплено' : desktop.preferences.autoHide ? 'Автоскрытие включено' : 'Автоскрытие выключено' }}<span>{{ desktop.preferences.shortcut }}</span></p>
    </section>
  </main>
</template>

