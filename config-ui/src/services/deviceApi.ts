import { logger } from '../utils/logger'

interface ApiResponse<T> {
    state: number
    message?: string
    data?: T
}

export interface SceneExecutionStatus {
    is_executing: boolean
    current_scene?: string
    current_step_index?: number
    total_steps?: number
}

/** 读取单个节点状态 */
export async function readNodeState(globalId: number) {
    logger.info('deviceApi', 'read node state', { globalId })
    const response = await fetch('/lspcapi/device/read', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ global_id: globalId })
    })
    const result = (await response.json()) as ApiResponse<{ value: number }>
    logger.info('deviceApi', 'read node state done', { globalId, state: result.state, data: result.data })
    return result
}

/** 写入单个节点值 */
export async function writeNodeValue(globalId: number, value: number) {
    logger.info('deviceApi', 'write node value', { globalId, value })
    const response = await fetch('/lspcapi/device/write', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ global_id: globalId, value })
    })
    const result = (await response.json()) as ApiResponse<null>
    logger.info('deviceApi', 'write node value done', { globalId, value, state: result.state })
    return result
}

/** 执行场景 */
export async function executeScene(sceneName: string) {
    logger.info('deviceApi', 'execute scene', { name: sceneName })
    const response = await fetch('/lspcapi/device/scene', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ name: sceneName })
    })
    const result = (await response.json()) as ApiResponse<null>
    logger.info('deviceApi', 'execute scene done', { name: sceneName, state: result.state })
    return result
}

/** 获取场景执行状态 */
export async function getSceneExecutionStatus() {
    logger.info('deviceApi', 'get scene execution status')
    const response = await fetch('/lspcapi/device/sceneStatus')
    const result = (await response.json()) as ApiResponse<SceneExecutionStatus>
    logger.info('deviceApi', 'get scene execution status done', {
        httpStatus: response.status,
        state: result.state,
        data: result.data
    })
    return result
}
