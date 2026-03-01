import type { DeviceConfig } from '../types/config'

interface ApiResponse<T> {
  state: number
  message?: string
  data?: T
}

export async function fetchDeviceConfig() {
  const response = await fetch('/lspcapi/device/config')
  return (await response.json()) as ApiResponse<DeviceConfig>
}

export async function saveDeviceConfig(payload: DeviceConfig) {
  const response = await fetch('/lspcapi/config/save', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json'
    },
    body: JSON.stringify(payload)
  })

  return (await response.json()) as ApiResponse<null>
}
