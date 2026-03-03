<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps<{
  content: string
  parseError: string
}>()

const emit = defineEmits<{
  (e: 'update:content', value: string): void
  (e: 'format'): void
  (e: 'apply'): void
}>()

const statusType = computed(() => (props.parseError ? 'error' : 'success'))

const statusText = computed(() => {
  if (props.parseError) {
    return `JSON invalid: ${props.parseError}`
  }
  return 'JSON is valid.'
})
</script>

<template>
  <div class="raw-page">
    <el-card shadow="never" class="raw-card">
      <template #header>
        <div class="raw-header">
          <div>
            <h3>Edit Full Config JSON</h3>
            <p>Changes here are saved directly as the config payload.</p>
          </div>
          <div class="raw-actions">
            <el-button @click="emit('format')">Format JSON</el-button>
            <el-button type="primary" @click="emit('apply')">Apply to Visual Editors</el-button>
          </div>
        </div>
      </template>

      <el-input
        :model-value="content"
        type="textarea"
        :rows="22"
        class="raw-input"
        @update:model-value="(v: string) => emit('update:content', v)"
      />

      <el-alert
        :title="statusText"
        :type="statusType"
        :closable="false"
        show-icon
        class="raw-status"
      />
    </el-card>
  </div>
</template>

<style scoped>
.raw-page {
  display: grid;
  gap: 12px;
}

.raw-card {
  border: 1px solid rgba(15, 23, 42, 0.06);
}

.raw-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 12px;
}

.raw-header h3 {
  margin: 0;
  font-size: 16px;
}

.raw-header p {
  margin-top: 6px;
  font-size: 13px;
  color: var(--el-text-color-secondary);
}

.raw-actions {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.raw-input :deep(textarea) {
  font-family: Consolas, 'Courier New', monospace;
  font-size: 13px;
  line-height: 1.5;
}

.raw-status {
  margin-top: 12px;
}

@media (max-width: 960px) {
  .raw-header {
    flex-direction: column;
  }
}
</style>
