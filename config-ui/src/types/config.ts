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
  max_body_limit?: number
}

export interface FileConfig {
  enable: boolean
  path: string
}

export interface DatabaseConfig {
  enable: boolean
  url: string
}

export interface ResourceConfig {
  enable: boolean
  path: string
  url_prefix: string
}

export interface DeviceConfig {
  web_server: WebServerConfig
  channels: Channel[]
  nodes: NodeItem[]
  scenes: Scene[]
  file?: FileConfig
  database?: DatabaseConfig
  resource?: ResourceConfig
}

export type PageKey = 'overview' | 'channels' | 'nodes' | 'scenes' | 'settings'

export type ToastType = 'success' | 'error'
