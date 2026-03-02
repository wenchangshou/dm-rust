<script setup lang="ts">
import { useI18n } from '../../composables/useI18n'

const props = defineProps<{
  channelsCount: number
  nodesCount: number
  scenesCount: number
  protocolCount: number
  webPort: number
}>()

const emit = defineEmits<{
  (e: 'update:webPort', value: number): void
}>()

const { t } = useI18n()

const normalizePort = (val: number | undefined) => {
  const port = Number(val)
  if (!Number.isFinite(port) || port < 1 || port > 65535) {
    emit('update:webPort', 18080)
    return
  }
  emit('update:webPort', Math.round(port))
}
</script>

<template>
  <div class="overview-page">
    <div class="kpi-grid">
      <el-card shadow="never" class="kpi-card">
        <el-statistic :value="channelsCount" :title="t('overview.channels')" />
      </el-card>
      <el-card shadow="never" class="kpi-card">
        <el-statistic :value="nodesCount" :title="t('overview.nodes')" />
      </el-card>
      <el-card shadow="never" class="kpi-card">
        <el-statistic :value="scenesCount" :title="t('overview.scenes')" />
      </el-card>
      <el-card shadow="never" class="kpi-card">
        <el-statistic :value="protocolCount" :title="t('overview.protocols')" />
      </el-card>
    </div>

    <el-card shadow="never" class="settings-card">
      <template #header>
        <span class="settings-title">{{ t('overview.systemSettings') }}</span>
      </template>
      <el-form label-position="left" label-width="140px">
        <el-form-item :label="t('overview.webPort')">
          <el-input-number
            :model-value="webPort"
            :min="1"
            :max="65535"
            controls-position="right"
            @change="normalizePort"
          />
        </el-form-item>
      </el-form>
    </el-card>
  </div>
</template>

<style scoped>
.overview-page {
  display: grid;
  gap: 16px;
}

.kpi-grid {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 12px;
}

.kpi-card {
  border: 1px solid rgba(15, 23, 42, 0.06);
}

.settings-card {
  border: 1px solid rgba(15, 23, 42, 0.06);
}

.settings-title {
  font-weight: 600;
  font-size: 15px;
}

@media (max-width: 1200px) {
  .kpi-grid {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}

@media (max-width: 960px) {
  .kpi-grid {
    grid-template-columns: 1fr;
  }
}
</style>
