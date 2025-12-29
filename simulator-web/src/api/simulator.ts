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
