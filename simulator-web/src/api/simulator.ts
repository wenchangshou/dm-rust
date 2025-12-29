import api from './index'
import type { ApiResponse } from '@/types/api'
import type {
  SimulatorInfo,
  ProtocolInfo,
  CreateSimulatorRequest,
  UpdateStateRequest,
  TriggerFaultRequest,
  ModbusSlaveConfig,
  UpdateModbusRegisterRequest,
  BatchUpdateModbusRegistersRequest,
  AddModbusSlaveRequest,
  SetModbusRegisterRequest,
  DeleteModbusRegisterRequest,
  GetPacketsResponse,
  PacketMonitorSettingsRequest,
  UpdateSimulatorInfoRequest,
} from '@/types/simulator'

/** 获取支持的协议列表 */
export async function getProtocols(): Promise<ProtocolInfo[]> {
  const response = await api.get<ApiResponse<ProtocolInfo[]>>('/protocols')
  return response.data.data || []
}

/** 创建模拟器 */
export async function createSimulator(data: CreateSimulatorRequest): Promise<SimulatorInfo> {
  const response = await api.post<ApiResponse<SimulatorInfo>>('/create', data)
  return response.data.data!
}

/** 获取模拟器列表 */
export async function listSimulators(): Promise<SimulatorInfo[]> {
  const response = await api.get<ApiResponse<SimulatorInfo[]>>('/list')
  return response.data.data || []
}

/** 获取模拟器详情 */
export async function getSimulator(id: string): Promise<SimulatorInfo> {
  const response = await api.get<ApiResponse<SimulatorInfo>>(`/${id}`)
  return response.data.data!
}

/** 删除模拟器 */
export async function deleteSimulator(id: string): Promise<void> {
  await api.delete(`/${id}`)
}

/** 启动模拟器 */
export async function startSimulator(id: string): Promise<SimulatorInfo> {
  const response = await api.post<ApiResponse<SimulatorInfo>>(`/${id}/start`)
  return response.data.data!
}

/** 停止模拟器 */
export async function stopSimulator(id: string): Promise<SimulatorInfo> {
  const response = await api.post<ApiResponse<SimulatorInfo>>(`/${id}/stop`)
  return response.data.data!
}

/** 更新模拟器状态 */
export async function updateState(id: string, data: UpdateStateRequest): Promise<SimulatorInfo> {
  const response = await api.post<ApiResponse<SimulatorInfo>>(`/${id}/state`, data)
  return response.data.data!
}

/** 触发故障 */
export async function triggerFault(id: string, data: TriggerFaultRequest): Promise<SimulatorInfo> {
  const response = await api.post<ApiResponse<SimulatorInfo>>(`/${id}/fault`, data)
  return response.data.data!
}

/** 清除故障 */
export async function clearFault(id: string): Promise<SimulatorInfo> {
  const response = await api.post<ApiResponse<SimulatorInfo>>(`/${id}/clear-fault`)
  return response.data.data!
}

/** 设置在线状态 */
export async function setOnline(id: string, online: boolean): Promise<SimulatorInfo> {
  const response = await api.post<ApiResponse<SimulatorInfo>>(`/${id}/online`, { online })
  return response.data.data!
}

/** 更新模拟器配置 */
export async function updateSimulatorConfig(id: string, protocolConfig: any): Promise<SimulatorInfo> {
  const response = await api.post<ApiResponse<SimulatorInfo>>(`/${id}/config`, { protocol_config: protocolConfig })
  return response.data.data!
}

/** 更新模拟器基本信息 */
export async function updateSimulatorInfo(id: string, data: UpdateSimulatorInfoRequest): Promise<SimulatorInfo> {
  const response = await api.post<ApiResponse<SimulatorInfo>>(`/${id}/info`, data)
  return response.data.data!
}

// ============ Modbus 模拟器 API ============

/** 获取 Modbus Slaves 配置 */
export async function getModbusSlaves(id: string): Promise<ModbusSlaveConfig[]> {
  const response = await api.get<ApiResponse<ModbusSlaveConfig[]>>(`/${id}/modbus/slaves`)
  return response.data.data || []
}

/** 添加 Modbus Slave */
export async function addModbusSlave(id: string, data: AddModbusSlaveRequest): Promise<SimulatorInfo> {
  const response = await api.post<ApiResponse<SimulatorInfo>>(`/${id}/modbus/slave`, data)
  return response.data.data!
}

/** 删除 Modbus Slave */
export async function deleteModbusSlave(id: string, slaveId: number): Promise<SimulatorInfo> {
  const response = await api.delete<ApiResponse<SimulatorInfo>>(`/${id}/modbus/slave/${slaveId}`)
  return response.data.data!
}

/** 设置/更新 Modbus 寄存器 */
export async function setModbusRegister(id: string, data: SetModbusRegisterRequest): Promise<SimulatorInfo> {
  const response = await api.post<ApiResponse<SimulatorInfo>>(`/${id}/modbus/register`, data)
  return response.data.data!
}

/** 删除 Modbus 寄存器 */
export async function deleteModbusRegister(id: string, data: DeleteModbusRegisterRequest): Promise<SimulatorInfo> {
  const response = await api.post<ApiResponse<SimulatorInfo>>(`/${id}/modbus/register/delete`, data)
  return response.data.data!
}

/** 更新单个 Modbus 寄存器值 */
export async function updateModbusRegisterValue(id: string, data: UpdateModbusRegisterRequest): Promise<SimulatorInfo> {
  const response = await api.post<ApiResponse<SimulatorInfo>>(`/${id}/modbus/register/value`, data)
  return response.data.data!
}

/** 批量更新 Modbus 寄存器值 */
export async function batchUpdateModbusRegisters(id: string, data: BatchUpdateModbusRegistersRequest): Promise<SimulatorInfo> {
  const response = await api.post<ApiResponse<SimulatorInfo>>(`/${id}/modbus/registers/batch`, data)
  return response.data.data!
}

// ============ 报文监控 API ============

/** 获取报文列表 */
export async function getPackets(id: string, afterId?: number, limit?: number): Promise<GetPacketsResponse> {
  const params = new URLSearchParams()
  if (afterId !== undefined) params.append('afterId', afterId.toString())
  if (limit !== undefined) params.append('limit', limit.toString())
  const query = params.toString() ? `?${params.toString()}` : ''
  const response = await api.get<ApiResponse<GetPacketsResponse>>(`/${id}/packets${query}`)
  return response.data.data!
}

/** 清空报文记录 */
export async function clearPackets(id: string): Promise<void> {
  await api.delete(`/${id}/packets`)
}

/** 设置报文监控选项 */
export async function setPacketMonitorSettings(id: string, settings: PacketMonitorSettingsRequest): Promise<SimulatorInfo> {
  const response = await api.post<ApiResponse<SimulatorInfo>>(`/${id}/packets/settings`, settings)
  return response.data.data!
}

// ============ 客户端连接 API ============

import type { ClientConnection } from '@/types/simulator'

/** 获取客户端连接列表 */
export async function listClients(id: string): Promise<ClientConnection[]> {
  const response = await api.get<ApiResponse<ClientConnection[]>>(`/${id}/clients`)
  return response.data.data || []
}

/** 断开客户端连接 */
export async function disconnectClient(id: string, clientId: string): Promise<void> {
  await api.post(`/${id}/clients/${clientId}/disconnect`)
}

// ============ Debug 模式 API ============

/** Debug 模式状态响应 */
export interface DebugModeResponse {
  debug_mode: boolean
  log_path: string | null
}

/** 设置 Debug 模式 */
export async function setDebugMode(id: string, enabled: boolean): Promise<DebugModeResponse> {
  const response = await api.post<ApiResponse<DebugModeResponse>>(`/${id}/debug`, { enabled })
  return response.data.data!
}

/** 获取 Debug 模式状态 */
export async function getDebugStatus(id: string): Promise<DebugModeResponse> {
  const response = await api.get<ApiResponse<DebugModeResponse>>(`/${id}/debug`)
  return response.data.data!
}

/** 下载 Debug 日志 */
export function getDebugLogUrl(id: string): string {
  return `/lspcapi/tcp-simulator/${id}/debug/log`
}

// ============ 模板管理 API ============

import type {
  SimulatorTemplate,
  CreateFromTemplateRequest,
  SaveAsTemplateRequest,
} from '@/types/simulator'

/** 获取模板列表 */
export async function listTemplates(): Promise<SimulatorTemplate[]> {
  const response = await api.get<ApiResponse<SimulatorTemplate[]>>('/templates')
  return response.data.data || []
}

/** 删除模板 */
export async function deleteTemplate(id: string): Promise<void> {
  await api.delete(`/templates/${id}`)
}

/** 更新模板 */
export async function updateTemplate(id: string, data: { name?: string; description?: string; config?: any }): Promise<SimulatorTemplate> {
  const response = await api.put<ApiResponse<SimulatorTemplate>>(`/templates/${id}`, data)
  return response.data.data!
}

/** 创建模板 */
export async function createTemplate(data: { name: string; description?: string; protocol: string; transport: string; config?: any }): Promise<SimulatorTemplate> {
  const response = await api.post<ApiResponse<SimulatorTemplate>>('/templates', data)
  return response.data.data!
}

/** 基于模板创建模拟器 */
export async function createFromTemplate(data: CreateFromTemplateRequest): Promise<SimulatorInfo> {
  const response = await api.post<ApiResponse<SimulatorInfo>>('/create-from-template', data)
  return response.data.data!
}

/** 将模拟器保存为模板 */
export async function saveAsTemplate(id: string, data: SaveAsTemplateRequest): Promise<SimulatorTemplate> {
  const response = await api.post<ApiResponse<SimulatorTemplate>>(`/${id}/save-as-template`, data)
  return response.data.data!
}
