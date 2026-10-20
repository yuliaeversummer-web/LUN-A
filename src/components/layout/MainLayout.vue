<script setup lang="ts">
import { provide } from 'vue'
import { LayoutDashboard, Settings2, AudioLines, Moon, ArrowUpRight } from '@lucide/vue'
import BrandMark from '../ui/BrandMark.vue'
import CustomTitleBar from './CustomTitleBar.vue'
import { createMascotReactions, mascotKey } from '../../composables/useMascotReactions'
import { useAudioStore } from '../../stores/audio'
import { useDiscordStore } from '../../stores/discord'
const audio = useAudioStore(), discord = useDiscordStore()
const mascot = createMascotReactions()
provide(mascotKey, mascot)
</script>
<template><div class="main-window" @pointerdown.capture="mascot.markActivity" @keydown.capture="mascot.markActivity" @input.capture="mascot.markActivity" @wheel.passive="mascot.markActivity"><CustomTitleBar/><div class="app-shell"><aside class="sidebar">
<div class="brand"><BrandMark/><div><strong>LUNÉA</strong><small>AUDIO CONTROL</small></div></div>
<p class="nav-label">ТВОЁ ПРОСТРАНСТВО</p><nav aria-label="Главная навигация">
<RouterLink to="/" exact-active-class="selected"><LayoutDashboard :size="19"/>Обзор</RouterLink>
<RouterLink to="/settings" active-class="selected"><Settings2 :size="19"/>Настройки</RouterLink></nav>
<div class="sidebar-note"><AudioLines :size="22"/><p>Всё звучит.<br/><span>В твоём ритме.</span></p></div>
<div class="sidebar-bottom"><Moon :size="15"/><span>Локально. Только для тебя.</span><small>ЭТАП 7 · v0.1.0</small></div></aside>
<div class="workspace"><header class="topbar"><span>LUNÉA <span class="slash">/</span> {{ $route.path === '/settings' ? 'Настройки' : 'Обзор' }}</span><span class="demo-badge">Windows Audio <ArrowUpRight :size="13"/></span></header>
<div class="content-scroll" tabindex="0" role="region" aria-label="Содержимое страницы"><main id="main-content"><RouterView/></main></div><footer><span class="status-dot"/>{{ audio.ready ? 'Windows Audio подключён' : 'Windows Audio: ' + (audio.snapshot.status === 'loading' ? 'подключение…' : 'недоступен') }}<span class="footer-right">Discord · {{ discord.running ? 'горячие клавиши' : 'недоступен' }}</span></footer></div></div></div></template>


