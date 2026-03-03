<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { ElMessage } from 'element-plus'
import zhCn from 'element-plus/es/locale/lang/zh-cn'
import enUs from 'element-plus/es/locale/lang/en'
import AppSidebar from './components/base/AppSidebar.vue'
import OverviewPage from './components/pages/OverviewPage.vue'
import ChannelsPage from './components/pages/ChannelsPage.vue'
import NodesPage from './components/pages/NodesPage.vue'
import ScenesPage from './components/pages/ScenesPage.vue'
import GeneralSettingsPage from './components/pages/GeneralSettingsPage.vue'
import RawConfigPage from './components/pages/RawConfigPage.vue'
import { useConfigSystem } from './composables/useConfigSystem'
import { useI18n } from './composables/useI18n'
import { fetchDeviceConfig, reloadDeviceRuntime, saveDeviceConfig } from './services/configApi'
import { useSchemaRegistry } from './services/schemaRegistry'
import type {
  Channel,
  DatabaseConfig,
  DeviceConfig,
  FileConfig,
  NodeItem,
  PageKey,
  ResourceConfig,
  Scene,
  ToastType,
  WebServerConfig
} from './types/config'
import { logger } from './utils/logger'

const activePage = ref<PageKey>('overview')
const lastSyncText = ref('-')
const rawConfigText = ref('{}')

const { t, locale } = useI18n()

const {
  channels,
  nodes,
  scenes,
  webServer,
  fileConfig,
  databaseConfig,
  resourceConfig,
  loading,
  saving,
  stats,
  setConfig,
  toPayload
} = useConfigSystem()

const { protocolList, initProtocols, ensureSchema, resolveSchema } = useSchemaRegistry()

const elementLocale = computed(() => (locale.value === 'en-US' ? enUs : zhCn))

const pageMeta = computed(() => {
  if (activePage.value === 'overview') {
    return { title: t('overview.title'), desc: t('overview.desc') }
  }
  if (activePage.value === 'channels') {
    return { title: t('sidebar.channels'), desc: t('channels.desc') }
  }
  if (activePage.value === 'nodes') {
    return { title: t('sidebar.nodes'), desc: t('nodes.desc') }
  }
  if (activePage.value === 'scenes') {
    return { title: t('sidebar.scenes'), desc: t('scenes.desc') }
  }
  if (activePage.value === 'settings') {
    return { title: t('settings.title'), desc: t('settings.desc') }
  }

  return {
    title: 'Raw JSON Editor',
    desc: 'Directly edit the full configuration JSON and save it as-is.'
  }
})

const rawConfigRootErrorText = computed(() => 'Root JSON must be an object.')

const parseRawConfig = () => {
  try {
    const parsed = JSON.parse(rawConfigText.value) as unknown
    if (!parsed || typeof parsed !== 'object' || Array.isArray(parsed)) {
      return { data: null, error: rawConfigRootErrorText.value }
    }
    return { data: parsed as Record<string, unknown>, error: null }
  } catch (error) {
    const text = error instanceof Error ? error.message : String(error)
    return { data: null, error: text }
  }
}

const rawConfigError = computed(() => {
  const parsed = parseRawConfig()
  return parsed.error ?? ''
})

const refreshLastSync = () => {
  lastSyncText.value = new Date().toLocaleString(locale.value)
}

const notify = (message: string, type: ToastType = 'success') => {
  ElMessage({
    type: type === 'error' ? 'error' : 'success',
    message,
    duration: 3200,
    showClose: true
  })
}

const onNotify = (payload: { message: string; type?: ToastType }) => {
  notify(payload.message, payload.type ?? 'success')
}

const preloadChannelSchemas = async () => {
  const statutes = [...new Set(channels.value.map((channel) => channel.statute).filter(Boolean))]
  logger.info('app', 'preload channel schemas start', { statutes })

  for (const statute of statutes) {
    try {
      await ensureSchema(statute)
    } catch {
      logger.warn('app', 'preload channel schema failed', { statute })
    }
  }
  logger.info('app', 'preload channel schemas done', { count: statutes.length })
}

const syncRawFromStructured = () => {
  rawConfigText.value = JSON.stringify(toPayload(), null, 2)
}

const applyRawConfig = async () => {
  const parsed = parseRawConfig()
  if (parsed.error || !parsed.data) {
    notify(t('toast.validationError', { message: parsed.error || '-' }), 'error')
    return
  }

  setConfig(parsed.data as Partial<DeviceConfig>)
  await preloadChannelSchemas()
  notify('Raw JSON applied to visual editors.')
}

const formatRawConfig = () => {
  const parsed = parseRawConfig()
  if (parsed.error || !parsed.data) {
    notify(t('toast.validationError', { message: parsed.error || '-' }), 'error')
    return
  }
  rawConfigText.value = JSON.stringify(parsed.data, null, 2)
}

const reloadRuntime = async () => {
  try {
    const response = await reloadDeviceRuntime()
    if (response.state === 0) {
      if (response.message) {
        notify(response.message)
      }
      return true
    }

    notify(t('toast.loadFailed', { message: response.message || '-' }), 'error')
    return false
  } catch (error) {
    const text = error instanceof Error ? error.message : String(error)
    notify(t('toast.connectionError', { message: text }), 'error')
    return false
  }
}

const loadConfig = async (hotReload = false) => {
  loading.value = true
  logger.info('app', 'load config start', { hotReload })

  try {
    if (hotReload) {
      const ok = await reloadRuntime()
      if (!ok) {
        return
      }
    }

    const response = await fetchDeviceConfig()
    if (response.state === 0 && response.data) {
      setConfig(response.data)
      rawConfigText.value = JSON.stringify(response.data, null, 2)
      logger.info('app', 'load config apply success', {
        channels: channels.value.length,
        nodes: nodes.value.length,
        scenes: scenes.value.length
      })
      await preloadChannelSchemas()
      refreshLastSync()
      notify(t('toast.loaded'))
      return
    }

    logger.warn('app', 'load config failed by state', {
      state: response.state,
      message: response.message
    })
    notify(t('toast.loadFailed', { message: response.message || '-' }), 'error')
  } catch (error) {
    const text = error instanceof Error ? error.message : String(error)
    logger.error('app', 'load config exception', { error: text })
    notify(t('toast.connectionError', { message: text }), 'error')
  } finally {
    loading.value = false
    logger.info('app', 'load config end')
  }
}

const saveRawConfig = async () => {
  const parsed = parseRawConfig()
  if (parsed.error || !parsed.data) {
    notify(t('toast.validationError', { message: parsed.error || '-' }), 'error')
    return
  }

  saving.value = true
  logger.info('app', 'save raw config start')

  try {
    const response = await saveDeviceConfig(parsed.data as unknown as DeviceConfig)
    if (response.state === 0) {
      const reloadOk = await reloadRuntime()
      if (!reloadOk) {
        notify(t('toast.saveFailed', { message: 'saved, but hot reload failed' }), 'error')
        return
      }

      await loadConfig()
      logger.info('app', 'save raw config success')
      refreshLastSync()
      notify(t('toast.saved'))
      return
    }

    logger.warn('app', 'save raw config failed by state', {
      state: response.state,
      message: response.message
    })
    notify(t('toast.saveFailed', { message: response.message || '-' }), 'error')
  } catch (error) {
    const text = error instanceof Error ? error.message : String(error)
    logger.error('app', 'save raw config exception', { error: text })
    notify(t('toast.connectionError', { message: text }), 'error')
  } finally {
    saving.value = false
    logger.info('app', 'save raw config end')
  }
}

const saveConfig = async () => {
  if (activePage.value === 'raw') {
    await saveRawConfig()
    return
  }

  saving.value = true
  logger.info('app', 'save config start')

  try {
    const response = await saveDeviceConfig(toPayload())
    if (response.state === 0) {
      const reloadOk = await reloadRuntime()
      if (!reloadOk) {
        notify(t('toast.saveFailed', { message: 'saved, but hot reload failed' }), 'error')
        return
      }

      await loadConfig()
      logger.info('app', 'save config success')
      syncRawFromStructured()
      refreshLastSync()
      notify(t('toast.saved'))
      return
    }

    logger.warn('app', 'save config failed by state', {
      state: response.state,
      message: response.message
    })
    notify(t('toast.saveFailed', { message: response.message || '-' }), 'error')
  } catch (error) {
    const text = error instanceof Error ? error.message : String(error)
    logger.error('app', 'save config exception', { error: text })
    notify(t('toast.connectionError', { message: text }), 'error')
  } finally {
    saving.value = false
    logger.info('app', 'save config end')
  }
}

const updateChannels = (value: Channel[]) => {
  logger.info('app', 'update channels', { count: value.length })
  channels.value = value
  syncRawFromStructured()
  void preloadChannelSchemas()
}

const updateNodes = (value: NodeItem[]) => {
  nodes.value = value
  syncRawFromStructured()
}

const updateScenes = (value: Scene[]) => {
  scenes.value = value
  syncRawFromStructured()
}

const updateWebPort = (value: number) => {
  webServer.value.port = value
  syncRawFromStructured()
}

const updateWebServer = (value: WebServerConfig) => {
  webServer.value = value
  syncRawFromStructured()
}

const updateFileConfig = (value: FileConfig) => {
  fileConfig.value = value
  syncRawFromStructured()
}

const updateDatabaseConfig = (value: DatabaseConfig) => {
  databaseConfig.value = value
  syncRawFromStructured()
}

const updateResourceConfig = (value: ResourceConfig) => {
  resourceConfig.value = value
  syncRawFromStructured()
}

const updateRawConfigText = (value: string) => {
  rawConfigText.value = value
}

onMounted(async () => {
  logger.info('app', 'mounted')
  try {
    await initProtocols()
  } catch (error) {
    const text = error instanceof Error ? error.message : String(error)
    logger.error('app', 'init protocols exception', { error: text })
    notify(t('toast.connectionError', { message: text }), 'error')
  }

  await loadConfig()
})
</script>

<template>
  <el-config-provider :locale="elementLocale">
    <div class="app-shell">
      <el-container class="layout-container">
        <el-aside class="sidebar-wrapper">
          <AppSidebar
            :active-page="activePage"
            :channels-count="stats.channels"
            :nodes-count="stats.nodes"
            :scenes-count="stats.scenes"
            :saving="saving"
            :loading="loading"
            @change-page="activePage = $event"
            @save="saveConfig"
            @reload="() => loadConfig(true)"
          />
        </el-aside>

        <el-container>
          <el-header class="top-wrapper">
            <el-card shadow="never" class="top-card">
              <div class="top-row">
                <div class="title-block">
                  <h1>{{ t('app.title') }}</h1>
                  <p>{{ t('app.subtitle') }}</p>
                </div>

                <div class="top-actions">
                  <el-tag effect="light" type="success">{{ t('common.online') }}</el-tag>
                  <el-tag effect="plain">{{ t('app.lastSync', { time: lastSyncText }) }}</el-tag>
                  <el-button :loading="loading" @click="loadConfig(true)">{{ t('common.reload') }}</el-button>
                  <el-button type="primary" :loading="saving" @click="saveConfig">{{ t('common.save') }}</el-button>
                </div>
              </div>

              <div class="module-row">
                <el-breadcrumb separator=">">
                  <el-breadcrumb-item>{{ t('app.title') }}</el-breadcrumb-item>
                  <el-breadcrumb-item>{{ t('app.currentModule') }}</el-breadcrumb-item>
                  <el-breadcrumb-item>{{ pageMeta.title }}</el-breadcrumb-item>
                </el-breadcrumb>
              </div>
            </el-card>
          </el-header>

          <el-main class="main-content">
            <section class="page-panel">
              <header class="page-panel-header">
                <h2>{{ pageMeta.title }}</h2>
                <p>{{ pageMeta.desc }}</p>
              </header>

              <OverviewPage
                v-if="activePage === 'overview'"
                :channels-count="stats.channels"
                :nodes-count="stats.nodes"
                :scenes-count="stats.scenes"
                :protocol-count="protocolList.length"
                :web-port="webServer.port"
                @update:web-port="updateWebPort"
              />
              <ChannelsPage
                v-if="activePage === 'channels'"
                :channels="channels"
                :protocol-list="protocolList"
                :ensure-schema="ensureSchema"
                :resolve-schema="resolveSchema"
                @update:channels="updateChannels"
                @notify="onNotify"
              />

              <NodesPage
                v-if="activePage === 'nodes'"
                :nodes="nodes"
                :channels="channels"
                @update:nodes="updateNodes"
                @notify="onNotify"
              />

              <ScenesPage
                v-if="activePage === 'scenes'"
                :scenes="scenes"
                :nodes="nodes"
                @update:scenes="updateScenes"
                @notify="onNotify"
              />

              <GeneralSettingsPage
                v-if="activePage === 'settings'"
                :web-server="webServer"
                :file-config="fileConfig"
                :database-config="databaseConfig"
                :resource-config="resourceConfig"
                @update:web-server="updateWebServer"
                @update:file-config="updateFileConfig"
                @update:database-config="updateDatabaseConfig"
                @update:resource-config="updateResourceConfig"
              />

              <RawConfigPage
                v-if="activePage === 'raw'"
                :content="rawConfigText"
                :parse-error="rawConfigError"
                @update:content="updateRawConfigText"
                @format="formatRawConfig"
                @apply="applyRawConfig"
              />
            </section>
          </el-main>
        </el-container>
      </el-container>
    </div>
  </el-config-provider>
</template>

<style scoped>
.app-shell {
  min-height: 100vh;
  background:
    radial-gradient(circle at 20% 10%, rgba(37, 99, 235, 0.08), transparent 35%),
    radial-gradient(circle at 85% 4%, rgba(14, 165, 233, 0.12), transparent 32%),
    #f2f5fb;
}

.layout-container {
  min-height: 100vh;
}

.sidebar-wrapper {
  width: 300px;
}

.top-wrapper {
  height: auto;
  padding: 14px 16px 0;
  display: grid;
  gap: 12px;
}

.top-card {
  border: 1px solid rgba(15, 23, 42, 0.08);
}

.top-row {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 12px;
}

.title-block h1 {
  margin: 0;
  font-size: 22px;
  font-weight: 700;
  letter-spacing: 0.02em;
}

.title-block p {
  margin-top: 6px;
  color: var(--el-text-color-secondary);
  font-size: 13px;
}

.top-actions {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
  justify-content: flex-end;
}

.module-row {
  margin-top: 12px;
}

.kpi-grid {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 12px;
}

.kpi-card {
  border: 1px solid rgba(15, 23, 42, 0.06);
}

.protocol-card {
  display: grid;
  gap: 8px;
}

.port-editor {
  display: grid;
  gap: 6px;
}

.port-editor span {
  font-size: 12px;
  color: var(--el-text-color-secondary);
}

.main-content {
  padding: 16px;
}

.page-panel {
  background: #ffffff;
  border: 1px solid rgba(15, 23, 42, 0.08);
  border-radius: 14px;
  padding: 14px;
  display: grid;
  gap: 12px;
}

.page-panel-header h2 {
  margin: 0;
  font-size: 18px;
}

.page-panel-header p {
  margin-top: 6px;
  color: var(--el-text-color-secondary);
  font-size: 13px;
}

@media (max-width: 1200px) {
  .kpi-grid {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}

@media (max-width: 960px) {
  .layout-container {
    flex-direction: column;
  }

  .sidebar-wrapper {
    width: 100%;
  }

  .top-row {
    flex-direction: column;
    align-items: stretch;
  }

  .top-actions {
    justify-content: flex-start;
  }

  .kpi-grid {
    grid-template-columns: 1fr;
  }
}
</style>
