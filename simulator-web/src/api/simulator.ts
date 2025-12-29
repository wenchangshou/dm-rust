import api from './index'
import type { ApiResponse } from '@/types/api'
import type {
  SimulatorInfo,
  ProtocolInfo,
  CreateSimulatorRequest,
  UpdateStateRequest,
  TriggerFaultRequest,
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
