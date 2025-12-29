import axios from 'axios'
import type { ApiResponse } from '@/types/api'

/** MQTT API 基础配置 */
const mqttApi = axios.create({
    baseURL: '/api/mqtt-simulator',
    timeout: 10000,
})

// 添加响应拦截器处理错误
mqttApi.interceptors.response.use(
    (response) => response,
    (error) => {
        const message = error.response?.data?.message || error.message || '请求失败'
        console.error('[MQTT API Error]', message)
        return Promise.reject(new Error(message))
    }
)

// ============ MQTT 类型定义 ============

/** MQTT 协议版本 */
export type MqttVersion = 'v4' | 'v5'

/** MQTT 模拟器模式 */
export type MqttMode = 'broker' | 'proxy'

/** MQTT 模拟器状态 */
export type MqttSimulatorStatus = 'stopped' | 'running' | { error: string }

/** 代理配置 */
export interface ProxyConfig {
    upstream_host: string
    upstream_port: number
    upstream_username?: string
    upstream_password?: string
    client_id_prefix: string
}

/** MQTT 统计信息 */
export interface MqttStats {
    total_connections: number
    active_connections: number
    messages_received: number
    messages_sent: number
    bytes_received: number
    bytes_sent: number
    last_activity: string | null
}

/** MQTT 客户端信息 */
export interface MqttClientInfo {
    client_id: string
    connected_at: string
    subscriptions: string[]
    last_activity: string | null
}

/** MQTT 报文记录 */
export interface MqttPacketRecord {
    id: number
    timestamp: string
    direction: 'received' | 'sent' | 'forwarded'
    client_id: string | null
    packet_type: string
    topic: string | null
    payload: string | null
    payload_hex: string | null
    qos: number | null
}

/** MQTT 模拟器状态 */
export interface MqttSimulatorState {
    mode: MqttMode
    clients: MqttClientInfo[]
    subscriptions: Record<string, string[]>
    stats: MqttStats
}

/** MQTT 模拟器完整信息 */
export interface MqttSimulatorInfo {
    id: string
    name: string
    description: string | null
    mode: MqttMode
    port: number
    bind_addr: string
    mqtt_versions: MqttVersion[]
    status: MqttSimulatorStatus
    state: MqttSimulatorState
    proxy_config: ProxyConfig | null
    auto_start: boolean
    created_at: string
}

/** 创建 MQTT 模拟器请求 */
export interface CreateMqttSimulatorRequest {
    name: string
    description?: string
    mode: MqttMode
    port: number
    bind_addr?: string
    mqtt_versions?: MqttVersion[]
    proxy_config?: ProxyConfig
    auto_start?: boolean
}

/** 规则动作类型 */
export type MqttRuleAction =
    | { type: 'respond'; topic: string; payload: string; use_topic_vars: boolean }
    | { type: 'forward'; target_topic: string }
    | { type: 'log'; message?: string }
    | { type: 'silence' }
    | { type: 'transform'; output_topic: string; output_payload: string }

/** Payload 匹配器 */
export type PayloadMatcher =
    | { type: 'exact'; value: string }
    | { type: 'prefix'; value: string }
    | { type: 'contains'; value: string }
    | { type: 'regex'; pattern: string }
    | { type: 'json_field'; path: string; value: string }
    | { type: 'hex'; pattern: string }

/** MQTT 规则 */
export interface MqttRule {
    id: string
    name: string
    enabled: boolean
    topic_pattern: string
    payload_match: PayloadMatcher | null
    action: MqttRuleAction
    priority: number
}

/** 添加规则请求 */
export interface AddRuleRequest {
    name: string
    topic_pattern: string
    action: MqttRuleAction
}

// ============ API 方法 ============

/** 创建 MQTT 模拟器 */
export async function createMqttSimulator(data: CreateMqttSimulatorRequest): Promise<MqttSimulatorInfo> {
    const response = await mqttApi.post<ApiResponse<MqttSimulatorInfo>>('/create', data)
    return response.data.data!
}

/** 获取 MQTT 模拟器列表 */
export async function listMqttSimulators(): Promise<MqttSimulatorInfo[]> {
    const response = await mqttApi.get<ApiResponse<MqttSimulatorInfo[]>>('/list')
    return response.data.data || []
}

/** 获取 MQTT 模拟器详情 */
export async function getMqttSimulator(id: string): Promise<MqttSimulatorInfo> {
    const response = await mqttApi.get<ApiResponse<MqttSimulatorInfo>>(`/${id}`)
    return response.data.data!
}

/** 删除 MQTT 模拟器 */
export async function deleteMqttSimulator(id: string): Promise<void> {
    await mqttApi.delete(`/${id}`)
}

/** 启动 MQTT 模拟器 */
export async function startMqttSimulator(id: string): Promise<void> {
    await mqttApi.post(`/${id}/start`)
}

/** 停止 MQTT 模拟器 */
export async function stopMqttSimulator(id: string): Promise<void> {
    await mqttApi.post(`/${id}/stop`)
}

/** 获取报文记录 */
export async function getMqttPackets(id: string, limit?: number, afterId?: number): Promise<MqttPacketRecord[]> {
    const params = new URLSearchParams()
    if (limit !== undefined) params.append('limit', limit.toString())
    if (afterId !== undefined) params.append('after_id', afterId.toString())
    const query = params.toString() ? `?${params.toString()}` : ''
    const response = await mqttApi.get<ApiResponse<MqttPacketRecord[]>>(`/${id}/packets${query}`)
    return response.data.data || []
}

/** 清空报文记录 */
export async function clearMqttPackets(id: string): Promise<void> {
    await mqttApi.delete(`/${id}/packets`)
}

/** 获取规则列表 */
export async function getMqttRules(id: string): Promise<MqttRule[]> {
    const response = await mqttApi.get<ApiResponse<MqttRule[]>>(`/${id}/rules`)
    return response.data.data || []
}

/** 添加规则 */
export async function addMqttRule(id: string, data: AddRuleRequest): Promise<void> {
    await mqttApi.post(`/${id}/rules`, data)
}

/** 删除规则 */
export async function removeMqttRule(id: string, ruleId: string): Promise<void> {
    await mqttApi.delete(`/${id}/rules/${ruleId}`)
}
