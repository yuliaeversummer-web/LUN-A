<script setup lang="ts">
import { Music2 } from '@lucide/vue'
import { useMediaStore } from '../../stores/media'
import { useDesktopStore } from '../../stores/desktop'
const media=useMediaStore(), desktop=useDesktopStore()
</script>
<template>
 <section class="panel desktop-settings media-settings" aria-labelledby="media-settings-title">
  <div class="desktop-settings-heading"><Music2 :size="23" aria-hidden="true"/><div><h2 id="media-settings-title">Яндекс Музыка</h2><p>Реальная десктопная сессия Windows GSMTC. Логин, токены и cookies не нужны.</p></div></div>
  <p role="status">{{ media.available?'Источник подключён.':media.snapshot.message || 'Обнаружение источника…' }}</p>
  <p class="media-source">{{ media.snapshot.selectedSource || media.snapshot.preferredSource || 'Источник не выбран' }} · {{ media.snapshot.selectionMode==='auto'?'автоматически':'ручной выбор' }}</p>
  <div class="media-settings-actions"><button class="demo-action" :disabled="media.busy || !desktop.native" @click="media.refresh">Обновить обнаружение</button><button class="demo-action" :disabled="media.busy || !desktop.native" @click="media.select(null)">Выбирать автоматически</button></div>
  <label v-for="(candidate,index) in media.snapshot.candidates.filter(s=>s.eligible)" :key="candidate.id" class="desktop-check"><input type="radio" name="music-source" :checked="media.snapshot.selectionMode==='manual' && media.snapshot.selectedId===candidate.id" :disabled="media.busy || !desktop.native" @change="media.select(candidate.id)"/><span>{{ candidate.name }} · сессия {{ index+1 }}<small>{{ candidate.sourceId }}</small></span></label>
  <p class="fine-print">Автоматически выбирается единственная подходящая сессия. Если их несколько — только текущая сессия Windows среди проверенных источников; иначе нужен ручной выбор. Сохраняется ID приложения, а не временный ID сессии. Браузеры и другие плееры не выбираются.</p>
  <details v-if="media.snapshot.candidates.some(s=>!s.eligible)"><summary>Другие источники Windows · не управляются</summary><p v-for="candidate in media.snapshot.candidates.filter(s=>!s.eligible)" :key="candidate.id" class="media-source">{{ candidate.sourceId }}</p></details>
  <p v-if="media.error || media.snapshot.persistenceError" class="media-error" role="alert">{{ media.error || media.snapshot.persistenceError }}</p>
 </section>
</template>
