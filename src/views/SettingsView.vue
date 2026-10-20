<script setup lang="ts">
import { Check, Palette } from '@lucide/vue'
import { themes } from '../types/theme'
import { useThemeStore } from '../stores/theme'
import MascotDisplay from '../components/ui/MascotDisplay.vue'
import { useMascotReactions } from '../composables/useMascotReactions'
import OverlaySettings from '../components/ui/OverlaySettings.vue'
import StartupSettings from '../components/ui/StartupSettings.vue'
import MediaSettings from '../components/ui/MediaSettings.vue'
import DiscordSettings from '../components/ui/DiscordSettings.vue'
const theme = useThemeStore()
const mascot = useMascotReactions()
</script>
<template><div class="page-heading settings-mascot-heading"><div><p class="eyebrow">ТВОЁ ЛИЧНОЕ ПРОСТРАНСТВО</p><h1>Настройки</h1><p>Настроение меняется. LUNÉA меняется с тобой.</p></div><div class="settings-mascot"><Palette class="heading-icon" :size="24" aria-hidden="true"/><MascotDisplay :state="mascot.state.value" size="medium"/></div></div>
<section class="panel settings-panel"><h2>Цветовая тема</h2><p>Выбери свою частоту. Тема сохраняется на этом устройстве.</p><div class="theme-grid"><button v-for="item in themes" :key="item.id" class="theme-card" :aria-pressed="theme.selected === item.id" :style="{ '--swatch': item.accent }" @click="theme.selected = item.id"><span class="theme-preview"><span class="preview-orb"/><Check v-if="theme.selected === item.id" :size="20"/></span><strong>{{ item.name }}</strong><small>{{ item.description }}</small></button></div><p v-if="theme.persistenceError" role="status">Тема применена, но браузер запретил локальное сохранение.</p></section>
<MediaSettings/><DiscordSettings/><StartupSettings/><OverlaySettings/><p class="fine-print">LUNÉA 0.1.0 · Данные остаются на устройстве</p></template>

