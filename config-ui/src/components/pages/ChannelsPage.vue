<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, ref, watch } from 'vue'
import { ElMessageBox } from 'element-plus'
import { useI18n } from '../../composables/useI18n'
import type { Channel, ToastType } from '../../types/config'
import { deepClone } from '../../utils/deepClone'
import { logger } from '../../utils/logger'

const props = defineProps<{
  channels: Channel[]
  protocolList: string[]
  ensureSchema: (name: string) => Promise<void>
  resolveSchema: (name: string) => Record<string, unknown> | undefined
}>()

const emit = defineEmits<{
  (e: 'update:channels', value: Channel[]): void
  (e: 'notify', payload: { message: string; type?: ToastType }): void
}>()

const { t } = useI18n()
const schemaOf = (name: string) => props.resolveSchema(name)

const editingChannel = ref<Channel | null>(null)
const editingIndex = ref(-1)
const editorVisible = ref(false)
const editorTab = ref<'form' | 'json'>('form')

const keyword = ref('')
const protocolFilter = ref('all')
const statusFilter = ref<'all' | 'enabled' | 'disabled'>('all')

const editorRef = ref<HTMLElement | null>(null)
const schemaLoading = ref(false)
const schemaError = ref('')
const rawArgsText = ref('{}')
const rawArgsError = ref('')

interface SchemaFieldInfo {
  key: string
  type: string
  required: boolean
  hasDefault: boolean
  defaultText: string
  defaultValue: unknown
  enumValues: string[]
}

type SchemaObject = {
  title?: string
  type?: string
  properties?: Record<string, unknown>
  required?: string[]
  defaultProperties?: string[]
}

let jsonEditor: {
  destroy: () => void
  on: (event: string, callback: () => void) => void
  getValue: () => Record<string, unknown>
  setValue?: (value: Record<string, unknown>) => void
} | null = null

let jsonEditorCtor: (new (...args: unknown[]) => unknown) | null = null

const filteredChannels = computed(() => {
  const key = keyword.value.trim().toLowerCase()

  return props.channels.filter((channel) => {
    if (protocolFilter.value !== 'all' && channel.statute !== protocolFilter.value) {
      return false
    }

    if (statusFilter.value === 'enabled' && !channel.enable) {
      return false
    }

    if (statusFilter.value === 'disabled' && channel.enable) {
      return false
    }

    if (!key) {
      return true
    }

    const combined = [
      String(channel.channel_id),
      channel.description ?? '',
      channel.statute ?? ''
    ].join(' ').toLowerCase()

    return combined.includes(key)
  })
})

const channelPage = ref(1)
const channelPageSize = ref(10)
const channelPageSizeOptions = [10, 20, 50, 100]

const pagedChannels = computed(() => {
  const start = (channelPage.value - 1) * channelPageSize.value
  return filteredChannels.value.slice(start, start + channelPageSize.value)
})

const channelRowIndex = (index: number) => {
  return (channelPage.value - 1) * channelPageSize.value + index + 1
}

const currentSchema = computed<SchemaObject | null>(() => {
  const statute = editingChannel.value?.statute
  if (!statute) {
    return null
  }

  const schema = schemaOf(statute)
  return (schema as SchemaObject | undefined) ?? null
})

const schemaFields = computed<SchemaFieldInfo[]>(() => {
  const schema = currentSchema.value
  const args = editingChannel.value?.arguments ?? {}
  const schemaProps = schema?.properties ?? {}
  const requiredSet = new Set(schema?.required ?? [])
  const seenKeys = new Set<string>()

  const fields: SchemaFieldInfo[] = []

  // 1) schema-defined fields
  for (const [key, propertyValue] of Object.entries(schemaProps)) {
    seenKeys.add(key)
    const property = (propertyValue as {
      type?: string
      default?: unknown
      enum?: string[]
    }) ?? {}

    const defaultText = property.default === undefined
      ? '-'
      : typeof property.default === 'object'
        ? JSON.stringify(property.default)
        : String(property.default)

    fields.push({
      key,
      type: property.type ?? 'unknown',
      required: requiredSet.has(key),
      hasDefault: property.default !== undefined,
      defaultText,
      defaultValue: property.default,
      enumValues: property.enum ?? []
    })
  }

  // 2) extra fields from existing arguments not in schema
  for (const [key, value] of Object.entries(args)) {
    if (seenKeys.has(key)) continue

    let inferredType = 'string'
    if (typeof value === 'number') {
      inferredType = Number.isInteger(value) ? 'integer' : 'number'
    } else if (typeof value === 'boolean') {
      inferredType = 'boolean'
    } else if (value !== null && typeof value === 'object') {
      inferredType = Array.isArray(value) ? 'array' : 'object'
    }

    fields.push({
      key,
      type: inferredType,
      required: false,
      hasDefault: false,
      defaultText: '-',
      defaultValue: undefined,
      enumValues: []
    })
  }

  return fields
})

const getArgValue = (key: string) => {
  return editingChannel.value?.arguments?.[key]
}

const setArgValue = (key: string, value: unknown) => {
  if (!editingChannel.value) return
  if (!editingChannel.value.arguments) {
    editingChannel.value.arguments = {}
  }
  editingChannel.value.arguments[key] = value
  syncRawFromArguments()
}

const getArgJsonText = (key: string): string => {
  const val = editingChannel.value?.arguments?.[key]
  return val === undefined ? '' : JSON.stringify(val, null, 2)
}

const setArgJsonText = (key: string, text: string) => {
  if (!editingChannel.value) return
  if (!editingChannel.value.arguments) {
    editingChannel.value.arguments = {}
  }
  try {
    editingChannel.value.arguments[key] = JSON.parse(text) as unknown
  } catch {
    // keep raw text, don't update until valid
  }
  syncRawFromArguments()
}

const enabledCount = computed(() => props.channels.filter((channel) => channel.enable).length)
const disabledCount = computed(() => props.channels.filter((channel) => !channel.enable).length)

function resolveJsonEditorCtor(source: unknown): (new (...args: unknown[]) => unknown) | null {
  const value = source as {
    default?: unknown
    JSONEditor?: unknown
  }

  const candidates: unknown[] = [
    source,
    value?.default,
    value?.JSONEditor,
    (value?.default as { default?: unknown } | undefined)?.default,
    (value?.default as { JSONEditor?: unknown } | undefined)?.JSONEditor
  ]

  for (const candidate of candidates) {
    if (typeof candidate === 'function') {
      return candidate as new (...args: unknown[]) => unknown
    }
  }

  return null
}

const channelName = (channel: Channel) => channel.description?.trim() || t('channels.defaultName', { id: channel.channel_id })

const channelArgumentCount = (channel: Channel) => Object.keys(channel.arguments ?? {}).length

const resetFilters = () => {
  keyword.value = ''
  protocolFilter.value = 'all'
  statusFilter.value = 'all'
}

watch([keyword, protocolFilter, statusFilter], () => {
  channelPage.value = 1
})

watch(channelPageSize, () => {
  channelPage.value = 1
})

watch(
  () => filteredChannels.value.length,
  (length) => {
    const maxPage = Math.max(1, Math.ceil(length / channelPageSize.value))
    if (channelPage.value > maxPage) {
      channelPage.value = maxPage
    }
  }
)

const destroyEditor = () => {
  if (jsonEditor) {
    logger.debug('channels', 'destroy json editor')
    jsonEditor.destroy()
    jsonEditor = null
  }
}

const syncRawFromArguments = () => {
  rawArgsText.value = JSON.stringify(editingChannel.value?.arguments ?? {}, null, 2)
  rawArgsError.value = ''
}

const syncEditorValue = () => {
  if (!jsonEditor?.setValue || !editingChannel.value) {
    return
  }

  try {
    jsonEditor.setValue(editingChannel.value.arguments ?? {})
  } catch (error) {
    logger.warn('channels', 'sync editor value failed', { error: String(error) })
  }
}

const applyDefaults = () => {
  const channel = editingChannel.value
  const schema = currentSchema.value
  if (!channel || !schema?.properties) {
    return
  }

  const merged = deepClone(channel.arguments ?? {})
  for (const [key, propertyValue] of Object.entries(schema.properties)) {
    const property = (propertyValue as { default?: unknown }) ?? {}
    if (property.default !== undefined && merged[key] === undefined) {
      merged[key] = deepClone(property.default)
    }
  }

  channel.arguments = merged
  syncRawFromArguments()
  syncEditorValue()
  logger.info('channels', 'apply defaults', { statute: channel.statute, keys: Object.keys(merged) })
}

const clearArguments = () => {
  if (!editingChannel.value) {
    return
  }

  editingChannel.value.arguments = {}
  syncRawFromArguments()
  syncEditorValue()
  logger.info('channels', 'clear protocol arguments')
}

const applyRawJson = () => {
  if (!editingChannel.value) {
    return
  }

  try {
    const parsed = JSON.parse(rawArgsText.value) as unknown
    if (!parsed || typeof parsed !== 'object' || Array.isArray(parsed)) {
      rawArgsError.value = t('channels.rawJsonObjectError')
      return
    }

    editingChannel.value.arguments = parsed as Record<string, unknown>
    rawArgsError.value = ''
    syncEditorValue()
    logger.info('channels', 'apply raw json success', {
      statute: editingChannel.value.statute,
      keys: Object.keys(editingChannel.value.arguments)
    })
  } catch (error) {
    rawArgsError.value = error instanceof Error ? error.message : String(error)
    logger.warn('channels', 'apply raw json failed', { error: rawArgsError.value })
  }
}

const initEditor = async (retries = 5) => {
  await nextTick()
  const channel = editingChannel.value
  if (!editorVisible.value || editorTab.value !== 'form' || !channel || !channel.statute) {
    logger.debug('channels', 'init editor skipped (dialog/tab/channel invalid)')
    return
  }

  logger.info('channels', 'init editor start', {
    statute: channel.statute,
    retries,
    hasArguments: Boolean(channel.arguments && Object.keys(channel.arguments).length)
  })

  schemaError.value = ''
  schemaLoading.value = true

  if (!jsonEditorCtor) {
    const fromWindow = resolveJsonEditorCtor((window as { JSONEditor?: unknown }).JSONEditor)
    if (fromWindow) {
      jsonEditorCtor = fromWindow
      logger.debug('channels', 'json-editor ctor loaded from window')
    }
  }

  if (!jsonEditorCtor) {
    const module = await import('@json-editor/json-editor')
    const fromModule = resolveJsonEditorCtor(module)
    if (fromModule) {
      jsonEditorCtor = fromModule
      logger.debug('channels', 'json-editor ctor loaded from module', {
        moduleKeys: Object.keys(module as Record<string, unknown>)
      })
    } else {
      logger.warn('channels', 'json-editor ctor resolve failed', {
        moduleKeys: Object.keys(module as Record<string, unknown>)
      })
    }
  }

  try {
    logger.info('channels', 'ensure schema start', { statute: channel.statute })
    await props.ensureSchema(channel.statute)
    logger.info('channels', 'ensure schema done', { statute: channel.statute })
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error)
    schemaError.value = message
    logger.error('channels', 'ensure schema failed', { statute: channel.statute, error: message })
    emit('notify', { message: t('toast.connectionError', { message }), type: 'error' })
  } finally {
    schemaLoading.value = false
  }

  const schema = schemaOf(channel.statute)
  const ctor = jsonEditorCtor ?? (window as { JSONEditor?: new (...args: unknown[]) => unknown }).JSONEditor
  if (!schema || !ctor) {
    logger.warn('channels', 'init editor aborted (missing schema or ctor)', {
      statute: channel.statute,
      hasSchema: Boolean(schema),
      hasCtor: Boolean(ctor)
    })
    destroyEditor()
    if (!schemaError.value) {
      schemaError.value = t('channels.schemaMissing', { name: channel.statute })
    }
    return
  }

  await nextTick()
  if (!editorRef.value) {
    logger.warn('channels', 'editor ref not ready, retry', { retries })
    if (retries > 0) {
      setTimeout(() => void initEditor(retries - 1), 50)
    }
    return
  }

  const editorSchema = deepClone(schema) as {
    type?: string
    properties?: Record<string, unknown>
    defaultProperties?: string[]
  }

  if (
    editorSchema.type === 'object' &&
    editorSchema.properties &&
    !editorSchema.defaultProperties
  ) {
    editorSchema.defaultProperties = Object.keys(editorSchema.properties)
  }

  destroyEditor()
  const instance = new ctor(editorRef.value, {
    schema: editorSchema,
    theme: 'html',
    disable_edit_json: true,
    disable_properties: true,
    disable_collapse: true,
    show_errors: 'change',
    startval: channel.arguments ?? {}
  }) as {
    destroy: () => void
    on: (event: string, callback: () => void) => void
    getValue: () => Record<string, unknown>
    setValue?: (value: Record<string, unknown>) => void
  }

  instance.on('change', () => {
    if (editingChannel.value) {
      editingChannel.value.arguments = instance.getValue()
      syncRawFromArguments()
      logger.debug('channels', 'editor value changed', {
        statute: editingChannel.value.statute,
        keys: Object.keys(editingChannel.value.arguments ?? {})
      })
    }
  })

  jsonEditor = instance
  logger.info('channels', 'init editor success', {
    statute: channel.statute,
    schemaKeys: Object.keys(editorSchema.properties ?? {})
  })
}

watch(
  () => editingChannel.value?.statute,
  (nextStatute, prevStatute) => {
    if (!editorVisible.value || !editingChannel.value) {
      destroyEditor()
      return
    }

    logger.info('channels', 'watch statute changed', { prevStatute, nextStatute })

    if (prevStatute && nextStatute && prevStatute !== nextStatute) {
      editingChannel.value.arguments = {}
      schemaError.value = ''
      syncRawFromArguments()
    }

    void initEditor()
  }
)

watch(editorTab, (tab) => {
  if (tab === 'form' && editorVisible.value && editingChannel.value?.statute) {
    void initEditor()
  }
})

watch(editorVisible, (visible) => {
  if (!visible) {
    return
  }

  if (editingChannel.value?.statute) {
    syncRawFromArguments()
    void initEditor()
  }
})

onBeforeUnmount(destroyEditor)

const openCreate = () => {
  const nextId = props.channels.length
    ? Math.max(...props.channels.map((channel) => channel.channel_id)) + 1
    : 1

  editingChannel.value = {
    channel_id: nextId,
    enable: true,
    statute: '',
    description: '',
    arguments: {}
  }
  editingIndex.value = -1
  editorTab.value = 'form'
  schemaError.value = ''
  syncRawFromArguments()
  editorVisible.value = true
  logger.info('channels', 'open create channel', { channelId: nextId })
}

const openEdit = (target: Channel) => {
  if (!target) {
    return
  }

  const sourceIndex = props.channels.findIndex((channel) => channel.channel_id === target.channel_id)
  if (sourceIndex < 0) {
    return
  }

  editingChannel.value = deepClone(target)
  editingIndex.value = sourceIndex
  editorTab.value = 'form'
  schemaError.value = ''
  syncRawFromArguments()
  editorVisible.value = true
  logger.info('channels', 'open edit channel', {
    index: sourceIndex,
    channelId: target.channel_id,
    statute: target.statute
  })
}

const resetEditorState = () => {
  logger.info('channels', 'close editor')
  editingChannel.value = null
  editingIndex.value = -1
  schemaLoading.value = false
  schemaError.value = ''
  rawArgsText.value = '{}'
  rawArgsError.value = ''
  editorTab.value = 'form'
  destroyEditor()
}

const closeEditor = () => {
  editorVisible.value = false
}

const validate = (channel: Channel): string | null => {
  if (!Number.isFinite(channel.channel_id) || channel.channel_id <= 0) {
    return t('channels.validation.idRequired')
  }

  const duplicated = props.channels.some((item, index) => {
    if (editingIndex.value === index) {
      return false
    }
    return item.channel_id === channel.channel_id
  })

  if (duplicated) {
    return t('channels.validation.idDuplicate')
  }

  if (!channel.statute) {
    return t('channels.validation.protocolRequired')
  }

  return null
}

const save = () => {
  const channel = editingChannel.value
  if (!channel) {
    return
  }

  logger.info('channels', 'save channel start', {
    editIndex: editingIndex.value,
    channelId: channel.channel_id,
    statute: channel.statute
  })

  const error = validate(channel)
  if (error) {
    logger.warn('channels', 'save channel validation failed', { error })
    emit('notify', { message: t('toast.validationError', { message: error }), type: 'error' })
    return
  }

  const next = deepClone(props.channels)
  const snapshot = deepClone(channel)

  if (editingIndex.value >= 0) {
    next[editingIndex.value] = snapshot
  } else {
    next.push(snapshot)
  }

  emit('update:channels', next)
  logger.info('channels', 'save channel success', { total: next.length })
  editorVisible.value = false
}

const remove = async (target: Channel) => {
  if (!target) {
    return
  }

  const sourceIndex = props.channels.findIndex((channel) => channel.channel_id === target.channel_id)
  if (sourceIndex < 0) {
    return
  }

  try {
    await ElMessageBox.confirm(
      t('channels.confirmDelete', { id: target.channel_id }),
      t('common.delete'),
      {
        type: 'warning',
        confirmButtonText: t('common.delete'),
        cancelButtonText: t('common.cancel')
      }
    )
  } catch {
    logger.info('channels', 'delete channel canceled by user', { channelId: target.channel_id })
    return
  }

  const next = deepClone(props.channels)
  next.splice(sourceIndex, 1)
  emit('update:channels', next)
  logger.info('channels', 'delete channel success', { channelId: target.channel_id, total: next.length })
}
</script>

<template>
  <section class="page-shell">
    <el-card shadow="never" class="toolbar-card">
      <div class="toolbar-row">
        <el-input v-model="keyword" :placeholder="t('channels.keywordPlaceholder')" clearable class="toolbar-item">
          <template #prefix>
            <el-icon>
              <Search />
            </el-icon>
          </template>
        </el-input>

        <el-select v-model="protocolFilter" class="toolbar-item">
          <el-option :label="t('channels.protocolAll')" value="all" />
          <el-option v-for="protocol in protocolList" :key="protocol" :label="schemaOf(protocol)?.title ?? protocol"
            :value="protocol" />
        </el-select>

        <el-select v-model="statusFilter" class="toolbar-item">
          <el-option :label="t('channels.statusAll')" value="all" />
          <el-option :label="t('common.enabled')" value="enabled" />
          <el-option :label="t('common.disabled')" value="disabled" />
        </el-select>

        <el-button @click="resetFilters">{{ t('common.reset') }}</el-button>

        <el-button type="primary" class="create-btn" @click="openCreate">
          <el-icon>
            <Plus />
          </el-icon>
          <span>{{ t('channels.add') }}</span>
        </el-button>
      </div>

      <div class="summary-row">
        <el-tag effect="plain">{{ t('common.total') }}: {{ channels.length }}</el-tag>
        <el-tag effect="plain" type="success">{{ t('common.enabled') }}: {{ enabledCount }}</el-tag>
        <el-tag effect="plain" type="info">{{ t('common.disabled') }}: {{ disabledCount }}</el-tag>
        <el-tag effect="plain" type="warning">{{ t('common.matched') }}: {{ filteredChannels.length }}</el-tag>
      </div>
    </el-card>

    <el-card v-if="filteredChannels.length" shadow="never">
      <el-table :data="pagedChannels" stripe border>
        <el-table-column type="index" width="60" :index="channelRowIndex" />

        <el-table-column prop="channel_id" :label="t('channels.channelId')" min-width="110">
          <template #default="{ row }">
            <el-tag>#{{ row.channel_id }}</el-tag>
          </template>
        </el-table-column>

        <el-table-column :label="t('channels.description')" min-width="220">
          <template #default="{ row }">
            {{ channelName(row) }}
          </template>
        </el-table-column>

        <el-table-column :label="t('channels.protocol')" min-width="220">
          <template #default="{ row }">
            {{ schemaOf(row.statute)?.title ?? (row.statute || t('channels.noProtocol')) }}
          </template>
        </el-table-column>

        <el-table-column :label="t('channels.argsCount')" min-width="100" align="center">
          <template #default="{ row }">
            {{ channelArgumentCount(row) }}
          </template>
        </el-table-column>

        <el-table-column :label="t('channels.status')" min-width="110">
          <template #default="{ row }">
            <el-tag :type="row.enable ? 'success' : 'info'">
              {{ row.enable ? t('common.enabled') : t('common.disabled') }}
            </el-tag>
          </template>
        </el-table-column>

        <el-table-column :label="t('common.actions')" width="190" fixed="right">
          <template #default="{ row }">
            <el-button type="primary" link @click="openEdit(row)">{{ t('common.edit') }}</el-button>
            <el-button type="danger" link @click="remove(row)">{{ t('common.delete') }}</el-button>
          </template>
        </el-table-column>
      </el-table>

      <div class="table-pagination">
        <el-pagination
          v-model:current-page="channelPage"
          v-model:page-size="channelPageSize"
          :page-sizes="channelPageSizeOptions"
          :total="filteredChannels.length"
          layout="total, sizes, prev, pager, next, jumper"
        />
      </div>
    </el-card>

    <el-empty v-else :description="t('channels.empty')" />

    <el-dialog v-model="editorVisible" :title="editingIndex >= 0 ? t('channels.editTitle') : t('channels.createTitle')"
      width="1080px" top="3vh" destroy-on-close @closed="resetEditorState">
      <template v-if="editingChannel">
        <el-form label-position="top">
          <el-row :gutter="14">
            <el-col :xs="24" :md="12">
              <el-form-item :label="t('channels.channelId')">
                <el-input-number v-model="editingChannel.channel_id" :min="1" class="full-width" />
              </el-form-item>
            </el-col>

            <el-col :xs="24" :md="12">
              <el-form-item :label="t('channels.description')">
                <el-input v-model="editingChannel.description" />
              </el-form-item>
            </el-col>

            <el-col :xs="24" :md="12">
              <el-form-item :label="t('channels.protocol')">
                <el-select v-model="editingChannel.statute" class="full-width">
                  <el-option disabled :label="t('channels.protocolSelect')" value="" />
                  <el-option v-for="protocol in protocolList" :key="protocol"
                    :label="schemaOf(protocol)?.title ?? protocol" :value="protocol" />
                </el-select>
              </el-form-item>
            </el-col>

            <el-col :xs="24" :md="12">
              <el-form-item :label="t('channels.status')">
                <el-switch v-model="editingChannel.enable" :active-text="t('common.enabled')"
                  :inactive-text="t('common.disabled')" />
              </el-form-item>
            </el-col>
          </el-row>
        </el-form>

        <div v-if="editingChannel.statute" class="schema-editor">
          <el-tabs v-model="editorTab">
            <el-tab-pane :label="t('channels.editorForm')" name="form">
              <el-descriptions v-if="schemaFields.length" :title="t('channels.schemaOverview')" :column="3" border
                size="small" class="schema-desc">
                <template #extra>
                  <div class="schema-tools">
                    <el-button size="small" @click="applyDefaults">{{ t('channels.applyDefaults') }}</el-button>
                    <el-button size="small" @click="clearArguments">{{ t('channels.clearArgs') }}</el-button>
                  </div>
                </template>
                <el-descriptions-item :label="t('channels.fieldCount')">
                  <el-tag effect="plain" round>{{ schemaFields.length }}</el-tag>
                </el-descriptions-item>
                <el-descriptions-item :label="t('channels.requiredCount')">
                  <el-tag effect="plain" type="danger" round>
                    {{schemaFields.filter((f) => f.required).length}}
                  </el-tag>
                </el-descriptions-item>
                <el-descriptions-item :label="t('channels.defaultCount')">
                  <el-tag effect="plain" type="success" round>
                    {{schemaFields.filter((f) => f.hasDefault).length}}
                  </el-tag>
                </el-descriptions-item>
              </el-descriptions>

              <div v-if="!schemaFields.length && !schemaLoading" class="schema-tools schema-tools--standalone">
                <el-button size="small" @click="applyDefaults">{{ t('channels.applyDefaults') }}</el-button>
                <el-button size="small" @click="clearArguments">{{ t('channels.clearArgs') }}</el-button>
              </div>

              <el-form v-if="schemaFields.length" label-position="top" class="schema-native-form">
                <el-row :gutter="16">
                  <el-col v-for="field in schemaFields" :key="field.key" :xs="24" :sm="12" :md="12">
                    <el-form-item :required="field.required">
                      <template #label>
                        <span>{{ field.key }}</span>
                        <el-tag size="small" effect="plain" class="field-type-tag">{{ field.type }}</el-tag>
                      </template>

                      <!-- string with enum → select -->
                      <el-select v-if="field.enumValues.length" :model-value="(getArgValue(field.key) as string) ?? ''"
                        :placeholder="field.hasDefault ? field.defaultText : field.key" class="full-width"
                        @update:model-value="setArgValue(field.key, $event)">
                        <el-option v-for="opt in field.enumValues" :key="opt" :label="opt" :value="opt" />
                      </el-select>

                      <!-- integer / number → input-number -->
                      <el-input-number v-else-if="field.type === 'integer' || field.type === 'number'"
                        :model-value="(getArgValue(field.key) as number | undefined) ?? (field.defaultValue as number | undefined)"
                        :placeholder="field.hasDefault ? field.defaultText : field.key" class="full-width"
                        controls-position="right" @update:model-value="setArgValue(field.key, $event)" />

                      <!-- boolean → switch -->
                      <el-switch v-else-if="field.type === 'boolean'"
                        :model-value="(getArgValue(field.key) as boolean | undefined) ?? (field.defaultValue as boolean | undefined) ?? false"
                        @update:model-value="setArgValue(field.key, $event)" />

                      <!-- object / array → json textarea -->
                      <el-input v-else-if="field.type === 'object' || field.type === 'array'" type="textarea"
                        :autosize="{ minRows: 2, maxRows: 8 }" :model-value="getArgJsonText(field.key)"
                        @update:model-value="setArgJsonText(field.key, $event)" />

                      <!-- string / fallback → input -->
                      <el-input v-else :model-value="(getArgValue(field.key) as string) ?? ''"
                        :placeholder="field.hasDefault ? field.defaultText : field.key"
                        @update:model-value="setArgValue(field.key, $event)" />
                    </el-form-item>
                  </el-col>
                </el-row>
              </el-form>

              <el-alert v-if="schemaLoading" :title="t('common.loading')" type="info" show-icon :closable="false" />
              <div v-else-if="schemaOf(editingChannel.statute)" ref="editorRef" class="schema-box" style="display:none">
              </div>
              <el-alert v-else :title="schemaError || t('channels.schemaMissing', { name: editingChannel.statute })"
                type="error" :closable="false" show-icon />
            </el-tab-pane>

            <el-tab-pane :label="t('channels.editorJson')" name="json">
              <div class="raw-json-editor">
                <div class="raw-header">
                  <h5>{{ t('channels.rawJsonTitle') }}</h5>
                  <el-button @click="applyRawJson">{{ t('channels.applyRawJson') }}</el-button>
                </div>
                <el-input v-model="rawArgsText" type="textarea" :rows="12" />
                <div v-if="rawArgsError" class="raw-error">{{ rawArgsError }}</div>
              </div>
            </el-tab-pane>
          </el-tabs>
        </div>
      </template>

      <template #footer>
        <el-button @click="closeEditor">{{ t('common.cancel') }}</el-button>
        <el-button type="primary" @click="save">
          {{ editingIndex >= 0 ? t('common.update') : t('common.create') }}
        </el-button>
      </template>
    </el-dialog>
  </section>
</template>

<style scoped>
.page-shell {
  display: grid;
  gap: 12px;
}

.toolbar-card {
  border-style: dashed;
}

.toolbar-row {
  display: grid;
  grid-template-columns: 1.5fr 1fr 1fr auto auto;
  gap: 10px;
  align-items: center;
}

.toolbar-item {
  width: 100%;
}

.create-btn {
  justify-self: end;
}

.summary-row {
  margin-top: 10px;
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.table-pagination {
  margin-top: 12px;
  display: flex;
  justify-content: flex-end;
}

.full-width {
  width: 100%;
}

.schema-editor {
  margin-top: 10px;
}

.schema-desc {
  margin-bottom: 14px;
}

.schema-tools {
  display: flex;
  gap: 8px;
}

.schema-tools--standalone {
  margin-bottom: 14px;
}

.schema-native-form {
  margin-bottom: 14px;
  border: 1px solid var(--el-border-color-lighter);
  border-radius: 10px;
  padding: 18px 16px 4px;
  background: var(--el-fill-color-blank);
}

.field-type-tag {
  margin-left: 6px;
  vertical-align: middle;
}

.schema-box {
  border: 1px solid var(--el-border-color);
  border-radius: 10px;
  padding: 10px;
  background: #ffffff;
  transition: box-shadow 0.25s;
}

.schema-box:hover {
  box-shadow: 0 2px 12px rgba(0, 0, 0, 0.05);
}

.raw-json-editor {
  border: 1px solid var(--el-border-color-light);
  border-radius: 10px;
  padding: 10px;
  display: grid;
  gap: 8px;
}

.raw-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.raw-header h5 {
  margin: 0;
  font-size: 14px;
}

.raw-error {
  color: var(--el-color-danger);
  font-size: 12px;
}

@media (max-width: 1200px) {
  .toolbar-row {
    grid-template-columns: 1fr 1fr;
  }

  .create-btn {
    justify-self: stretch;
  }
}

@media (max-width: 960px) {
  .raw-header {
    flex-direction: column;
    align-items: flex-start;
  }
}
</style>
