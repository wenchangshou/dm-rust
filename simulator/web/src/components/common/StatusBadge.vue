<template>
  <el-tag :class="['status-badge', `status-${status}`]" effect="plain">
    <span v-if="showIcon" :class="['status-dot', status]"></span>
    {{ statusText }}
  </el-tag>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { SimulatorStatus } from '@/types/simulator'

const props = withDefaults(defineProps<{
  status: SimulatorStatus
  showIcon?: boolean
}>(), {
  showIcon: true,
})

const statusText = computed(() => {
  switch (props.status) {
    case 'running':
      return '运行中'
    case 'stopped':
      return '已停止'
    case 'error':
      return '错误'
    default:
      return props.status
  }
})
</script>

<style lang="scss" scoped>
.status-badge {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 4px 12px;
  border-radius: 20px;
  font-weight: 500;
  font-size: 12px;
  border: none;

  &.status-running {
    background: rgba(var(--success-rgb), 0.15);
    color: var(--success);
  }

  &.status-stopped {
    background: rgba(100, 116, 139, 0.15);
    color: #94a3b8;
  }

  &.status-error {
    background: rgba(var(--danger-rgb), 0.15);
    color: var(--danger);
  }
}

.status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;

  &.running {
    background: var(--success);
    box-shadow: 0 0 8px rgba(var(--success-rgb), 0.6);
    animation: pulse 2s ease-in-out infinite;
  }

  &.stopped {
    background: #64748b;
  }

  &.error {
    background: var(--danger);
    box-shadow: 0 0 8px rgba(var(--danger-rgb), 0.6);
    animation: pulse 1s ease-in-out infinite;
  }
}

@keyframes pulse {

  0%,
  100% {
    opacity: 1;
    transform: scale(1);
  }

  50% {
    opacity: 0.6;
    transform: scale(0.9);
  }
}
</style>
