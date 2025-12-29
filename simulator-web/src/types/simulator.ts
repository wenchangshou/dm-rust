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

// ============ Modbus 模拟器相关类型 ============

/** Modbus 寄存器类型 */
export type ModbusRegisterType =
  | 'coil'              // 线圈 (0x) - 读写单bit
  | 'discrete_input'    // 离散输入 (1x) - 只读单bit
  | 'holding_register'  // 保持寄存器 (4x) - 读写16bit
  | 'input_register'    // 输入寄存器 (3x) - 只读16bit

/** Modbus 数据类型 */
export type ModbusDataType =
  | 'bit'       // 单bit (用于 coil/discrete_input)
  | 'uint16'    // 无符号16位
  | 'int16'     // 有符号16位
  | 'uint32'    // 无符号32位 (占2个寄存器)
  | 'int32'     // 有符号32位 (占2个寄存器)
  | 'float32'   // 32位浮点 (占2个寄存器)

/** 值生成模式 */
export type GeneratorMode =
  | 'fixed'      // 固定值（默认）
  | 'random'     // 随机值
  | 'increment'  // 递增
  | 'decrement'  // 递减
  | 'sine'       // 正弦波
  | 'toggle'     // 开关切换（用于 Bit 类型）
  | 'sequence'   // 序列循环

/** 值生成器配置 */
export interface GeneratorConfig {
  mode: GeneratorMode       // 生成模式
  min?: number              // 最小值（用于 random, increment, decrement, sine）
  max?: number              // 最大值（用于 random, increment, decrement, sine）
  step?: number             // 步长（用于 increment, decrement）
  period?: number           // 周期（毫秒，用于 sine, toggle）
  sequence?: number[]       // 序列值（用于 sequence）
  interval: number          // 更新间隔（毫秒）
}

/** Modbus 寄存器配置 */
export interface ModbusRegisterConfig {
  address: number           // 寄存器地址
  type: ModbusRegisterType  // 寄存器类型
  dataType: ModbusDataType  // 数据类型
  name?: string             // 寄存器名称/描述
  value: number | boolean   // 当前值
  readonly?: boolean        // 是否只读 (用于 UI 提示)
  generator?: GeneratorConfig // 值生成器配置
}

/** Modbus Slave 配置 */
export interface ModbusSlaveConfig {
  slaveId: number                         // Slave ID (1-247)
  registers: ModbusRegisterConfig[]       // 寄存器列表
}

/** Modbus 模拟器状态 (扩展 SimulatorState.values) */
export interface ModbusSimulatorValues {
  slaves: ModbusSlaveConfig[]             // Slave 配置列表
  defaultSlaveId?: number                 // 默认 Slave ID
}

/** 更新 Modbus 寄存器请求 */
export interface UpdateModbusRegisterRequest {
  slaveId: number
  registerType: ModbusRegisterType
  address: number
  value: number | boolean
}

/** 批量更新 Modbus 寄存器请求 */
export interface BatchUpdateModbusRegistersRequest {
  updates: UpdateModbusRegisterRequest[]
}

/** 添加 Modbus Slave 请求 */
export interface AddModbusSlaveRequest {
  slaveId: number
  registers?: ModbusRegisterConfig[]
}

/** 添加/更新 Modbus 寄存器请求 */
export interface SetModbusRegisterRequest {
  slaveId: number
  register: ModbusRegisterConfig
}

/** 删除 Modbus 寄存器请求 */
export interface DeleteModbusRegisterRequest {
  slaveId: number
  registerType: ModbusRegisterType
  address: number
}

// ============ 报文监控相关类型 ============

/** 报文方向 */
export type PacketDirection = 'received' | 'sent'

/** 报文记录 */
export interface PacketRecord {
  /** 唯一 ID */
  id: number
  /** 时间戳 (ISO 8601) */
  timestamp: string
  /** 方向 */
  direction: PacketDirection
  /** 客户端地址 */
  peer_addr: string
  /** 十六进制数据 */
  hex_data: string
  /** 数据大小（字节） */
  size: number
  /** 协议解析信息（可选） */
  parsed?: unknown
}

/** 报文监控器状态 */
export interface PacketMonitor {
  /** 是否启用 */
  enabled: boolean
  /** 最大报文数量 */
  max_packets: number
  /** 报文列表 */
  packets: PacketRecord[]
  /** Debug 模式 */
  debug_mode?: boolean
  /** Debug 日志路径 */
  debug_log_path?: string | null
}

/** 获取报文响应 */
export interface GetPacketsResponse {
  packets: PacketRecord[]
  total: number
}

/** 报文监控设置请求 */
export interface PacketMonitorSettingsRequest {
  enabled?: boolean
  maxPackets?: number
}

// ============ 客户端连接相关类型 ============

/** 客户端连接信息 */
export interface ClientConnection {
  /** 唯一标识 */
  id: string
  /** 客户端地址 */
  peer_addr: string
  /** 连接时间 (ISO 8601) */
  connected_at: string
  /** 接收字节数 */
  bytes_received: number
  /** 发送字节数 */
  bytes_sent: number
  /** 最后活动时间 (ISO 8601) */
  last_activity: string
}

// ============ 模板相关类型 ============

/** 模拟器模板 */
export interface SimulatorTemplate {
  id: string
  name: string
  description: string
  protocol: string
  config: Record<string, unknown>
  values: Record<string, unknown>
  created_at: string
  updated_at: string
}

/** 从模板创建请求 */
export interface CreateFromTemplateRequest {
  template_id: string
  name: string
  bind_addr: string
  port: number
}

/** 保存为模板请求 */
export interface SaveAsTemplateRequest {
  name: string
  description: string
}
