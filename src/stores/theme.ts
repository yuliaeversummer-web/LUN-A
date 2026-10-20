import { defineStore } from 'pinia'
import { ref, watch } from 'vue'
import { isTheme, type ThemeId } from '../types/theme'
import { readPreference, savePreference } from '../services/preferences'
export const useThemeStore = defineStore('theme', () => {
 const saved = readPreference('lunea.theme')
 const selected = ref<ThemeId>(isTheme(saved) ? saved : 'neon-pink')
 const persistenceError = ref(false)
 watch(selected, value => { document.documentElement.dataset.theme = value; persistenceError.value = !savePreference('lunea.theme', value) }, { immediate: true })
 window.addEventListener('storage', e => { if (e.key === 'lunea.theme' && isTheme(e.newValue)) selected.value = e.newValue })
 return { selected, persistenceError }
})
