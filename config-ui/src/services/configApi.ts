import type { DeviceConfig } from '../types/config'
import { logger } from '../utils/logger'

interface ApiResponse<T> {
  state: number
  message?: string
  data?: T
}

export async function fetchDeviceConfig() {
  logger.info('configApi', 'fetch device config start', { url: '/lspcapi/device/config' })
  const response = await fetch('/lspcapi/device/config')
  const result = (await response.json()) as ApiResponse<DeviceConfig>
  logger.info('configApi', 'fetch device config done', {
    httpStatus: response.status,
    state: result.state,
    message: result.message,
    channels: result.data?.channels?.length ?? 0,
    nodes: result.data?.nodes?.length ?? 0,
    scenes: result.data?.scenes?.length ?? 0
  })
  return result
}

export async function saveDeviceConfig(payload: DeviceConfig) {
  logger.info('configApi', 'save device config start', {
    channels: payload.channels.length,
    nodes: payload.nodes.length,
    scenes: payload.scenes.length
  })
  const response = await fetch('/lspcapi/config/save', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json'
    },
    body: JSON.stringify(payload)
  })

  const result = (await response.json()) as ApiResponse<null>
  logger.info('configApi', 'save device config done', {
    httpStatus: response.status,
    state: result.state,
    message: result.message
  })
  return result
}

export async function reloadDeviceRuntime() {
  logger.info('configApi', 'reload runtime config start', { url: '/lspcapi/config/reload' })
  const response = await fetch('/lspcapi/config/reload', {
    method: 'POST'
  })
  const result = (await response.json()) as ApiResponse<{ requires_restart?: boolean; port_changed?: boolean }>
  logger.info('configApi', 'reload runtime config done', {
    httpStatus: response.status,
    state: result.state,
    message: result.message,
    requiresRestart: result.data?.requires_restart ?? false
  })
  return result
}
