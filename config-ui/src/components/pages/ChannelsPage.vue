<script setup lang="ts">
import { nextTick, onBeforeUnmount, ref, watch } from 'vue'
import { useI18n } from '../../composables/useI18n'
import type { Channel, ToastType } from '../../types/config'
import { deepClone } from '../../utils/deepClone'

const props = defineProps<{
  channels: Channel[]
  protocolList: string[]
  schemas: Record<string, Record<string, unknown>>
}>()

const emit = defineEmits<{
  (e: 'update:channels', value: Channel[]): void
  (e: 'notify', payload: { message: string; type?: ToastType }): void
}>()

const { t } = useI18n()

const editingChannel = ref<Channel | null>(null)
const editingIndex = ref(-1)
const editorRef = ref<HTMLElement | null>(null)

let jsonEditor: {
  destroy: () => void
  on: (event: string, callback: () => void) => void
  getValue: () => Record<string, unknown>
} | null = null
let jsonEditorCtor: (new (...args: unknown[]) => unknown) | null = null

const destroyEditor = () => {
  if (jsonEditor) {
    jsonEditor.destroy()
    jsonEditor = null
  }
}

const initEditor = async (retries = 5) => {
  await nextTick()
  const channel = editingChannel.value
  if (!channel || !channel.statute) {
    return
  }

  // editorRef may not be in DOM yet due to Transition animation.
  // Retry a few times with a short delay.
  if (!editorRef.value) {
    if (retries > 0) {
      setTimeout(() => void initEditor(retries - 1), 50)
    }
    return
  }

  if (!(window as { JSONEditor?: unknown }).JSONEditor) {
    const module = await import('@json-editor/json-editor')
    const ctor = (module.default ?? module) as unknown
    if (typeof ctor === 'function') {
      jsonEditorCtor = ctor as new (...args: unknown[]) => unknown
    }
  } else if (!jsonEditorCtor) {
    const ctor = (window as { JSONEditor?: unknown }).JSONEditor
    if (typeof ctor === 'function') {
      jsonEditorCtor = ctor as new (...args: unknown[]) => unknown
    }
  }

  const schema = props.schemas[channel.statute]
  const ctor = jsonEditorCtor ?? (window as { JSONEditor?: new (...args: unknown[]) => unknown }).JSONEditor
  if (!schema || !ctor) {
    return
  }

  destroyEditor()
  const instance = new ctor(editorRef.value, {
    schema,
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
  }

  instance.on('change', () => {
    if (editingChannel.value) {
      editingChannel.value.arguments = instance.getValue()
    }
  })

  jsonEditor = instance
}

watch(
  () => editingChannel.value?.statute,
  (nextStatute, prevStatute) => {
    if (!editingChannel.value) {
      destroyEditor()
      return
    }

    // Keep existing arguments when protocol is unchanged.
    // Reset arguments only when user switches to a different protocol.
    if (prevStatute && nextStatute && prevStatute !== nextStatute) {
      editingChannel.value.arguments = {}
    }

    void initEditor()
  }
)

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
}

const openEdit = (index: number) => {
  const target = props.channels[index]
  if (!target) {
    return
  }

  editingChannel.value = deepClone(target)
  editingIndex.value = index

  // Explicitly init editor since the watcher may not fire
  // when going from null -> cloned object with existing statute
  if (target.statute && props.schemas[target.statute]) {
    void initEditor()
  }
}

const closeEditor = () => {
  editingChannel.value = null
  editingIndex.value = -1
  destroyEditor()
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

  const error = validate(channel)
  if (error) {
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
  closeEditor()
}

const remove = (index: number) => {
  const target = props.channels[index]
  if (!target) {
    return
  }

  if (!window.confirm(t('channels.confirmDelete', { id: target.channel_id }))) {
    return
  }

  const next = deepClone(props.channels)
  next.splice(index, 1)
  emit('update:channels', next)
}

const channelName = (channel: Channel) => {
  return channel.description?.trim() || t('channels.defaultName', { id: channel.channel_id })
}
</script>

<template>
  <section class="page-shell">
    <div class="page-header">
      <div>
        <h1>{{ t('channels.title') }}</h1>
        <p>{{ t('channels.desc') }}</p>
      </div>
      <button class="btn btn-primary" @click="openCreate">+ {{ t('channels.add') }}</button>
    </div>

    <div v-if="channels.length" class="card-grid">
      <article v-for="(channel, index) in channels" :key="channel.channel_id" class="channel-card"
        :class="{ disabled: !channel.enable }">
        <header>
          <span class="chip id">#{{ channel.channel_id }}</span>
          <span class="chip" :class="channel.enable ? 'on' : 'off'">
            {{ channel.enable ? t('common.enabled') : t('common.disabled') }}
          </span>
        </header>
        <h3>{{ channelName(channel) }}</h3>
        <p class="protocol">{{ props.schemas[channel.statute]?.title ?? (channel.statute || t('channels.noProtocol')) }}
        </p>
        <div class="actions">
          <button class="btn btn-light" @click="openEdit(index)">{{ t('common.edit') }}</button>
          <button class="btn btn-danger" @click="remove(index)">{{ t('common.delete') }}</button>
        </div>
      </article>
    </div>
    <div v-else class="empty-state">{{ t('channels.empty') }}</div>

    <Transition name="modal">
      <div v-if="editingChannel" class="modal-mask" @click.self="closeEditor">
        <section class="editor-panel modal-panel">
          <h2>{{ editingIndex >= 0 ? t('channels.editTitle') : t('channels.createTitle') }}</h2>

          <div class="form-grid">
            <label>
              <span>{{ t('channels.channelId') }}</span>
              <input v-model.number="editingChannel.channel_id" type="number" min="1" />
            </label>

            <label>
              <span>{{ t('channels.description') }}</span>
              <input v-model="editingChannel.description" type="text" />
            </label>

            <label>
              <span>{{ t('channels.protocol') }}</span>
              <select v-model="editingChannel.statute">
                <option disabled value="">{{ t('channels.protocolSelect') }}</option>
                <option v-for="protocol in protocolList" :key="protocol" :value="protocol">
                  {{ props.schemas[protocol]?.title ?? protocol }}
                </option>
              </select>
            </label>

            <label class="switch-field">
              <span>{{ t('channels.status') }}</span>
              <label class="switch">
                <input v-model="editingChannel.enable" type="checkbox" />
                <span></span>
              </label>
            </label>
          </div>

          <div v-if="editingChannel.statute && props.schemas[editingChannel.statute]" class="schema-editor">
            <h4>{{ t('channels.protocolArgs') }}</h4>
            <div ref="editorRef" class="schema-box"></div>
          </div>
          <div v-else-if="editingChannel.statute" class="schema-missing">
            {{ t('channels.schemaMissing', { name: editingChannel.statute }) }}
          </div>

          <div class="actions">
            <button class="btn btn-light" @click="closeEditor">{{ t('common.cancel') }}</button>
            <button class="btn btn-primary" @click="save">
              {{ editingIndex >= 0 ? t('common.update') : t('common.create') }}
            </button>
          </div>
        </section>
      </div>
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

.card-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
  gap: 12px;
}

.channel-card {
  background: #ffffff;
  border: 1px solid var(--border);
  border-radius: 14px;
  padding: 12px;
  display: grid;
  gap: 8px;
}

.channel-card.disabled {
  opacity: 0.64;
}

.channel-card header {
  display: flex;
  justify-content: space-between;
}

.channel-card h3 {
  margin: 0;
  font-size: 16px;
}

.protocol {
  margin: 0;
  color: var(--text-secondary);
  font-size: 13px;
}

.chip {
  border-radius: 999px;
  padding: 2px 8px;
  font-size: 12px;
  font-weight: 600;
}

.chip.id {
  color: #1e3a8a;
  background: #dbeafe;
}

.chip.on {
  color: #166534;
  background: #dcfce7;
}

.chip.off {
  color: #334155;
  background: #e2e8f0;
}

.actions {
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

.modal-mask {
  position: fixed;
  inset: 0;
  z-index: 1100;
  background: rgba(15, 23, 42, 0.46);
  display: grid;
  place-items: center;
  padding: 20px;
}

.modal-panel {
  width: min(900px, 100%);
  max-height: calc(100vh - 40px);
  overflow: auto;
  border-radius: 16px;
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

.switch-field {
  align-content: end;
}

.switch {
  width: 52px;
  height: 30px;
  display: inline-flex;
  position: relative;
}

.switch input {
  display: none;
}

.switch span {
  width: 52px;
  height: 30px;
  border-radius: 999px;
  background: #cbd5e1;
  position: relative;
  transition: background 0.2s ease;
}

.switch span::after {
  content: '';
  position: absolute;
  width: 24px;
  height: 24px;
  border-radius: 50%;
  left: 3px;
  top: 3px;
  background: #ffffff;
  transition: transform 0.2s ease;
}

.switch input:checked+span {
  background: #3b82f6;
}

.switch input:checked+span::after {
  transform: translateX(22px);
}

.schema-editor {
  display: grid;
  gap: 8px;
}

.schema-editor h4 {
  margin: 0;
  font-size: 14px;
  color: var(--text-secondary);
}

.schema-box {
  border: 1px solid var(--border);
  border-radius: 10px;
  padding: 10px;
  min-height: 120px;
  background: #fafcff;
}

.schema-missing {
  border: 1px dashed #fca5a5;
  border-radius: 10px;
  padding: 10px;
  color: #991b1b;
  background: #fef2f2;
  font-size: 13px;
}

.modal-enter-active,
.modal-leave-active {
  transition: all 0.2s ease;
}

.modal-enter-from,
.modal-leave-to {
  opacity: 0;
}

@media (max-width: 768px) {
  .form-grid {
    grid-template-columns: 1fr;
  }
}
</style>
