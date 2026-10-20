<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { isTauri } from '@tauri-apps/api/core'
import { getCurrentWindow } from '@tauri-apps/api/window'
import type { UnlistenFn } from '@tauri-apps/api/event'
import { Minus, Square, Copy, X } from '@lucide/vue'
import BrandMark from '../ui/BrandMark.vue'

const nativeWindow = isTauri() ? getCurrentWindow() : null
const maximized = ref(false)
const busy = ref(false)
const error = ref('')
const maximizeLabel = computed(() => maximized.value ? 'Восстановить окно' : 'Развернуть окно')
let disposed = false
let unlistenResize: UnlistenFn | undefined
type ResizeDirection = Parameters<ReturnType<typeof getCurrentWindow>['startResizeDragging']>[0]
const resizeEdges: { direction: ResizeDirection; cursor: string }[] = [
  { direction: 'North', cursor: 'ns-resize' },
  { direction: 'South', cursor: 'ns-resize' },
  { direction: 'East', cursor: 'ew-resize' },
  { direction: 'West', cursor: 'ew-resize' },
  { direction: 'NorthEast', cursor: 'nesw-resize' },
  { direction: 'NorthWest', cursor: 'nwse-resize' },
  { direction: 'SouthEast', cursor: 'nwse-resize' },
  { direction: 'SouthWest', cursor: 'nesw-resize' },
]

function startResize(event: MouseEvent, direction: ResizeDirection) {
  if (!nativeWindow || maximized.value || event.button !== 0) return
  event.preventDefault()
  void nativeWindow.startResizeDragging(direction).catch(cause => {
    error.value = 'Не удалось изменить размер окна. Попробуйте ещё раз.'
    console.error('LUNÉA: window resize failed', cause)
  })
}

async function syncMaximized() {
  if (!nativeWindow || disposed) return
  const value = await nativeWindow.isMaximized()
  if (!disposed) maximized.value = value
}

async function run(action: () => Promise<void>) {
  if (!nativeWindow || busy.value) return
  busy.value = true
  error.value = ''
  try { await action() }
  catch (cause) {
    error.value = 'Не удалось управлять окном. Попробуйте ещё раз.'
    console.error('LUNÉA: window action failed', cause)
  } finally { busy.value = false }
}

function toggleMaximize() {
  return run(async () => { await nativeWindow!.toggleMaximize(); await syncMaximized() })
}

// Tauri's manual drag-region pattern: the second primary mousedown maximizes.
// The controls are siblings of this region and cannot start a drag.
function onDragMouseDown(event: MouseEvent) {
  if (!nativeWindow || event.button !== 0 || event.buttons !== 1) return
  event.preventDefault()
  if (event.detail === 2) void toggleMaximize()
  else {
    // Dragging must not lock the buttons: Windows may deliver the second
    // mousedown before the native move-loop promise has settled.
    void nativeWindow.startDragging().catch(cause => {
      error.value = 'Не удалось переместить окно. Попробуйте ещё раз.'
      console.error('LUNÉA: window drag failed', cause)
    })
  }
}

onMounted(async () => {
  if (!nativeWindow) return
  try {
    const stop = await nativeWindow.onResized(() => {
      void syncMaximized().catch(cause => {
        if (!disposed) { error.value = 'Не удалось обновить состояние окна.'; console.error(cause) }
      })
    })
    if (disposed) { stop(); return }
    unlistenResize = stop
    await syncMaximized()
  } catch (cause) {
    if (!disposed) { error.value = 'Не удалось получить состояние окна.'; console.error(cause) }
  }
})

onUnmounted(() => { disposed = true; unlistenResize?.() })
</script>

<template>
  <header class="titlebar" aria-label="Панель окна LUNÉA">
    <div class="titlebar-drag-region" @mousedown="onDragMouseDown" @contextmenu.prevent>
      <div class="titlebar-brand"><BrandMark /><strong>LUNÉA</strong><span>Audio Control</span></div>
      <span v-if="error" class="titlebar-error" role="status" :title="error">{{ error }}</span>
    </div>
    <div class="titlebar-controls" aria-label="Управление окном">
      <button type="button" class="window-button" aria-label="Свернуть окно"
        :title="nativeWindow ? 'Свернуть окно' : 'Доступно в приложении Windows'"
        :disabled="!nativeWindow || busy" @click="run(() => nativeWindow!.minimize())">
        <Minus :size="16" aria-hidden="true" />
      </button>
      <button type="button" class="window-button" :aria-label="maximizeLabel"
        :title="nativeWindow ? maximizeLabel : 'Доступно в приложении Windows'"
        :disabled="!nativeWindow || busy" @click="toggleMaximize">
        <Copy v-if="maximized" :size="14" aria-hidden="true" /><Square v-else :size="14" aria-hidden="true" />
      </button>
      <button type="button" class="window-button window-button-close" aria-label="Закрыть окно"
        :title="nativeWindow ? 'Закрыть окно' : 'Доступно в приложении Windows'"
        :disabled="!nativeWindow || busy" @click="run(() => nativeWindow!.close())">
        <X :size="17" aria-hidden="true" />
      </button>
    </div>
  </header>
  <template v-if="nativeWindow && !maximized">
    <div v-for="edge in resizeEdges" :key="edge.direction" aria-hidden="true"
      class="window-resize-edge" :class="`resize-${edge.direction}`" :style="{ cursor: edge.cursor }"
      @mousedown.stop="startResize($event, edge.direction)" />
  </template>
</template>
