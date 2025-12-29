<template>
  <el-tag :type="tagType" effect="light">
    <el-icon v-if="showIcon" style="margin-right: 4px">
      <component :is="iconComponent" />
    </el-icon>
    {{ statusText }}
  </el-tag>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { VideoPlay, VideoPause, WarningFilled } from '@element-plus/icons-vue'
import type { SimulatorStatus } from '@/types/simulator'

const props = withDefaults(defineProps<{
  status: SimulatorStatus
  showIcon?: boolean
}>(), {
  showIcon: true,
})

const tagType = computed(() => {
  switch (props.status) {
    case 'running':
      return 'success'
    case 'stopped':
      return 'info'
    case 'error':
      return 'danger'
    default:
      return 'info'
  }
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

const iconComponent = computed(() => {
  switch (props.status) {
    case 'running':
      return VideoPlay
    case 'stopped':
      return VideoPause
    case 'error':
      return WarningFilled
    default:
      return VideoPause
  }
})
</script>
