export interface Channel {
  channel_id: number
  enable: boolean
  statute: string
  description?: string
  arguments: Record<string, unknown>
}

export interface NodeItem {
  global_id: number
  channel_id: number
  id: number
  alias: string
}

export interface SceneNode {
  id: number
  value: number
  delay?: number
}

export interface Scene {
  name: string
  nodes: SceneNode[]
}

export interface WebServerConfig {
  port: number
}

export interface DeviceConfig {
  web_server: WebServerConfig
  channels: Channel[]
  nodes: NodeItem[]
  scenes: Scene[]
}

export type PageKey = 'channels' | 'nodes' | 'scenes'

export type ToastType = 'success' | 'error'
