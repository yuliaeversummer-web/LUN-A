<script setup lang="ts">
import { Mic, Headphones } from '@lucide/vue'
import { useDiscordStore } from '../../stores/discord'
withDefaults(defineProps<{compact?:boolean}>(),{compact:false})
const discord=useDiscordStore()
</script>
<template><div class="discord-actions" :class="{'discord-actions-compact':compact}">
 <button :disabled="!discord.canSend" aria-label="Переключить микрофон Discord" :title="discord.canSend?'Отправить настроенное Toggle Mute; состояние не подтверждается':discord.snapshot.message" @click="discord.toggle('mute')"><Mic :size="16" aria-hidden="true"/><span>{{ compact?'Микрофон':'Переключить микрофон' }}</span></button>
 <button :disabled="!discord.canSend" aria-label="Переключить звук Discord" :title="discord.canSend?'Отправить настроенное Toggle Deafen; состояние не подтверждается':discord.snapshot.message" @click="discord.toggle('deafen')"><Headphones :size="16" aria-hidden="true"/><span>{{ compact?'Звук':'Переключить звук' }}</span></button>
 <small>Состояние {{ discord.running?'неизвестно':'недоступно' }}</small>
</div></template>
