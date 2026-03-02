<script setup lang="ts">
import { computed, ref } from 'vue'
import { ElMessageBox } from 'element-plus'
import { useI18n } from '../../composables/useI18n'
import type { Channel, NodeItem, ToastType } from '../../types/config'
import { deepClone } from '../../utils/deepClone'

const props = defineProps<{
  nodes: NodeItem[]
  channels: Channel[]
}>()

const emit = defineEmits<{
  (e: 'update:nodes', value: NodeItem[]): void
  (e: 'notify', payload: { message: string; type?: ToastType }): void
}>()

const { t } = useI18n()

const editingNode = ref<NodeItem | null>(null)
const editingIndex = ref(-1)
const editorVisible = ref(false)

const keyword = ref('')
const channelFilter = ref('all')

const channelMap = computed(() => {
  const map = new Map<number, Channel>()
  props.channels.forEach((channel) => map.set(channel.channel_id, channel))
  return map
})

const filteredNodes = computed(() => {
  const key = keyword.value.trim().toLowerCase()

  return props.nodes.filter((node) => {
    if (channelFilter.value !== 'all' && node.channel_id !== Number(channelFilter.value)) {
      return false
    }

    if (!key) {
      return true
    }

    const channelName = channelMap.value.get(node.channel_id)?.description ?? ''
    const combined = [
      String(node.global_id),
      String(node.channel_id),
      node.alias,
      channelName
    ].join(' ').toLowerCase()

    return combined.includes(key)
  })
})

const channelCoverage = computed(() => new Set(props.nodes.map((node) => node.channel_id)).size)

const resetFilters = () => {
  keyword.value = ''
  channelFilter.value = 'all'
}

const openCreate = () => {
  const firstChannel = props.channels[0]
  if (!firstChannel) {
    emit('notify', { message: t('toast.validationError', { message: t('nodes.validation.channelRequired') }), type: 'error' })
    return
  }

  const nextGlobalId = props.nodes.length
    ? Math.max(...props.nodes.map((node) => node.global_id)) + 1
    : 1

  editingNode.value = {
    global_id: nextGlobalId,
    channel_id: firstChannel.channel_id,
    id: 1,
    alias: ''
  }
  editingIndex.value = -1
  editorVisible.value = true
}

const openEdit = (index: number) => {
  const target = filteredNodes.value[index]
  if (!target) {
    return
  }

  const sourceIndex = props.nodes.findIndex((node) => node.global_id === target.global_id)
  if (sourceIndex < 0) {
    return
  }

  editingNode.value = deepClone(target)
  editingIndex.value = sourceIndex
  editorVisible.value = true
}

const resetEditor = () => {
  editingNode.value = null
  editingIndex.value = -1
}

const closeEditor = () => {
  editorVisible.value = false
}

const validate = (node: NodeItem): string | null => {
  if (!Number.isFinite(node.global_id) || node.global_id <= 0) {
    return t('nodes.validation.globalIdRequired')
  }

  const duplicated = props.nodes.some((item, index) => {
    if (editingIndex.value === index) {
      return false
    }
    return item.global_id === node.global_id
  })

  if (duplicated) {
    return t('nodes.validation.globalIdDuplicate')
  }

  if (!node.alias.trim()) {
    return t('nodes.validation.aliasRequired')
  }

  const channelExists = props.channels.some((channel) => channel.channel_id === node.channel_id)
  if (!channelExists) {
    return t('nodes.validation.channelRequired')
  }

  return null
}

const save = () => {
  const node = editingNode.value
  if (!node) {
    return
  }

  const error = validate(node)
  if (error) {
    emit('notify', { message: t('toast.validationError', { message: error }), type: 'error' })
    return
  }

  const next = deepClone(props.nodes)
  const snapshot = deepClone(node)

  if (editingIndex.value >= 0) {
    next[editingIndex.value] = snapshot
  } else {
    next.push(snapshot)
  }

  emit('update:nodes', next)
  editorVisible.value = false
}

const remove = async (index: number) => {
  const target = filteredNodes.value[index]
  if (!target) {
    return
  }

  try {
    await ElMessageBox.confirm(
      t('nodes.confirmDelete', { name: target.alias }),
      t('common.delete'),
      {
        type: 'warning',
        confirmButtonText: t('common.delete'),
        cancelButtonText: t('common.cancel')
      }
    )
  } catch {
    return
  }

  const sourceIndex = props.nodes.findIndex((node) => node.global_id === target.global_id)
  if (sourceIndex < 0) {
    return
  }

  const next = deepClone(props.nodes)
  next.splice(sourceIndex, 1)
  emit('update:nodes', next)
}

const channelLabel = (channelId: number) => {
  const channel = channelMap.value.get(channelId)
  if (!channel) {
    return '#-'
  }
  return `#${channel.channel_id} ${channel.description || channel.statute || ''}`
}
</script>

<template>
  <section class="page-shell">
    <el-card shadow="never" class="toolbar-card">
      <div class="toolbar-row">
        <el-input
          v-model="keyword"
          :placeholder="t('nodes.keywordPlaceholder')"
          clearable
          class="toolbar-item"
        >
          <template #prefix>
            <el-icon><Search /></el-icon>
          </template>
        </el-input>

        <el-select v-model="channelFilter" class="toolbar-item">
          <el-option :label="t('nodes.channelAll')" value="all" />
          <el-option
            v-for="channel in channels"
            :key="channel.channel_id"
            :label="`#${channel.channel_id} ${channel.description || channel.statute || '-'}`"
            :value="String(channel.channel_id)"
          />
        </el-select>

        <el-button @click="resetFilters">{{ t('common.reset') }}</el-button>

        <el-button type="primary" class="create-btn" @click="openCreate">
          <el-icon><Plus /></el-icon>
          <span>{{ t('nodes.add') }}</span>
        </el-button>
      </div>

      <div class="summary-row">
        <el-tag effect="plain">{{ t('common.total') }}: {{ nodes.length }}</el-tag>
        <el-tag effect="plain" type="success">{{ t('nodes.coverage') }}: {{ channelCoverage }}</el-tag>
        <el-tag effect="plain" type="warning">{{ t('common.matched') }}: {{ filteredNodes.length }}</el-tag>
      </div>
    </el-card>

    <el-card v-if="filteredNodes.length" shadow="never">
      <el-table :data="filteredNodes" stripe border>
        <el-table-column type="index" width="60" />

        <el-table-column prop="global_id" :label="t('nodes.globalId')" min-width="120">
          <template #default="{ row }">
            <el-tag>#{{ row.global_id }}</el-tag>
          </template>
        </el-table-column>

        <el-table-column prop="alias" :label="t('nodes.alias')" min-width="220" />

        <el-table-column :label="t('nodes.channelName')" min-width="240">
          <template #default="{ row }">
            {{ channelLabel(row.channel_id) }}
          </template>
        </el-table-column>

        <el-table-column prop="id" :label="t('nodes.deviceId')" min-width="120" align="center" />

        <el-table-column :label="t('common.actions')" width="180" fixed="right">
          <template #default="{ $index }">
            <el-button type="primary" link @click="openEdit($index)">{{ t('common.edit') }}</el-button>
            <el-button type="danger" link @click="remove($index)">{{ t('common.delete') }}</el-button>
          </template>
        </el-table-column>
      </el-table>
    </el-card>

    <el-empty v-else :description="t('nodes.empty')" />

    <el-dialog
      v-model="editorVisible"
      :title="editingIndex >= 0 ? t('nodes.editTitle') : t('nodes.createTitle')"
      width="760px"
      destroy-on-close
      @closed="resetEditor"
    >
      <template v-if="editingNode">
        <el-form label-position="top">
          <el-row :gutter="14">
            <el-col :xs="24" :md="12">
              <el-form-item :label="t('nodes.globalId')">
                <el-input-number v-model="editingNode.global_id" :min="1" class="full-width" />
              </el-form-item>
            </el-col>

            <el-col :xs="24" :md="12">
              <el-form-item :label="t('nodes.channel')">
                <el-select v-model="editingNode.channel_id" class="full-width">
                  <el-option
                    v-for="channel in channels"
                    :key="channel.channel_id"
                    :label="`#${channel.channel_id} - ${channel.description || channel.statute || '-'}`"
                    :value="channel.channel_id"
                  />
                </el-select>
              </el-form-item>
            </el-col>

            <el-col :xs="24" :md="12">
              <el-form-item :label="t('nodes.nodeIdInChannel')">
                <el-input-number v-model="editingNode.id" :min="1" class="full-width" />
              </el-form-item>
            </el-col>

            <el-col :xs="24" :md="12">
              <el-form-item :label="t('nodes.alias')">
                <el-input v-model="editingNode.alias" />
              </el-form-item>
            </el-col>
          </el-row>
        </el-form>
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
  grid-template-columns: 1.5fr 1fr auto auto;
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

.full-width {
  width: 100%;
}

@media (max-width: 1200px) {
  .toolbar-row {
    grid-template-columns: 1fr 1fr;
  }

  .create-btn {
    justify-self: stretch;
  }
}
</style>
