<script setup lang="ts">
import { computed,ref,watch,onMounted } from 'vue'
import { useRoute } from 'vue-router'
import { useDiscordStore } from '../../stores/discord'
import { useDesktopStore } from '../../stores/desktop'
import type { DiscordPreferences } from '../../types/discord'
const discord=useDiscordStore(),desktop=useDesktopStore()
const route=useRoute()
onMounted(()=>{if(route.hash==='#discord')document.getElementById('discord')?.scrollIntoView({block:'start'})})
const draft=ref<DiscordPreferences>({...discord.snapshot.preferences}),dirty=ref(false),touched=ref(false),saved=ref(false)
watch(()=>discord.snapshot.configurationRevision,()=>{if(!dirty.value)draft.value={...discord.snapshot.preferences}},{immediate:true})
function canonical(value:string){const parts=value.toUpperCase().replace(/\s/g,'').split('+');return parts.length===4&&new Set(parts.slice(0,3)).size===3&&parts.slice(0,3).every(p=>['CTRL','ALT','SHIFT'].includes(p))&&/^F([6-9]|1[0-1])$/.test(parts[3]??'')?`Ctrl+Alt+Shift+${parts[3]}`:null}
const problem=computed(()=>{
 for(const value of [draft.value.mute,draft.value.deafen])if(value&&!canonical(value))return 'Допустимы Ctrl+Alt+Shift и одна клавиша F6–F11.'
 if(draft.value.mute&&canonical(draft.value.mute)===canonical(draft.value.deafen))return 'Для Mute и Deafen нужны разные сочетания.'
 const overlay=desktop.preferences.shortcut.toUpperCase().replace(/\s/g,'').split('+').sort().join('+');if([draft.value.mute,draft.value.deafen].some(key=>key&&key.toUpperCase().replace(/\s/g,'').split('+').sort().join('+')===overlay))return 'Сочетание совпадает с клавишей оверлея.'
 if(draft.value.enabled&&(!draft.value.confirmed||!draft.value.mute||!draft.value.deafen))return 'Назначьте оба сочетания в Discord и подтвердите настройку.'
 return ''
})
function edit(){dirty.value=true;saved.value=false}
function bindingsChanged(){draft.value.confirmed=false;edit()}
async function save(){touched.value=true;if(problem.value)return;if(await discord.configure({...draft.value})){dirty.value=false;saved.value=true;draft.value={...discord.snapshot.preferences}}}
</script>
<template><section id="discord" class="panel desktop-settings discord-settings" aria-labelledby="discord-settings-title">
 <h2 id="discord-settings-title">Discord</h2><p role="status">{{ discord.snapshot.message }}</p>
 <div class="discord-client-list"><span v-for="client in discord.snapshot.clients" :key="client.variant">{{ client.variant.toUpperCase() }} · {{ client.running?'запущен':client.installed?'установлен, закрыт':'не найден' }}</span></div>
 <form @submit.prevent="save" @input="edit">
  <label class="discord-field">Клиент<select v-model="draft.client" @change="bindingsChanged"><option value="stable">Discord Stable</option><option value="ptb">Discord PTB</option><option value="canary">Discord Canary</option></select></label>
  <p class="fine-print">Режим: пользовательские горячие клавиши Windows. Состояния микрофона и Deafen неизвестны.</p>
  <div class="discord-hotkey-fields"><label class="discord-field">Toggle Mute<input v-model="draft.mute" maxlength="64" placeholder="Ctrl+Alt+Shift+F9" :aria-invalid="touched && !!problem" aria-describedby="discord-hotkey-help discord-validation" @input="bindingsChanged" @blur="touched=true"/></label><label class="discord-field">Toggle Deafen<input v-model="draft.deafen" maxlength="64" placeholder="Ctrl+Alt+Shift+F10" :aria-invalid="touched && !!problem" aria-describedby="discord-hotkey-help discord-validation" @input="bindingsChanged" @blur="touched=true"/></label></div>
  <p id="discord-hotkey-help" class="fine-print">Разрешены Ctrl+Alt+Shift+F6–F11. Используйте разные сочетания, не назначенные другим приложениям. Win, текстовые последовательности и системные команды запрещены. LUNÉA не регистрирует эти клавиши вместо Discord.</p>
  <label class="desktop-check"><input v-model="draft.confirmed" type="checkbox"/><span>Я сам назначил эти сочетания именно в выбранном Discord</span></label>
  <label class="desktop-check"><input v-model="draft.enabled" type="checkbox"/><span>Разрешить отправку настроенных сочетаний</span></label>
  <p id="discord-validation" class="discord-feedback" role="status">{{ touched?problem:'' }}</p>
  <div class="discord-settings-actions"><button class="demo-action" type="submit" :disabled="discord.busy || !desktop.native">Сохранить Discord</button><button class="demo-action" type="button" :disabled="discord.busy || !desktop.native" @click="discord.refresh">Проверить Discord</button></div>
  <p v-if="saved" role="status" class="fine-print">Настройки сохранены.</p>
 </form>
 <p v-if="discord.error || discord.snapshot.persistenceError" class="discord-feedback" role="alert">{{ discord.error || discord.snapshot.persistenceError }}</p>
 <p v-if="discord.snapshot.lastAttempt" class="discord-feedback" role="status">Последняя попытка: {{ discord.snapshot.lastAttempt.message }}</p>
 <details class="discord-help"><summary>Как настроить горячие клавиши Discord</summary><ol><li>Открой Discord → «Настройки пользователя» → «Горячие клавиши». В новых версиях: «Система» → «Пользовательские горячие клавиши».</li><li>Создай «Переключить микрофон» / «Вкл./выкл. микрофон» (Toggle Mute) и назначь, например, Ctrl+Alt+Shift+F9.</li><li>Создай «Переключить звук» / «Вкл./выкл. звук в динамиках» (Toggle Deafen), например Ctrl+Alt+Shift+F10.</li><li>Закрой раздел настройки клавиш Discord: пока он открыт, сочетания отключены. Введи те же сочетания выше, подтверди настройку и разреши отправку.</li></ol><p>Названия пунктов зависят от версии и языка. Клавиши назначаются пользователем. Windows/UIPI, игры и другие приложения могут блокировать ввод. Другие программы с тем же сочетанием тоже могут на него реагировать. Несколько одновременно запущенных вариантов Discord блокируют отправку.</p></details>
</section></template>
