<script setup lang="ts">
import { onMounted, ref } from 'vue'
import AppSidebar from './components/base/AppSidebar.vue'
import AppToast from './components/base/AppToast.vue'
import ChannelsPage from './components/pages/ChannelsPage.vue'
import NodesPage from './components/pages/NodesPage.vue'
import ScenesPage from './components/pages/ScenesPage.vue'
import { useConfigSystem } from './composables/useConfigSystem'
import { useI18n } from './composables/useI18n'
import { useToast } from './composables/useToast'
import { fetchDeviceConfig, saveDeviceConfig } from './services/configApi'
import { useSchemaRegistry } from './services/schemaRegistry'
import type { Channel, NodeItem, PageKey, Scene, ToastType } from './types/config'

const activePage = ref<PageKey>('channels')

const { t } = useI18n()
const { message, type, show } = useToast()

const {
  channels,
  nodes,
  scenes,
  webServer,
  loading,
  saving,
  stats,
  setConfig,
  toPayload
} = useConfigSystem()

const { schemas, protocolList } = useSchemaRegistry()

const onNotify = (payload: { message: string; type?: ToastType }) => {
  show(payload.message, payload.type ?? 'success')
}

const normalizePort = () => {
  const port = Number(webServer.value.port)
  if (!Number.isFinite(port) || port < 1 || port > 65535) {
    webServer.value.port = 18080
    show(t('toast.validationError', { message: `${t('overview.webPort')} 1-65535` }), 'error')
    return
  }

  webServer.value.port = Math.round(port)
}

const loadConfig = async () => {
  loading.value = true

  try {
    const response = await fetchDeviceConfig()
    if (response.state === 0 && response.data) {
      setConfig(response.data)
      show(t('toast.loaded'), 'success')
      return
    }

    show(t('toast.loadFailed', { message: response.message || '-' }), 'error')
  } catch (error) {
    const text = error instanceof Error ? error.message : String(error)
    show(t('toast.connectionError', { message: text }), 'error')
  } finally {
    loading.value = false
  }
}

const saveConfig = async () => {
  saving.value = true

  try {
    const response = await saveDeviceConfig(toPayload())
    if (response.state === 0) {
      show(t('toast.saved'), 'success')
      return
    }

    show(t('toast.saveFailed', { message: response.message || '-' }), 'error')
  } catch (error) {
    const text = error instanceof Error ? error.message : String(error)
    show(t('toast.connectionError', { message: text }), 'error')
  } finally {
    saving.value = false
  }
}

const updateChannels = (value: Channel[]) => {
  channels.value = value
}

const updateNodes = (value: NodeItem[]) => {
  nodes.value = value
}

const updateScenes = (value: Scene[]) => {
  scenes.value = value
}

onMounted(() => {
  void loadConfig()
})
</script>

<template>
  <div class="app-shell">
    <AppToast :message="message" :type="type" />

    <AppSidebar
      :active-page="activePage"
      :channels-count="stats.channels"
      :nodes-count="stats.nodes"
      :scenes-count="stats.scenes"
      :saving="saving"
      :loading="loading"
      @change-page="activePage = $event"
      @save="saveConfig"
      @reload="loadConfig"
    />

    <main class="main-content">
      <section class="overview-card">
        <div>
          <h2>{{ t('overview.title') }}</h2>
          <p>{{ t('overview.desc') }}</p>
        </div>
        <div class="overview-right">
          <label>
            <span>{{ t('overview.webPort') }}</span>
            <input v-model.number="webServer.port" type="number" min="1" max="65535" @change="normalizePort" />
          </label>
          <div class="stats-grid">
            <article>
              <strong>{{ stats.channels }}</strong>
              <span>{{ t('overview.channels') }}</span>
            </article>
            <article>
              <strong>{{ stats.nodes }}</strong>
              <span>{{ t('overview.nodes') }}</span>
            </article>
            <article>
              <strong>{{ stats.scenes }}</strong>
              <span>{{ t('overview.scenes') }}</span>
            </article>
          </div>
        </div>
      </section>

      <ChannelsPage
        v-if="activePage === 'channels'"
        :channels="channels"
        :protocol-list="protocolList"
        :schemas="schemas"
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
    </main>
  </div>
</template>

<style scoped>
.app-shell {
  min-height: 100vh;
  display: flex;
  align-items: stretch;
  background: radial-gradient(circle at 14% 10%, #eff6ff, #eef2ff 36%, #f8fafc 65%);
}

.main-content {
  flex: 1;
  min-width: 0;
  padding: 20px;
  display: grid;
  align-content: start;
  gap: 18px;
}

.overview-card {
  background: #ffffff;
  border: 1px solid var(--border);
  border-radius: 16px;
  padding: 16px;
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 16px;
}

.overview-card h2 {
  margin: 0;
  font-size: 20px;
}

.overview-card p {
  margin: 8px 0 0;
  color: var(--text-secondary);
  font-size: 14px;
}

.overview-right {
  display: grid;
  gap: 10px;
}

.overview-right label {
  display: grid;
  gap: 6px;
}

.overview-right span {
  color: var(--text-secondary);
  font-size: 12px;
}

.overview-right input {
  width: 180px;
  height: 36px;
  border: 1px solid var(--border);
  border-radius: 10px;
  padding: 0 10px;
}

.overview-right input:focus {
  outline: none;
  border-color: #60a5fa;
  box-shadow: 0 0 0 3px rgba(96, 165, 250, 0.2);
}

.stats-grid {
  display: grid;
  grid-template-columns: repeat(3, minmax(80px, 1fr));
  gap: 8px;
}

.stats-grid article {
  border: 1px solid #dbeafe;
  border-radius: 10px;
  background: #f8fbff;
  min-height: 58px;
  display: grid;
  place-items: center;
  gap: 2px;
}

.stats-grid strong {
  font-size: 20px;
  color: #1e3a8a;
}

.stats-grid span {
  color: #334155;
  font-size: 12px;
}

@media (max-width: 960px) {
  .app-shell {
    flex-direction: column;
  }

  .overview-card {
    flex-direction: column;
    align-items: flex-start;
  }

  .overview-right {
    width: 100%;
  }

  .overview-right input {
    width: 100%;
  }
}
</style>
