<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { Power } from '@lucide/vue'
import { useDesktopStore } from '../../stores/desktop'
const desktop = useDesktopStore()
const busy = ref(false)
async function update(patch: { enabled?: boolean; startInTray?: boolean }) {
  if (busy.value) return
  busy.value = true
  try { await desktop.configureStartup(patch) }
  finally { busy.value = false }
}
onMounted(() => { if (desktop.native) void desktop.refresh() })
</script>
<template>
  <section class="panel desktop-settings" aria-labelledby="startup-settings-title">
    <div class="desktop-settings-heading"><Power :size="23" aria-hidden="true"/><div><h2 id="startup-settings-title">Запуск и системный трей</h2><p>Обычный запуск всегда открывает главное окно.</p></div></div>
    <label class="desktop-check"><input type="checkbox" :checked="desktop.autostartEnabled === true" :disabled="!desktop.native || !desktop.ready || busy || desktop.autostartEnabled === null" @change="update({ enabled: ($event.target as HTMLInputElement).checked })"/><span>Запускать вместе с Windows</span></label>
    <label class="desktop-check"><input type="checkbox" :checked="desktop.preferences.startInTray" :disabled="!desktop.native || !desktop.ready || busy" @change="update({ startInTray: ($event.target as HTMLInputElement).checked })"/><span>Запускать свёрнутым в трей</span></label>
    <p class="desktop-status" role="status">{{ !desktop.native ? 'Автозапуск доступен в приложении Windows.' : busy ? 'Сохранение…' : desktop.autostartEnabled === null ? 'Состояние автозапуска пока неизвестно.' : desktop.autostartEnabled ? 'Автозапуск Windows включён.' : 'Автозапуск Windows выключен.' }}</p>
    <p class="fine-print">Запуск в трее применяется только при входе в Windows и включённом автозапуске. Крестик скрывает окно, «Свернуть» оставляет его в панели задач. Двойной клик по иконке трея открывает LUNÉA; правый клик — меню с пунктом «Выход».</p>
    <p v-if="desktop.autostartError" class="desktop-error" role="alert">{{ desktop.autostartError }}</p>
    <button v-if="desktop.autostartError" class="demo-action" :disabled="busy" @click="desktop.refresh()">Проверить снова</button>
  </section>
</template>
