import { computed, ref } from 'vue'
import type { Channel, DatabaseConfig, DeviceConfig, FileConfig, NodeItem, ResourceConfig, Scene, WebServerConfig } from '../types/config'
import { deepClone } from '../utils/deepClone'

const defaultWebServer: WebServerConfig = {
  port: 18080
}

const defaultFileConfig: FileConfig = {
  enable: false,
  path: '/file'
}

const defaultDatabaseConfig: DatabaseConfig = {
  enable: false,
  url: ''
}

const defaultResourceConfig: ResourceConfig = {
  enable: false,
  path: '/data',
  url_prefix: '/static'
}

export function useConfigSystem() {
  const channels = ref<Channel[]>([])
  const nodes = ref<NodeItem[]>([])
  const scenes = ref<Scene[]>([])
  const webServer = ref<WebServerConfig>(deepClone(defaultWebServer))
  const fileConfig = ref<FileConfig>(deepClone(defaultFileConfig))
  const databaseConfig = ref<DatabaseConfig>(deepClone(defaultDatabaseConfig))
  const resourceConfig = ref<ResourceConfig>(deepClone(defaultResourceConfig))

  const loading = ref(false)
  const saving = ref(false)

  const stats = computed(() => ({
    channels: channels.value.length,
    nodes: nodes.value.length,
    scenes: scenes.value.length
  }))

  const setConfig = (payload?: Partial<DeviceConfig>) => {
    channels.value = deepClone(payload?.channels ?? [])
    nodes.value = deepClone(payload?.nodes ?? [])
    scenes.value = deepClone(payload?.scenes ?? [])
    webServer.value = deepClone(payload?.web_server ?? defaultWebServer)
    fileConfig.value = deepClone(payload?.file ?? defaultFileConfig)
    databaseConfig.value = deepClone(payload?.database ?? defaultDatabaseConfig)
    resourceConfig.value = deepClone(payload?.resource ?? defaultResourceConfig)
  }

  const toPayload = (): DeviceConfig => ({
    web_server: deepClone(webServer.value),
    channels: deepClone(channels.value),
    nodes: deepClone(nodes.value),
    scenes: deepClone(scenes.value),
    file: deepClone(fileConfig.value),
    database: deepClone(databaseConfig.value),
    resource: deepClone(resourceConfig.value)
  })

  return {
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
  }
}

