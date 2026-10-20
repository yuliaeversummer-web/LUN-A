<script setup lang="ts">
import { ref, watch } from 'vue'
import { Play, Pause, SkipBack, SkipForward } from '@lucide/vue'
import BrandMark from './BrandMark.vue'
import { useMediaStore } from '../../stores/media'
import { useDesktopStore } from '../../stores/desktop'
withDefaults(defineProps<{compact?:boolean}>(),{compact:false})
const media=useMediaStore(), desktop=useDesktopStore()
const draft=ref<number|null>(null)
watch(()=>media.snapshot.track?.token,()=>{draft.value=null})
function time(ms:number|undefined){if(ms===undefined||!Number.isFinite(ms)||ms<0)return '--:--';const seconds=Math.floor(ms/1000);return `${Math.floor(seconds/60).toString().padStart(2,'0')}:${(seconds%60).toString().padStart(2,'0')}`}
async function seek(){const value=draft.value;if(value===null)return;try{await media.seek(value)}finally{draft.value=null}}
</script>
<template>
 <section class="media-card" :class="compact?'media-compact':'panel media-full'" aria-label="Яндекс Музыка через Windows GSMTC" :aria-busy="media.snapshot.status==='loading'">
  <div class="media-heading"><strong>Яндекс Музыка</strong><span>{{ media.snapshot.status==='loading'?'Подключение…':media.available?media.playing?'Воспроизведение':'Пауза / остановлено':'Нет подключения' }} · GSMTC</span></div>
  <div class="media-content">
   <div class="media-artwork"><img v-if="media.artwork" :key="media.snapshot.track?.artworkId ?? ''" :src="media.artwork" :alt="`Обложка: ${media.snapshot.track?.title || 'текущая композиция'}`" width="80" height="80" decoding="async"/><BrandMark v-else/><span v-if="!compact && media.snapshot.track && !media.artwork">Нет обложки</span></div>
   <div class="media-copy"><strong :title="media.snapshot.track?.title">{{ media.snapshot.track?.title || (media.available?'Название недоступно':media.snapshot.status==='loading'?'Чтение композиции…':'Яндекс Музыка недоступна') }}</strong><span :title="media.snapshot.track?.artist">{{ media.snapshot.track ? media.snapshot.track.artist || 'Исполнитель не указан' : 'Десктопное приложение Windows' }}</span><small v-if="!compact && media.snapshot.track?.album" :title="media.snapshot.track.album">{{ media.snapshot.track.album }}</small></div>
   <div class="media-controls"><button :disabled="!media.available || !media.snapshot.capabilities.previous || media.busy" aria-label="Предыдущая композиция" :title="media.snapshot.capabilities.previous?'Предыдущая композиция':'Плеер не поддерживает Previous'" @click="media.command('media_previous')"><SkipBack :size="16" aria-hidden="true"/></button><button class="media-toggle" :disabled="!media.canToggle || media.busy" :aria-label="media.playing?'Пауза Яндекс Музыки':'Воспроизвести Яндекс Музыку'" :title="media.canToggle?'Управление десктопной Яндекс Музыкой':'Play/Pause недоступен у выбранной сессии'" @click="media.command('media_toggle_playback')"><Pause v-if="media.playing" :size="17" aria-hidden="true"/><Play v-else :size="17" aria-hidden="true"/></button><button :disabled="!media.available || !media.snapshot.capabilities.next || media.busy" aria-label="Следующая композиция" :title="media.snapshot.capabilities.next?'Следующая композиция':'Плеер не поддерживает Next'" @click="media.command('media_next')"><SkipForward :size="16" aria-hidden="true"/></button></div>
  </div>
  <div class="media-progress"><input type="range" :value="draft ?? media.snapshot.timeline?.positionMs ?? 0" :min="media.snapshot.timeline?.minSeekMs ?? 0" :max="Math.max(1,media.snapshot.timeline?.maxSeekMs ?? 1)" step="1000" aria-label="Позиция композиции" :title="media.canSeek?'Перемотка через Windows GSMTC':'Плеер не предоставил доступную перемотку'" :disabled="!media.canSeek || media.busy" @input="draft=Number(($event.target as HTMLInputElement).value)" @change="seek"/><div><span :title="media.snapshot.timeline?.estimated?'Оценка между подтверждёнными обновлениями Windows':'Подтверждённая позиция Windows'">{{ time(media.snapshot.timeline?.positionMs) }}{{ media.snapshot.timeline?.estimated?' ≈':'' }}</span><span>{{ media.snapshot.timeline && media.snapshot.timeline.durationMs>0?time(media.snapshot.timeline.durationMs):'--:--' }}</span></div></div>
  <p v-if="media.error || media.snapshot.commandError" class="media-error" role="alert">{{ media.error || media.snapshot.commandError?.message }}</p>
  <div v-if="!media.available && media.snapshot.status!=='loading'" class="media-unavailable"><p role="status">{{ media.snapshot.message || 'Windows пока не предоставил сессию Яндекс Музыки.' }}</p><button v-if="!compact" class="text-link" :disabled="media.busy || !desktop.native" @click="media.refresh">Проверить подключение</button><button v-else class="text-link" :disabled="!desktop.native" @click="desktop.command('open_main',{settings:true})">Настроить источник</button></div>
  <p v-if="!compact && media.snapshot.warning" class="fine-print" role="status">{{ media.snapshot.warning }}</p>
 </section>
</template>
