import { computed, ref } from 'vue'
import type { Channel, DeviceConfig, NodeItem, Scene, WebServerConfig } from '../types/config'
import { deepClone } from '../utils/deepClone'

const defaultWebServer: WebServerConfig = {
  port: 18080
}

export function useConfigSystem() {
  const channels = ref<Channel[]>([])
  const nodes = ref<NodeItem[]>([])
  const scenes = ref<Scene[]>([])
  const webServer = ref<WebServerConfig>(deepClone(defaultWebServer))

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
  }

  const toPayload = (): DeviceConfig => ({
    web_server: deepClone(webServer.value),
    channels: deepClone(channels.value),
    nodes: deepClone(nodes.value),
    scenes: deepClone(scenes.value)
  })

  return {
    channels,
    nodes,
    scenes,
    webServer,
    loading,
    saving,
    stats,
    setConfig,
    toPayload
  }
}
