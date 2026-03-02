import { ref } from 'vue'
import { fetchSchemaByName, fetchSchemaList } from './schemaApi'
import { logger } from '../utils/logger'

function normalizeKey(input: string) {
  return input
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9]/g, '')
}

function aliasCandidates(name: string) {
  const trimmed = name.trim()
  if (!trimmed) {
    return []
  }

  const lower = trimmed.toLowerCase()
  const normalized = normalizeKey(trimmed)
  return Array.from(new Set([trimmed, lower, normalized]))
}

export function useSchemaRegistry() {
  const schemas = ref<Record<string, Record<string, unknown>>>({})
  const protocolList = ref<string[]>([])

  const applySchema = (requestName: string, actualName: string, schema: Record<string, unknown>) => {
    logger.debug('schemaRegistry', 'apply schema', {
      requestName,
      actualName,
      aliases: [...aliasCandidates(requestName), ...aliasCandidates(actualName)]
    })

    for (const key of [...aliasCandidates(requestName), ...aliasCandidates(actualName)]) {
      schemas.value[key] = schema
    }

    if (!protocolList.value.includes(actualName)) {
      protocolList.value = [...protocolList.value, actualName].sort((a, b) => a.localeCompare(b))
    }
  }

  const resolveSchema = (name: string) => {
    for (const key of aliasCandidates(name)) {
      const schema = schemas.value[key]
      if (schema) {
        return schema
      }
    }
    return undefined
  }

  const ensureSchema = async (name: string) => {
    const trimmed = name.trim()
    if (!trimmed) {
      logger.warn('schemaRegistry', 'ensure schema skipped due to empty name')
      return
    }

    if (resolveSchema(trimmed)) {
      logger.debug('schemaRegistry', 'schema cache hit', { name: trimmed })
      return
    }

    logger.info('schemaRegistry', 'schema cache miss, requesting', { name: trimmed })
    const response = await fetchSchemaByName(trimmed)
    if (response.state !== 0 || !response.data?.schema || !response.data?.name) {
      logger.error('schemaRegistry', 'schema request failed', {
        name: trimmed,
        state: response.state,
        message: response.message
      })
      throw new Error(response.message || `schema load failed: ${trimmed}`)
    }

    applySchema(trimmed, response.data.name, response.data.schema)
    logger.info('schemaRegistry', 'schema loaded', {
      requestName: trimmed,
      actualName: response.data.name,
      schemaKeys: Object.keys(response.data.schema)
    })
  }

  const initProtocols = async () => {
    logger.info('schemaRegistry', 'init protocols start')
    const response = await fetchSchemaList()
    if (response.state !== 0 || !response.data) {
      logger.error('schemaRegistry', 'init protocols failed', {
        state: response.state,
        message: response.message
      })
      throw new Error(response.message || 'schema list load failed')
    }

    protocolList.value = [...response.data].sort((a, b) => a.localeCompare(b))
    logger.info('schemaRegistry', 'init protocols done', {
      count: protocolList.value.length,
      protocols: protocolList.value
    })
  }

  return {
    schemas,
    protocolList,
    initProtocols,
    ensureSchema,
    resolveSchema
  }
}
