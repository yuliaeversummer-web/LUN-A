<script setup lang="ts">
import { AudioLines, Volume2, VolumeX, RefreshCw, MonitorSpeaker, Mic, ArrowUpRight } from '@lucide/vue'
import { useAudioStore } from '../../stores/audio'
const audio = useAudioStore()
const deviceState: Record<string, string> = { active: 'Подключено', disabled: 'Отключено в Windows', unplugged: 'Не подключено', notPresent: 'Недоступно' }
</script>
<template>
  <section class="panel native-mixer" aria-labelledby="mixer-title" :aria-busy="audio.snapshot.status === 'loading'">
    <div class="card-heading"><span class="icon-tile"><AudioLines :size="21" aria-hidden="true"/></span><div><h2 id="mixer-title">Аудиомикшер Windows</h2><p>{{ audio.snapshot.endpoint?.name ?? 'Устройство по умолчанию' }}</p></div><span class="chip">{{ audio.ready ? 'Подключено' : audio.snapshot.status === 'loading' ? 'Загрузка' : 'Недоступно' }}</span><button class="icon-button" aria-label="Обновить Windows Audio" :disabled="audio.refreshing" @click="audio.refresh"><RefreshCw :size="17" aria-hidden="true"/></button></div>
    <p v-if="audio.snapshot.message || audio.error" class="audio-notice" role="status">{{ audio.error || audio.snapshot.message }}</p>
    <div class="master-volume"><span>Общая громкость <small>· системная</small></span><strong>{{ audio.ready ? Math.round(audio.volume) : '—' }}<small>%</small></strong></div>
    <div class="volume-row"><button class="icon-button" :aria-label="audio.muted ? 'Включить системный звук' : 'Выключить системный звук'" :aria-pressed="audio.muted" :disabled="!audio.ready" @click="audio.setMuted(!audio.muted)"><VolumeX v-if="audio.muted" :size="20" aria-hidden="true"/><Volume2 v-else :size="20" aria-hidden="true"/></button><input :value="audio.volume" aria-label="Системная громкость" type="range" min="0" max="100" :disabled="!audio.ready" @input="audio.setVolume(Number(($event.target as HTMLInputElement).value))"/></div>
    <p v-if="audio.muted && audio.ready" class="fine-print">Системный звук выключен. Уровень громкости сохранён.</p>
    <div class="audio-session-heading"><h3>Приложения</h3><span>{{ audio.sessions.length }} сессий · все активные выходы</span></div>
    <div class="audio-sessions" aria-label="Аудиосессии Windows" tabindex="0">
      <article v-for="(session, index) in audio.sessions" :key="session.id" class="audio-session">
        <img v-if="session.icon" :src="session.icon" alt="" width="24" height="24"/><AudioLines v-else :size="24" aria-hidden="true"/>
        <div class="audio-session-info"><strong :title="session.name">{{ session.name }}</strong><small>{{ session.state === 'active' ? 'Воспроизводит' : 'Ожидает' }} · PID {{ session.pid }} · сессия {{ index + 1 }}</small><small>{{ session.deviceName }}</small></div>
        <button class="icon-button" :aria-label="`${audio.sessionMuted(session) ? 'Включить' : 'Выключить'} звук: ${session.name}, сессия ${index + 1}`" :aria-pressed="audio.sessionMuted(session)" :disabled="!audio.ready" @click="audio.setSessionMuted(session.id, !audio.sessionMuted(session))"><VolumeX v-if="audio.sessionMuted(session)" :size="17" aria-hidden="true"/><Volume2 v-else :size="17" aria-hidden="true"/></button>
        <input :value="audio.sessionVolume(session)" :aria-label="`Громкость: ${session.name}, сессия ${index + 1}`" type="range" min="0" max="100" :disabled="!audio.ready" @input="audio.setSessionVolume(session.id, Number(($event.target as HTMLInputElement).value))"/><output>{{ Math.round(audio.sessionVolume(session)) }}%</output>
      </article>
      <p v-if="!audio.sessions.length" class="fine-print">{{ audio.ready ? 'Пока нет аудиосессий. Включите звук в браузере или плеере.' : 'Сессии появятся после подключения Windows Audio.' }}</p>
    </div>
    <p v-if="audio.snapshot.warning" class="fine-print" role="status">{{ audio.snapshot.warning }}</p>
    <p class="fine-print">Каждая строка — отдельная сессия Windows. Сессии одного приложения не объединяются.</p>
  </section>
  <section class="panel audio-devices" aria-labelledby="devices-title">
    <div class="audio-devices-heading"><h2 id="devices-title">Устройства</h2><button class="text-link" @click="audio.openSettings">Параметры звука Windows <ArrowUpRight :size="15" aria-hidden="true"/></button></div>
    <p class="fine-print">Просмотр устройств. Громкость выше управляет выходом Windows по умолчанию. Сменить его можно в параметрах Windows.</p>
    <div class="audio-device-columns"><div v-for="group in [{ label: 'Вывод', devices: audio.snapshot.outputs, icon: MonitorSpeaker }, { label: 'Ввод', devices: audio.snapshot.inputs, icon: Mic }]" :key="group.label"><h3><component :is="group.icon" :size="17" aria-hidden="true"/>{{ group.label }}</h3><ul><li v-for="device in group.devices" :key="device.id"><strong>{{ device.name }}</strong><span>{{ deviceState[device.state] }}{{ device.defaultConsole ? ' · по умолчанию' : '' }}{{ device.defaultMultimedia && !device.defaultConsole ? ' · мультимедиа' : '' }}{{ device.defaultCommunications ? ' · связь' : '' }}</span></li><li v-if="!group.devices.length">Устройства не найдены.</li></ul></div></div>
    <p class="fine-print">Микрофоны отображаются только для просмотра. Их громкость и mute не изменяются.</p>
  </section>
</template>
