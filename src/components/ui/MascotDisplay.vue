<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { mascotPoses, type MascotState, type MascotSize } from '../../types/mascot'

const props = withDefaults(defineProps<{
  state: MascotState
  size?: MascotSize
  animated?: boolean
  alt?: string
}>(), { size: 'medium', animated: true })
const pose = computed(() => mascotPoses[props.state])
const failed = ref(false)
watch(() => props.state, () => { failed.value = false })
</script>

<template>
  <figure class="mascot-display" :class="`mascot-size-${size}`" :data-state="state">
    <Transition name="mascot-pose" mode="out-in" :css="animated">
      <img v-if="!failed" :key="state" :src="pose.src" :alt="alt ?? pose.alt"
        width="1346" height="1168" decoding="async" draggable="false" @error="failed = true" />
      <figcaption v-else class="mascot-load-error" role="status">Не удалось загрузить изображение Лунэи.</figcaption>
    </Transition>
  </figure>
</template>
