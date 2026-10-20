<script setup lang="ts">
import { ref, watch } from 'vue'
import { PanelsTopLeft } from '@lucide/vue'
import { useDesktopStore } from '../../stores/desktop'
const desktop = useDesktopStore()
const shortcut = ref(desktop.preferences.shortcut)
const saving = ref(false)
watch(() => desktop.preferences.shortcut, value => { shortcut.value = value })
async function saveShortcut() {
  saving.value = true
  try { await desktop.configure({ shortcut: shortcut.value.trim() }) }
  finally { saving.value = false }
}
</script>
<template>
  <section class="panel desktop-settings" aria-labelledby="overlay-settings-title">
    <div class="desktop-settings-heading"><PanelsTopLeft :size="23" aria-hidden="true"/><div><h2 id="overlay-settings-title">Компактный оверлей</h2><p>Системная громкость Windows поверх обычных окон. Яндекс Музыка через GSMTC; Discord через настроенные сочетания.</p></div></div>
    <button class="demo-action" :disabled="!desktop.native || !desktop.ready" @click="desktop.command('toggle_overlay')">Показать / скрыть оверлей</button>
    <form class="shortcut-form" @submit.prevent="saveShortcut"><label for="overlay-shortcut">Глобальная горячая клавиша</label><div><input id="overlay-shortcut" v-model="shortcut" type="text" autocomplete="off" :spellcheck="false" placeholder="Ctrl+D" aria-describedby="shortcut-help" :disabled="!desktop.native"/><button class="demo-action" type="submit" :disabled="!desktop.native || !desktop.ready || saving">{{ saving ? 'Сохранение…' : 'Применить' }}</button></div><p id="shortcut-help">Введите, например, Ctrl+D или Ctrl+Shift+D. Нужны Ctrl или Alt. При конфликте прежнее сочетание сохранится.</p></form>
    <label class="desktop-check"><input type="checkbox" :checked="desktop.preferences.pinned" :disabled="!desktop.native" @change="desktop.configure({ pinned: ($event.target as HTMLInputElement).checked })"/><span>Закрепить — не скрывать при потере фокуса</span></label>
    <label class="desktop-check"><input type="checkbox" :checked="desktop.preferences.autoHide" :disabled="!desktop.native || desktop.preferences.pinned" @change="desktop.configure({ autoHide: ($event.target as HTMLInputElement).checked })"/><span>Автоматически скрывать при потере фокуса</span></label>
    <p class="desktop-status" role="status">{{ !desktop.native ? 'Настройка окон и горячих клавиш доступна в приложении Windows.' : desktop.shortcutRegistered ? `Сейчас действует ${desktop.preferences.shortcut}.` : 'Горячая клавиша не зарегистрирована; оверлей доступен из трея.' }}</p>
    <p v-if="desktop.error || desktop.shortcutError || desktop.persistenceError" class="desktop-error" role="alert">{{ desktop.error || desktop.shortcutError || desktop.persistenceError }}</p>
    <p class="fine-print">Закрытие окна прячет LUNÉA в трей. Для завершения выберите «Выход» в меню трея. Работа поверх игр в эксклюзивном полноэкранном режиме не гарантируется.</p>
  </section>
</template>

