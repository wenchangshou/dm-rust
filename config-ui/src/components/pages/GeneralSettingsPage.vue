<script setup lang="ts">
import { useI18n } from '../../composables/useI18n'
import type { DatabaseConfig, FileConfig, ResourceConfig, WebServerConfig } from '../../types/config'

const props = defineProps<{
  webServer: WebServerConfig
  fileConfig: FileConfig
  databaseConfig: DatabaseConfig
  resourceConfig: ResourceConfig
}>()

const emit = defineEmits<{
  (e: 'update:webServer', value: WebServerConfig): void
  (e: 'update:fileConfig', value: FileConfig): void
  (e: 'update:databaseConfig', value: DatabaseConfig): void
  (e: 'update:resourceConfig', value: ResourceConfig): void
}>()

const { t } = useI18n()

/* ---- helpers ---- */
const updateWebServer = (field: keyof WebServerConfig, val: unknown) => {
  emit('update:webServer', { ...props.webServer, [field]: val })
}

const updateFile = (field: keyof FileConfig, val: unknown) => {
  emit('update:fileConfig', { ...props.fileConfig, [field]: val })
}

const updateDatabase = (field: keyof DatabaseConfig, val: unknown) => {
  emit('update:databaseConfig', { ...props.databaseConfig, [field]: val })
}

const updateResource = (field: keyof ResourceConfig, val: unknown) => {
  emit('update:resourceConfig', { ...props.resourceConfig, [field]: val })
}

const normalizePort = (val: number | undefined) => {
  const port = Number(val)
  if (!Number.isFinite(port) || port < 1 || port > 65535) {
    updateWebServer('port', 18080)
    return
  }
  updateWebServer('port', Math.round(port))
}

const normalizeBodyLimit = (val: number | undefined) => {
  const limit = Number(val)
  if (!Number.isFinite(limit) || limit < 0) {
    updateWebServer('max_body_limit', 104857600)
    return
  }
  updateWebServer('max_body_limit', Math.round(limit))
}

const formatBytes = (bytes: number | undefined) => {
  if (!bytes) return '0 B'
  const units = ['B', 'KB', 'MB', 'GB']
  let i = 0
  let size = bytes
  while (size >= 1024 && i < units.length - 1) {
    size /= 1024
    i++
  }
  return `${size.toFixed(1)} ${units[i]}`
}
</script>

<template>
  <div class="settings-page">
    <!-- Web Server -->
    <el-card shadow="never" class="section-card">
      <template #header>
        <div class="section-header">
          <el-icon size="18"><Monitor /></el-icon>
          <span class="section-title">{{ t('settings.webServer') }}</span>
        </div>
      </template>
      <el-form label-position="left" label-width="180px">
        <el-form-item :label="t('settings.webPort')">
          <el-input-number
            :model-value="webServer.port"
            :min="1"
            :max="65535"
            controls-position="right"
            @change="normalizePort"
          />
        </el-form-item>
        <el-form-item :label="t('settings.maxBodyLimit')">
          <div class="body-limit-row">
            <el-input-number
              :model-value="webServer.max_body_limit ?? 104857600"
              :min="1048576"
              :step="1048576"
              controls-position="right"
              @change="normalizeBodyLimit"
            />
            <el-tag type="info" effect="plain" class="size-hint">
              {{ formatBytes(webServer.max_body_limit ?? 104857600) }}
            </el-tag>
          </div>
        </el-form-item>
      </el-form>
    </el-card>

    <!-- File Management -->
    <el-card shadow="never" class="section-card">
      <template #header>
        <div class="section-header">
          <el-icon size="18"><FolderOpened /></el-icon>
          <span class="section-title">{{ t('settings.fileManagement') }}</span>
          <el-switch
            class="header-switch"
            :model-value="fileConfig.enable"
            :active-text="t('common.enabled')"
            :inactive-text="t('common.disabled')"
            @change="(v: boolean) => updateFile('enable', v)"
          />
        </div>
      </template>
      <el-form label-position="left" label-width="180px">
        <el-form-item :label="t('settings.filePath')">
          <el-input
            :model-value="fileConfig.path"
            :placeholder="t('settings.filePathPlaceholder')"
            :disabled="!fileConfig.enable"
            @input="(v: string) => updateFile('path', v)"
          />
        </el-form-item>
      </el-form>
    </el-card>

    <!-- Database -->
    <el-card shadow="never" class="section-card">
      <template #header>
        <div class="section-header">
          <el-icon size="18"><Coin /></el-icon>
          <span class="section-title">{{ t('settings.database') }}</span>
          <el-switch
            class="header-switch"
            :model-value="databaseConfig.enable"
            :active-text="t('common.enabled')"
            :inactive-text="t('common.disabled')"
            @change="(v: boolean) => updateDatabase('enable', v)"
          />
        </div>
      </template>
      <el-form label-position="left" label-width="180px">
        <el-form-item :label="t('settings.databaseUrl')">
          <el-input
            :model-value="databaseConfig.url"
            :placeholder="t('settings.databaseUrlPlaceholder')"
            :disabled="!databaseConfig.enable"
            @input="(v: string) => updateDatabase('url', v)"
          />
        </el-form-item>
      </el-form>
    </el-card>

    <!-- Resource Management -->
    <el-card shadow="never" class="section-card">
      <template #header>
        <div class="section-header">
          <el-icon size="18"><Picture /></el-icon>
          <span class="section-title">{{ t('settings.resourceManagement') }}</span>
          <el-switch
            class="header-switch"
            :model-value="resourceConfig.enable"
            :active-text="t('common.enabled')"
            :inactive-text="t('common.disabled')"
            @change="(v: boolean) => updateResource('enable', v)"
          />
        </div>
      </template>
      <el-form label-position="left" label-width="180px">
        <el-form-item :label="t('settings.resourcePath')">
          <el-input
            :model-value="resourceConfig.path"
            :placeholder="t('settings.resourcePathPlaceholder')"
            :disabled="!resourceConfig.enable"
            @input="(v: string) => updateResource('path', v)"
          />
        </el-form-item>
        <el-form-item :label="t('settings.urlPrefix')">
          <el-input
            :model-value="resourceConfig.url_prefix"
            :placeholder="t('settings.urlPrefixPlaceholder')"
            :disabled="!resourceConfig.enable"
            @input="(v: string) => updateResource('url_prefix', v)"
          />
        </el-form-item>
      </el-form>
    </el-card>
  </div>
</template>

<style scoped>
.settings-page {
  display: grid;
  gap: 16px;
}

.section-card {
  border: 1px solid rgba(15, 23, 42, 0.06);
}

.section-header {
  display: flex;
  align-items: center;
  gap: 8px;
}

.section-title {
  font-weight: 600;
  font-size: 15px;
}

.header-switch {
  margin-left: auto;
}

.body-limit-row {
  display: flex;
  align-items: center;
  gap: 12px;
}

.size-hint {
  font-family: monospace;
}
</style>
