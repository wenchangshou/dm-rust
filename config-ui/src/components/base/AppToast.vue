<script setup lang="ts">
import type { ToastType } from '../../types/config'

defineProps<{
  message: string
  type: ToastType
}>()
</script>

<template>
  <Transition name="toast">
    <div v-if="message" :class="['app-toast', type]">
      {{ message }}
    </div>
  </Transition>
</template>

<style scoped>
.app-toast {
  position: fixed;
  top: 24px;
  right: 24px;
  min-width: 280px;
  max-width: 460px;
  padding: 10px 14px;
  border-radius: 12px;
  font-size: 13px;
  border: 1px solid transparent;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.12);
  z-index: 1200;
}

.app-toast.success {
  color: #14532d;
  background: #dcfce7;
  border-color: #bbf7d0;
}

.app-toast.error {
  color: #991b1b;
  background: #fef2f2;
  border-color: #fecaca;
}

.toast-enter-active,
.toast-leave-active {
  transition: all 0.2s ease;
}

.toast-enter-from,
.toast-leave-to {
  opacity: 0;
  transform: translateY(-8px);
}
</style>
