<script setup lang="ts">
import { MessageCircle, Volume2, VolumeX } from '@lucide/vue'
import DiscordActions from './DiscordActions.vue'
import { useDiscordStore } from '../../stores/discord'
import { useAudioStore } from '../../stores/audio'
const discord=useDiscordStore(),audio=useAudioStore()
</script>
<template><section class="panel discord-card" aria-labelledby="discord-card-title">
 <div class="discord-heading"><MessageCircle :size="20" aria-hidden="true"/><h3 id="discord-card-title">Discord</h3><span>{{ discord.snapshot.preferences.client.toUpperCase() }}</span></div>
 <p class="discord-availability" role="status">{{ discord.snapshot.message }}</p><DiscordActions/>
 <p class="fine-print">Горячая клавиша — действие, а не подтверждение Mute/Deafen.</p>
 <p v-if="discord.error || discord.snapshot.lastAttempt" class="discord-feedback" :role="discord.error || discord.snapshot.lastAttempt?.status==='not_sent'?'alert':'status'">{{ discord.error || discord.snapshot.lastAttempt?.message }}</p>
 <RouterLink to="/settings#discord" class="text-link">Настроить Discord →</RouterLink>
 <div class="discord-session-list" aria-label="Аудиосессии выбранного Discord" tabindex="0">
  <article v-for="(session,index) in discord.snapshot.audioSessions" :key="session.id" class="discord-session"><div><strong>Аудиосессия {{ index+1 }} · PID {{ session.pid }}</strong><small>{{ session.deviceName }}</small></div><div class="discord-session-controls"><button :disabled="!audio.ready" :aria-pressed="audio.sessionMuted(session)" :aria-label="`Mute аудиосессии Discord ${index+1}`" title="Отключить только эту аудиосессию Windows. Это не Deafen." @click="audio.setSessionMuted(session.id,!audio.sessionMuted(session))"><VolumeX v-if="audio.sessionMuted(session)" :size="16" aria-hidden="true"/><Volume2 v-else :size="16" aria-hidden="true"/></button><input type="range" min="0" max="100" :value="audio.sessionVolume(session)" :aria-label="`Громкость аудиосессии Discord ${index+1}`" :disabled="!audio.ready" @input="audio.setSessionVolume(session.id,Number(($event.target as HTMLInputElement).value))"/><output>{{ Math.round(audio.sessionVolume(session)) }}%</output></div></article>
  <p v-if="!discord.snapshot.audioSessions.length" class="fine-print">{{ discord.running?'Discord пока не создал аудиосессию на активных выходах Windows.':'Аудиосессии недоступны.' }}</p>
 </div><p v-if="audio.error" class="discord-feedback" role="alert">{{ audio.error }}</p><p class="fine-print">Громкость и mute каждой сессии Windows независимы от Deafen.</p>
</section></template>
