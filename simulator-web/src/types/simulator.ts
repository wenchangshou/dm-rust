/** 模拟器状态 */
export type SimulatorStatus = 'running' | 'stopped' | 'error'

/** 连接统计 */
export interface ConnectionStats {
  total_connections: number
  active_connections: number
  bytes_received: number
  bytes_sent: number
  last_activity: string | null
}

/** 模拟器状态 */
export interface SimulatorState {
  online: boolean
  fault: string | null
  values: Record<string, unknown>
  stats: ConnectionStats
}

/** 模拟器信息 */
export interface SimulatorInfo {
  id: string
  name: string
  protocol: string
  bind_addr: string
  port: number
  status: SimulatorStatus
  state: SimulatorState
}

/** 协议信息 */
export interface ProtocolInfo {
  name: string
  description: string
  default_port: number
  commands: string[]
}

/** 创建模拟器请求 */
export interface CreateSimulatorRequest {
  name: string
  protocol: string
  bind_addr?: string
  port: number
  initial_state?: Record<string, unknown>
  auto_start?: boolean
}

/** 更新状态请求 */
export interface UpdateStateRequest {
  online?: boolean
  fault?: string
}

/** 触发故障请求 */
export interface TriggerFaultRequest {
  fault_type: string
}
