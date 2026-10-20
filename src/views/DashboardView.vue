<script setup lang="ts">
import { AudioLines, ArrowRight, Sparkles, Play, Pause, SkipForward, Mic } from '@lucide/vue'
import AudioMixer from '../components/ui/AudioMixer.vue'
import MediaCard from '../components/ui/MediaCard.vue'
import DiscordCard from '../components/ui/DiscordCard.vue'
import { useMediaStore } from '../stores/media'
import MascotDisplay from '../components/ui/MascotDisplay.vue'
import { mascotPoses } from '../types/mascot'
import { useDiscordStore } from '../stores/discord'
import { useMascotReactions } from '../composables/useMascotReactions'

const discord = useDiscordStore()
const media = useMediaStore()
const mascot = useMascotReactions()
</script>
<template>
<div class="page-heading"><div><p class="eyebrow">НАСТРОЙСЯ НА СВОЮ ВОЛНУ</p><h1>Твой звук. Твои правила.</h1><p>Одно пространство для музыки, общения и тишины.</p></div><AudioLines class="heading-icon" :size="30"/></div>
<section class="mascot-scene" aria-label="Лунэя — твоя музыкальная спутница">
  <div class="mascot-copy"><span class="eyebrow"><Sparkles :size="14" aria-hidden="true"/> ЗНАКОМЬСЯ, ЛУНЭЯ</span>
    <h2 aria-live="polite">{{ mascotPoses[mascot.state.value].caption }}</h2>
    <p>Твоя музыка, твоё пространство.<br/>А я побуду рядом.</p>
    <RouterLink to="/settings" class="text-link">Выбрать настроение <ArrowRight :size="16" aria-hidden="true"/></RouterLink>
    <div class="demo-track"><span class="chip">GSMTC</span><span>{{ media.snapshot.track?.title || 'Нет композиции' }}</span></div>
  </div>
  <div class="mascot-composition"><div class="mascot-halo" aria-hidden="true"/><MascotDisplay :state="mascot.state.value" size="large"/></div>
  <div class="mascot-demo-controls" aria-label="Музыка и реакции маскота">
    <button class="demo-action demo-action-primary" :aria-pressed="media.playing" :disabled="!media.canToggle || media.busy" @click="media.command('media_toggle_playback')"><Pause v-if="media.playing" :size="16" aria-hidden="true"/><Play v-else :size="16" aria-hidden="true"/>{{ media.playing ? 'Пауза' : 'Воспроизвести' }}</button>
    <button class="demo-action" :disabled="!media.available || !media.snapshot.capabilities.next || media.busy" @click="mascot.nextTrack"><SkipForward :size="16" aria-hidden="true"/>Следующий трек</button>
    <button class="demo-action" :disabled="!discord.canSend" aria-label="Переключить микрофон Discord" :title="discord.snapshot.message" @click="discord.toggle('mute')"><Mic :size="16" aria-hidden="true"/>Микрофон Discord</button>
  </div>
  <p class="mascot-demo-note">Музыка — Windows GSMTC. Discord — настроенные сочетания; состояние микрофона неизвестно.</p>
</section>
<div class="section-heading"><h2>Аудиопространство</h2><span>WINDOWS AUDIO</span></div>
<AudioMixer/><div class="media-and-discord"><MediaCard/><DiscordCard/></div>
</template>
