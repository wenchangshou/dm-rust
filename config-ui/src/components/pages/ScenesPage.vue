<script setup lang="ts">
import { ref } from 'vue'
import { useI18n } from '../../composables/useI18n'
import type { NodeItem, Scene, SceneNode, ToastType } from '../../types/config'
import { deepClone } from '../../utils/deepClone'

const props = defineProps<{
  scenes: Scene[]
  nodes: NodeItem[]
}>()

const emit = defineEmits<{
  (e: 'update:scenes', value: Scene[]): void
  (e: 'notify', payload: { message: string; type?: ToastType }): void
}>()

const { t } = useI18n()

const editingScene = ref<Scene | null>(null)
const editingIndex = ref(-1)

const openCreate = () => {
  editingScene.value = {
    name: '',
    nodes: []
  }
  editingIndex.value = -1
}

const openEdit = (index: number) => {
  const target = props.scenes[index]
  if (!target) {
    return
  }

  editingScene.value = deepClone(target)
  editingIndex.value = index
}

const closeEditor = () => {
  editingScene.value = null
  editingIndex.value = -1
}

const addStep = () => {
  if (!editingScene.value) {
    return
  }

  const firstNode = props.nodes[0]
  if (!firstNode) {
    emit('notify', { message: t('toast.validationError', { message: t('scenes.validation.noNode') }), type: 'error' })
    return
  }

  editingScene.value.nodes.push({
    id: firstNode.global_id,
    value: 0,
    delay: 0
  })
}

const removeStep = (index: number) => {
  editingScene.value?.nodes.splice(index, 1)
}

const validate = (scene: Scene): string | null => {
  if (!scene.name.trim()) {
    return t('scenes.validation.nameRequired')
  }

  return null
}

const save = () => {
  const scene = editingScene.value
  if (!scene) {
    return
  }

  const error = validate(scene)
  if (error) {
    emit('notify', { message: t('toast.validationError', { message: error }), type: 'error' })
    return
  }

  const next = deepClone(props.scenes)
  const snapshot = deepClone(scene)

  if (editingIndex.value >= 0) {
    next[editingIndex.value] = snapshot
  } else {
    next.push(snapshot)
  }

  emit('update:scenes', next)
  closeEditor()
}

const remove = (index: number) => {
  const target = props.scenes[index]
  if (!target) {
    return
  }

  if (!window.confirm(t('scenes.confirmDelete', { name: target.name }))) {
    return
  }

  const next = deepClone(props.scenes)
  next.splice(index, 1)
  emit('update:scenes', next)
}

const nodeLabel = (globalId: number) => {
  const node = props.nodes.find((item) => item.global_id === globalId)
  return node ? node.alias : `#${globalId}`
}

const normalizeSceneNode = (node: SceneNode) => {
  node.value = Number.isFinite(node.value) ? node.value : 0
  node.delay = Number.isFinite(node.delay) ? node.delay : 0
}
</script>

<template>
  <section class="page-shell">
    <div class="page-header">
      <div>
        <h1>{{ t('scenes.title') }}</h1>
        <p>{{ t('scenes.desc') }}</p>
      </div>
      <button class="btn btn-primary" @click="openCreate">+ {{ t('scenes.add') }}</button>
    </div>

    <div v-if="scenes.length" class="scene-list">
      <article v-for="(scene, index) in scenes" :key="index" class="scene-card">
        <header>
          <h3>{{ scene.name }}</h3>
          <span>{{ t('scenes.stepCount', { count: scene.nodes.length }) }}</span>
        </header>
        <div class="chips">
          <span v-for="(step, stepIndex) in scene.nodes.slice(0, 5)" :key="stepIndex" class="chip">
            {{ nodeLabel(step.id) }} -> {{ step.value }}
          </span>
          <span v-if="scene.nodes.length > 5" class="chip more">
            {{ t('scenes.more', { count: scene.nodes.length - 5 }) }}
          </span>
        </div>
        <div class="actions">
          <button class="btn btn-light" @click="openEdit(index)">{{ t('common.edit') }}</button>
          <button class="btn btn-danger" @click="remove(index)">{{ t('common.delete') }}</button>
        </div>
      </article>
    </div>
    <div v-else class="empty-state">{{ t('scenes.empty') }}</div>

    <Transition name="panel">
      <section v-if="editingScene" class="editor-panel">
        <h2>{{ editingIndex >= 0 ? t('scenes.editTitle') : t('scenes.createTitle') }}</h2>

        <label class="field">
          <span>{{ t('scenes.sceneName') }}</span>
          <input v-model="editingScene.name" type="text" />
        </label>

        <div class="step-header">
          <h4>{{ t('scenes.steps') }}</h4>
          <button class="btn btn-light" @click="addStep">+ {{ t('scenes.addStep') }}</button>
        </div>

        <table v-if="editingScene.nodes.length">
          <thead>
            <tr>
              <th>{{ t('scenes.device') }}</th>
              <th>{{ t('scenes.value') }}</th>
              <th>{{ t('scenes.delay') }}</th>
              <th>{{ t('common.actions') }}</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="(step, index) in editingScene.nodes" :key="index">
              <td>
                <select v-model.number="step.id" @change="normalizeSceneNode(step)">
                  <option v-for="node in nodes" :key="node.global_id" :value="node.global_id">
                    {{ node.alias }} (#{{ node.global_id }})
                  </option>
                </select>
              </td>
              <td>
                <input v-model.number="step.value" type="number" @change="normalizeSceneNode(step)" />
              </td>
              <td>
                <input v-model.number="step.delay" type="number" min="0" @change="normalizeSceneNode(step)" />
              </td>
              <td class="table-actions">
                <button class="btn btn-danger" @click="removeStep(index)">{{ t('common.delete') }}</button>
              </td>
            </tr>
          </tbody>
        </table>
        <div v-else class="empty-state subtle">{{ t('scenes.noSteps') }}</div>

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

.scene-list {
  display: grid;
  gap: 12px;
}

.scene-card {
  background: #ffffff;
  border: 1px solid var(--border);
  border-radius: 14px;
  padding: 12px;
  display: grid;
  gap: 10px;
}

.scene-card header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.scene-card h3 {
  margin: 0;
  font-size: 16px;
}

.scene-card header span {
  color: var(--text-secondary);
  font-size: 12px;
}

.chips {
  display: flex;
  gap: 6px;
  flex-wrap: wrap;
}

.chip {
  border-radius: 999px;
  padding: 3px 9px;
  font-size: 12px;
  color: #334155;
  background: #f1f5f9;
}

.chip.more {
  color: #1e40af;
  background: #dbeafe;
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

.empty-state.subtle {
  padding: 12px;
  font-size: 13px;
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

.field {
  display: grid;
  gap: 6px;
}

.field span,
.step-header h4 {
  font-size: 13px;
  color: var(--text-secondary);
  margin: 0;
}

.step-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

table {
  width: 100%;
  border-collapse: collapse;
  border: 1px solid var(--border);
  border-radius: 12px;
  overflow: hidden;
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
  font-size: 12px;
  text-transform: uppercase;
  letter-spacing: 0.04em;
  background: #f8fbff;
}

tr:last-child td {
  border-bottom: none;
}

input,
select {
  height: 36px;
  border: 1px solid var(--border);
  border-radius: 8px;
  padding: 0 10px;
  font-size: 14px;
  width: 100%;
}

input:focus,
select:focus {
  outline: none;
  border-color: #60a5fa;
  box-shadow: 0 0 0 3px rgba(96, 165, 250, 0.2);
}

.table-actions {
  display: flex;
  justify-content: flex-start;
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
</style>
