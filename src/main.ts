import { createApp } from 'vue'
import { createPinia } from 'pinia'
import '@fontsource-variable/manrope'
import './styles/main.css'
import './styles/window.css'
import './styles/mascot.css'
import './styles/overlay.css'
import './styles/audio.css'
import './styles/media.css'
import './styles/discord.css'
import App from './App.vue'
import { router } from './app/router'
import { initializeDesktopSync } from './services/desktopSync'
import { useAudioStore } from './stores/audio'
import { useMediaStore } from './stores/media'
import { useDiscordStore } from './stores/discord'
const pinia = createPinia()
router.afterEach(to => { document.documentElement.dataset.window = to.path === '/overlay' ? 'overlay' : 'main' })
document.documentElement.dataset.window = location.hash.startsWith('#/overlay') ? 'overlay' : 'main'
createApp(App).use(pinia).use(router).mount('#app')
void initializeDesktopSync(pinia)
void useAudioStore(pinia).initialize()
void useMediaStore(pinia).initialize()
void useDiscordStore(pinia).initialize()
