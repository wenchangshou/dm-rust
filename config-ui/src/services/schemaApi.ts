import { logger } from '../utils/logger'

interface ApiResponse<T> {
  state: number
  message?: string
  data?: T
}

export interface SchemaPayload {
  name: string
  schema: Record<string, unknown>
}

export async function fetchSchemaList() {
  logger.info('schemaApi', 'fetch schema list start', { url: '/lspcapi/schema' })
  const response = await fetch('/lspcapi/schema')
  const result = (await response.json()) as ApiResponse<string[]>
  logger.info('schemaApi', 'fetch schema list done', {
    httpStatus: response.status,
    state: result.state,
    size: result.data?.length ?? 0,
    message: result.message
  })
  return result
}

export async function fetchSchemaByName(name: string) {
  const url = `/lspcapi/schema/${encodeURIComponent(name)}`
  logger.info('schemaApi', 'fetch schema start', { name, url })
  const response = await fetch(url)
  const result = (await response.json()) as ApiResponse<SchemaPayload>
  logger.info('schemaApi', 'fetch schema done', {
    requestName: name,
    httpStatus: response.status,
    state: result.state,
    returnedName: result.data?.name,
    hasSchema: Boolean(result.data?.schema),
    message: result.message
  })
  return result
}
