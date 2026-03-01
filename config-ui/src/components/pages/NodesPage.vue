<script setup lang="ts">
import { computed, ref } from 'vue'
import { useI18n } from '../../composables/useI18n'
import type { Channel, NodeItem, ToastType } from '../../types/config'
import { deepClone } from '../../utils/deepClone'

interface NodeGroup {
  channelId: number
  channel?: Channel
  items: Array<{ node: NodeItem; index: number }>
}

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

const groupedNodes = computed<NodeGroup[]>(() => {
  const map = new Map<number, NodeGroup>()

  props.nodes.forEach((node, index) => {
    if (!map.has(node.channel_id)) {
      map.set(node.channel_id, {
        channelId: node.channel_id,
        channel: props.channels.find((channel) => channel.channel_id === node.channel_id),
        items: []
      })
    }

    map.get(node.channel_id)?.items.push({ node, index })
  })

  return [...map.values()].sort((a, b) => a.channelId - b.channelId)
})

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
}

const openEdit = (index: number) => {
  const target = props.nodes[index]
  if (!target) {
    return
  }

  editingNode.value = deepClone(target)
  editingIndex.value = index
}

const closeEditor = () => {
  editingNode.value = null
  editingIndex.value = -1
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
  closeEditor()
}

const remove = (index: number) => {
  const target = props.nodes[index]
  if (!target) {
    return
  }

  if (!window.confirm(t('nodes.confirmDelete', { name: target.alias }))) {
    return
  }

  const next = deepClone(props.nodes)
  next.splice(index, 1)
  emit('update:nodes', next)
}
</script>

<template>
  <section class="page-shell">
    <div class="page-header">
      <div>
        <h1>{{ t('nodes.title') }}</h1>
        <p>{{ t('nodes.desc') }}</p>
      </div>
      <button class="btn btn-primary" @click="openCreate">+ {{ t('nodes.add') }}</button>
    </div>

    <div v-if="groupedNodes.length" class="groups">
      <section v-for="group in groupedNodes" :key="group.channelId" class="group-card">
        <header>
          <h3>{{ t('nodes.groupTitle', { id: group.channelId }) }}</h3>
          <span class="protocol">{{ group.channel?.description || group.channel?.statute || '-' }}</span>
        </header>

        <table>
          <thead>
            <tr>
              <th>{{ t('nodes.globalId') }}</th>
              <th>{{ t('nodes.deviceId') }}</th>
              <th>{{ t('nodes.alias') }}</th>
              <th>{{ t('common.actions') }}</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="item in group.items" :key="item.node.global_id">
              <td><code>{{ item.node.global_id }}</code></td>
              <td><code>{{ item.node.id }}</code></td>
              <td>{{ item.node.alias }}</td>
              <td class="table-actions">
                <button class="btn btn-light" @click="openEdit(item.index)">{{ t('common.edit') }}</button>
                <button class="btn btn-danger" @click="remove(item.index)">{{ t('common.delete') }}</button>
              </td>
            </tr>
          </tbody>
        </table>
      </section>
    </div>
    <div v-else class="empty-state">{{ t('nodes.empty') }}</div>

    <Transition name="panel">
      <section v-if="editingNode" class="editor-panel">
        <h2>{{ editingIndex >= 0 ? t('nodes.editTitle') : t('nodes.createTitle') }}</h2>

        <div class="form-grid">
          <label>
            <span>{{ t('nodes.globalId') }}</span>
            <input v-model.number="editingNode.global_id" type="number" min="1" />
          </label>

          <label>
            <span>{{ t('nodes.channel') }}</span>
            <select v-model.number="editingNode.channel_id">
              <option v-for="channel in channels" :key="channel.channel_id" :value="channel.channel_id">
                #{{ channel.channel_id }} - {{ channel.description || channel.statute || '-' }}
              </option>
            </select>
          </label>

          <label>
            <span>{{ t('nodes.nodeIdInChannel') }}</span>
            <input v-model.number="editingNode.id" type="number" min="1" />
          </label>

          <label>
            <span>{{ t('nodes.alias') }}</span>
            <input v-model="editingNode.alias" type="text" />
          </label>
        </div>

        <div class="actions">
          <button class="btn btn-light" @click="closeEditor">{{ t('common.cancel') }}</button>
          <button class="btn btn-primary" @click="save">
            {{ editingIndex >= 0 ? t('common.update') : t('common.create') }}
          </button>
        </div>
      </section>
    </Transition>
  </section>
</template>

<style scoped>
.page-shell {
  display: grid;
  gap: 16px;
}

.page-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 14px;
}

.page-header h1 {
  margin: 0;
  font-size: 24px;
}

.page-header p {
  margin-top: 8px;
  color: var(--text-secondary);
  max-width: 760px;
  line-height: 1.6;
  font-size: 14px;
}

.groups {
  display: grid;
  gap: 14px;
}

.group-card {
  background: #ffffff;
  border: 1px solid var(--border);
  border-radius: 14px;
  overflow: hidden;
}

.group-card header {
  min-height: 44px;
  padding: 0 12px;
  display: flex;
  justify-content: space-between;
  align-items: center;
  border-bottom: 1px solid var(--border);
  background: #f8fbff;
}

.group-card h3 {
  margin: 0;
  font-size: 15px;
}

.protocol {
  color: var(--text-secondary);
  font-size: 12px;
}

table {
  width: 100%;
  border-collapse: collapse;
}

th,
td {
  text-align: left;
  padding: 10px 12px;
  border-bottom: 1px solid #edf2f7;
  font-size: 13px;
}

th {
  color: var(--text-secondary);
  background: #ffffff;
  font-size: 12px;
  text-transform: uppercase;
  letter-spacing: 0.04em;
}

tr:last-child td {
  border-bottom: none;
}

code {
  background: #eff6ff;
  color: #1e3a8a;
  border-radius: 6px;
  padding: 2px 6px;
}

.table-actions {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.empty-state {
  padding: 24px;
  border: 1px dashed var(--border);
  border-radius: 14px;
  color: var(--text-secondary);
  background: #ffffff;
}

.editor-panel {
  background: #ffffff;
  border: 1px solid var(--border);
  border-radius: 14px;
  padding: 16px;
  display: grid;
  gap: 14px;
}

.editor-panel h2 {
  margin: 0;
  font-size: 18px;
}

.form-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 10px;
}

.form-grid label {
  display: grid;
  gap: 6px;
}

.form-grid span {
  font-size: 13px;
  color: var(--text-secondary);
}

input,
select {
  height: 38px;
  border: 1px solid var(--border);
  border-radius: 10px;
  padding: 0 10px;
  font-size: 14px;
}

input:focus,
select:focus {
  outline: none;
  border-color: #60a5fa;
  box-shadow: 0 0 0 3px rgba(96, 165, 250, 0.2);
}

.actions {
  display: flex;
  gap: 8px;
}

.panel-enter-active,
.panel-leave-active {
  transition: all 0.2s ease;
}

.panel-enter-from,
.panel-leave-to {
  opacity: 0;
  transform: translateY(8px);
}

@media (max-width: 768px) {
  .form-grid {
    grid-template-columns: 1fr;
  }
}
</style>
