<script setup lang="ts">
import { computed, ref } from 'vue'
import { ElMessageBox } from 'element-plus'
import { useI18n } from '../../composables/useI18n'
import type { NodeItem, Scene, SceneNode, ToastType } from '../../types/config'
import { deepClone } from '../../utils/deepClone'
import { VueDraggableNext } from 'vue-draggable-next'

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
const editorVisible = ref(false)
const editorMode = ref<'table' | 'visual'>('visual')

const keyword = ref('')

const filteredScenes = computed(() => {
  const key = keyword.value.trim().toLowerCase()
  return props.scenes.filter((scene) => {
    if (!key) {
      return true
    }
    return scene.name.toLowerCase().includes(key)
  })
})

const totalSteps = computed(() => props.scenes.reduce((sum, scene) => sum + scene.nodes.length, 0))

const avgSteps = computed(() => {
  if (!props.scenes.length) {
    return 0
  }
  return Number((totalSteps.value / props.scenes.length).toFixed(1))
})

const resetFilters = () => {
  keyword.value = ''
}

const openCreate = () => {
  editingScene.value = {
    name: '',
    nodes: []
  }
  editingIndex.value = -1
  editorVisible.value = true
}

const openEdit = (index: number) => {
  const target = filteredScenes.value[index]
  if (!target) {
    return
  }

  const sourceIndex = props.scenes.indexOf(target)
  if (sourceIndex < 0) {
    return
  }

  editingScene.value = deepClone(target)
  editingIndex.value = sourceIndex
  editorVisible.value = true
}

const resetEditor = () => {
  editingScene.value = null
  editingIndex.value = -1
}

const closeEditor = () => {
  editorVisible.value = false
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
  editorVisible.value = false
}

const remove = async (index: number) => {
  const target = filteredScenes.value[index]
  if (!target) {
    return
  }

  try {
    await ElMessageBox.confirm(
      t('scenes.confirmDelete', { name: target.name }),
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

  const sourceIndex = props.scenes.indexOf(target)
  if (sourceIndex < 0) {
    return
  }

  const next = deepClone(props.scenes)
  next.splice(sourceIndex, 1)
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

const previewSteps = (scene: Scene) => scene.nodes.slice(0, 4)

const insertStepAfter = (index: number) => {
  if (!editingScene.value) return
  const firstNode = props.nodes[0]
  if (!firstNode) {
    emit('notify', { message: t('toast.validationError', { message: t('scenes.validation.noNode') }), type: 'error' })
    return
  }
  editingScene.value.nodes.splice(index + 1, 0, {
    id: firstNode.global_id,
    value: 0,
    delay: 0
  })
}

const updateStepField = (index: number, field: 'id' | 'value' | 'delay', val: number) => {
  const node = editingScene.value?.nodes[index]
  if (!node) return
  node[field] = val
  normalizeSceneNode(node)
}

const viewingScene = ref<Scene | null>(null)
const viewerVisible = ref(false)

const openViewer = (index: number) => {
  const target = filteredScenes.value[index]
  if (!target) return
  viewingScene.value = target
  viewerVisible.value = true
}

const resetViewer = () => {
  viewingScene.value = null
}

/** 计算场景总执行时间（毫秒） */
const totalDelay = (scene: Scene) => {
  return scene.nodes.reduce((sum, node) => sum + (node.delay ?? 0), 0)
}
</script>

<template>
  <section class="page-shell">
    <el-card shadow="never" class="toolbar-card">
      <div class="toolbar-row">
        <el-input v-model="keyword" :placeholder="t('scenes.keywordPlaceholder')" clearable class="toolbar-item">
          <template #prefix>
            <el-icon>
              <Search />
            </el-icon>
          </template>
        </el-input>

        <el-button @click="resetFilters">{{ t('common.reset') }}</el-button>

        <el-button type="primary" class="create-btn" @click="openCreate">
          <el-icon>
            <Plus />
          </el-icon>
          <span>{{ t('scenes.add') }}</span>
        </el-button>
      </div>

      <div class="summary-row">
        <el-tag effect="plain">{{ t('common.total') }}: {{ scenes.length }}</el-tag>
        <el-tag effect="plain" type="success">{{ t('scenes.totalSteps') }}: {{ totalSteps }}</el-tag>
        <el-tag effect="plain" type="info">{{ t('scenes.avgSteps') }}: {{ avgSteps }}</el-tag>
        <el-tag effect="plain" type="warning">{{ t('common.matched') }}: {{ filteredScenes.length }}</el-tag>
      </div>
    </el-card>

    <el-card v-if="filteredScenes.length" shadow="never">
      <el-table :data="filteredScenes" stripe border>
        <el-table-column type="index" width="60" />

        <el-table-column prop="name" :label="t('scenes.sceneName')" min-width="220" />

        <el-table-column :label="t('scenes.steps')" min-width="110" align="center">
          <template #default="{ row }">
            <el-tag>{{ row.nodes.length }}</el-tag>
          </template>
        </el-table-column>

        <el-table-column :label="t('scenes.stepPreview')" min-width="320">
          <template #default="{ row }">
            <div class="preview-tags">
              <el-tag v-for="(step, idx) in previewSteps(row)" :key="idx" type="info" effect="plain">
                {{ nodeLabel(step.id) }} -> {{ step.value }}
              </el-tag>
              <el-tag v-if="row.nodes.length > 4" type="primary" effect="plain">
                {{ t('scenes.more', { count: row.nodes.length - 4 }) }}
              </el-tag>
            </div>
          </template>
        </el-table-column>

        <el-table-column :label="t('common.actions')" width="240" fixed="right">
          <template #default="{ $index }">
            <el-button type="success" link @click="openViewer($index)">{{ t('scenes.viewFlow') }}</el-button>
            <el-button type="primary" link @click="openEdit($index)">{{ t('common.edit') }}</el-button>
            <el-button type="danger" link @click="remove($index)">{{ t('common.delete') }}</el-button>
          </template>
        </el-table-column>
      </el-table>
    </el-card>

    <el-empty v-else :description="t('scenes.empty')" />

    <!-- ============ 编辑器对话框 ============ -->
    <el-dialog v-model="editorVisible" :title="editingIndex >= 0 ? t('scenes.editTitle') : t('scenes.createTitle')"
      width="720px" destroy-on-close @closed="resetEditor">
      <template v-if="editingScene">
        <el-form label-position="top">
          <el-form-item :label="t('scenes.sceneName')">
            <el-input v-model="editingScene.name" />
          </el-form-item>
        </el-form>

        <el-tabs v-model="editorMode">
          <el-tab-pane :label="t('scenes.editorVisual')" name="visual">
            <div class="visual-toolbar">
              <el-button @click="addStep">
                <el-icon>
                  <Plus />
                </el-icon>
                <span>{{ t('scenes.addStep') }}</span>
              </el-button>
            </div>

            <div v-if="editingScene.nodes.length" class="tl-container">
              <VueDraggableNext :list="editingScene.nodes" handle=".drag-handle" ghost-class="tl-ghost" :animation="200"
                class="tl-list">
                <div v-for="(step, index) in editingScene.nodes" :key="index" class="tl-item">
                  <!-- 延迟连接线（步骤之间） -->
                  <div v-if="index > 0" class="tl-connector">
                    <div class="tl-connector-line"></div>
                    <div v-if="step.delay" class="tl-connector-delay">
                      ⏱ {{ step.delay }}ms
                    </div>
                  </div>

                  <!-- 步骤卡片 -->
                  <div class="tl-card">
                    <div class="tl-card-head">
                      <span class="drag-handle">⠿</span>
                      <span class="tl-step-badge">{{ index + 1 }}</span>
                      <span class="tl-step-device">{{ nodeLabel(step.id) }}</span>
                      <el-button type="danger" link size="small" class="tl-del-btn"
                        @click="removeStep(index)">✖</el-button>
                    </div>
                    <div class="tl-card-body">
                      <div class="tl-field">
                        <label>{{ t('scenes.device') }}</label>
                        <el-select :model-value="step.id" size="small" class="full-width"
                          @update:model-value="updateStepField(index, 'id', $event as number)">
                          <el-option v-for="node in nodes" :key="node.global_id"
                            :label="`${node.alias} (#${node.global_id})`" :value="node.global_id" />
                        </el-select>
                      </div>
                      <div class="tl-field-row">
                        <div class="tl-field">
                          <label>{{ t('scenes.value') }}</label>
                          <el-input-number :model-value="step.value" size="small" controls-position="right"
                            class="full-width" @update:model-value="updateStepField(index, 'value', $event ?? 0)" />
                        </div>
                        <div class="tl-field">
                          <label>{{ t('scenes.delay') }} (ms)</label>
                          <el-input-number :model-value="step.delay ?? 0" size="small" :min="0"
                            controls-position="right" class="full-width"
                            @update:model-value="updateStepField(index, 'delay', $event ?? 0)" />
                        </div>
                      </div>
                    </div>
                    <div class="tl-card-actions">
                      <el-button size="small" text @click="insertStepAfter(index)">
                        <el-icon>
                          <Plus />
                        </el-icon>
                        <span>{{ t('scenes.addStep') }}</span>
                      </el-button>
                    </div>
                  </div>
                </div>
              </VueDraggableNext>

              <!-- END 标记 -->
              <div class="tl-item">
                <div class="tl-connector">
                  <div class="tl-connector-line"></div>
                </div>
                <div class="tl-end-node">
                  <span>{{ t('scenes.flowEnd') }}</span>
                </div>
              </div>
            </div>
            <el-empty v-else :description="t('scenes.noSteps')" />
          </el-tab-pane>

          <el-tab-pane :label="t('scenes.editorTable')" name="table">
            <div class="step-header">
              <h4>{{ t('scenes.steps') }}</h4>
              <el-button @click="addStep">
                <el-icon>
                  <Plus />
                </el-icon>
                <span>{{ t('scenes.addStep') }}</span>
              </el-button>
            </div>

            <el-table v-if="editingScene.nodes.length" :data="editingScene.nodes" stripe border>
              <el-table-column :label="t('scenes.device')" min-width="260">
                <template #default="{ row }">
                  <el-select v-model="row.id" class="full-width" @change="normalizeSceneNode(row)">
                    <el-option v-for="node in nodes" :key="node.global_id" :label="`${node.alias} (#${node.global_id})`"
                      :value="node.global_id" />
                  </el-select>
                </template>
              </el-table-column>
              <el-table-column :label="t('scenes.value')" min-width="180">
                <template #default="{ row }">
                  <el-input-number v-model="row.value" class="full-width" @change="normalizeSceneNode(row)" />
                </template>
              </el-table-column>
              <el-table-column :label="t('scenes.delay')" min-width="180">
                <template #default="{ row }">
                  <el-input-number v-model="row.delay" :min="0" class="full-width" @change="normalizeSceneNode(row)" />
                </template>
              </el-table-column>
              <el-table-column :label="t('common.actions')" width="110">
                <template #default="{ $index }">
                  <el-button type="danger" link @click="removeStep($index)">{{ t('common.delete') }}</el-button>
                </template>
              </el-table-column>
            </el-table>
            <el-empty v-else :description="t('scenes.noSteps')" />
          </el-tab-pane>
        </el-tabs>
      </template>

      <template #footer>
        <el-button @click="closeEditor">{{ t('common.cancel') }}</el-button>
        <el-button type="primary" @click="save">
          {{ editingIndex >= 0 ? t('common.update') : t('common.create') }}
        </el-button>
      </template>
    </el-dialog>

    <!-- ============ 查看器对话框 ============ -->
    <el-dialog v-model="viewerVisible" :title="viewingScene ? t('scenes.flowTitle', { name: viewingScene.name }) : ''"
      width="640px" top="8vh" destroy-on-close @closed="resetViewer">
      <template v-if="viewingScene">
        <div v-if="viewingScene.nodes.length" class="tl-viewer">
          <div class="tl-viewer-summary">
            <el-tag>{{ viewingScene.nodes.length }} {{ t('scenes.steps') }}</el-tag>
            <el-tag v-if="totalDelay(viewingScene)" type="warning">
              ⏱ {{ totalDelay(viewingScene) }}ms
            </el-tag>
          </div>

          <div class="tl-list">
            <div v-for="(step, index) in viewingScene.nodes" :key="index" class="tl-item">
              <!-- 延迟连接线 -->
              <div v-if="index > 0" class="tl-connector">
                <div class="tl-connector-line"></div>
                <div v-if="step.delay" class="tl-connector-delay">
                  ⏱ {{ step.delay }}ms
                </div>
              </div>

              <!-- 只读步骤卡片 -->
              <div class="tl-view-card">
                <span class="tl-step-badge">{{ index + 1 }}</span>
                <span class="tl-view-device">{{ nodeLabel(step.id) }}</span>
                <span class="tl-view-arrow">→</span>
                <span class="tl-view-value">{{ step.value }}</span>
              </div>
            </div>

            <!-- END 标记 -->
            <div class="tl-item">
              <div class="tl-connector">
                <div class="tl-connector-line"></div>
              </div>
              <div class="tl-end-node">
                <span>{{ t('scenes.flowEnd') }}</span>
              </div>
            </div>
          </div>
        </div>
        <el-empty v-else :description="t('scenes.noSteps')" />
      </template>
    </el-dialog>
  </section>
</template>

<style scoped>
/* ===== Page Layout ===== */
.page-shell {
  display: grid;
  gap: 12px;
}

.toolbar-card {
  border-style: dashed;
}

.toolbar-row {
  display: grid;
  grid-template-columns: 1.6fr auto auto;
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

.preview-tags {
  display: flex;
  gap: 6px;
  flex-wrap: wrap;
}

.step-header {
  margin: 10px 0;
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.step-header h4 {
  margin: 0;
  font-size: 15px;
}

.full-width {
  width: 100%;
}

.visual-toolbar {
  margin-bottom: 12px;
}

/* ===== Timeline Container ===== */
.tl-container {
  background: var(--el-fill-color-lighter);
  border: 1px dashed var(--el-border-color);
  border-radius: 12px;
  padding: 20px 16px;
  max-height: 60vh;
  overflow-y: auto;
}

.tl-list {
  display: flex;
  flex-direction: column;
}

.tl-item {
  display: flex;
  flex-direction: column;
}

/* ===== Timeline Connector ===== */
.tl-connector {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 4px 0;
  position: relative;
}

.tl-connector-line {
  width: 2px;
  height: 20px;
  background: var(--el-color-primary-light-3);
}

.tl-connector-delay {
  font-size: 11px;
  color: var(--el-color-warning);
  font-weight: 600;
  background: var(--el-color-warning-light-9);
  border: 1px solid var(--el-color-warning-light-5);
  padding: 1px 8px;
  border-radius: 10px;
  margin: 2px 0;
  white-space: nowrap;
}

/* ===== Editable Card ===== */
.tl-card {
  background: var(--el-bg-color);
  border: 1px solid var(--el-border-color-lighter);
  border-radius: 10px;
  padding: 0;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.04);
  transition: box-shadow 0.2s, border-color 0.2s;
  overflow: hidden;
}

.tl-card:hover {
  border-color: var(--el-color-primary-light-5);
  box-shadow: 0 4px 14px rgba(0, 0, 0, 0.08);
}

.tl-card-head {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  background: var(--el-fill-color-lighter);
  border-bottom: 1px solid var(--el-border-color-extra-light);
}

.tl-step-badge {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 22px;
  height: 22px;
  border-radius: 50%;
  background: var(--el-color-primary);
  color: #fff;
  font-size: 11px;
  font-weight: 700;
  flex-shrink: 0;
}

.tl-step-device {
  flex: 1;
  font-weight: 600;
  font-size: 13px;
  color: var(--el-text-color-primary);
}

.tl-del-btn {
  flex-shrink: 0;
}

.tl-card-body {
  padding: 10px 12px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.tl-field label {
  display: block;
  font-size: 11px;
  color: var(--el-text-color-secondary);
  margin-bottom: 2px;
}

.tl-field-row {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 8px;
}

.tl-card-actions {
  padding: 4px 12px 8px;
  display: flex;
  justify-content: center;
}

/* ===== Drag Handle ===== */
.drag-handle {
  cursor: grab;
  font-size: 16px;
  color: var(--el-text-color-placeholder);
  padding: 2px 4px;
  border-radius: 4px;
  transition: color 0.2s, background-color 0.2s;
  user-select: none;
  line-height: 1;
}

.drag-handle:hover {
  color: var(--el-color-primary);
  background: var(--el-color-primary-light-9);
}

.drag-handle:active {
  cursor: grabbing;
}

/* ===== Drag Ghost ===== */
.tl-ghost {
  opacity: 0.4;
}

.tl-ghost .tl-card {
  border-color: var(--el-color-primary);
  box-shadow: 0 4px 14px rgba(0, 0, 0, 0.15);
}

/* ===== END Node ===== */
.tl-end-node {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 8px 16px;
  background: var(--el-fill-color-light);
  border: 1px dashed var(--el-border-color);
  border-radius: 8px;
  color: var(--el-text-color-secondary);
  font-size: 12px;
  font-weight: 600;
}

/* ===== Viewer ===== */
.tl-viewer {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.tl-viewer-summary {
  display: flex;
  gap: 8px;
}

.tl-view-card {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 16px;
  background: linear-gradient(135deg, var(--el-color-primary-light-9), var(--el-color-primary-light-8));
  border: 1px solid var(--el-color-primary-light-5);
  border-radius: 8px;
  transition: transform 0.2s, box-shadow 0.2s;
}

.tl-view-card:hover {
  transform: translateX(4px);
  box-shadow: 0 2px 12px rgba(0, 0, 0, 0.08);
}

.tl-view-device {
  font-weight: 600;
  font-size: 14px;
  color: var(--el-text-color-primary);
  flex: 1;
}

.tl-view-arrow {
  color: var(--el-text-color-secondary);
  font-size: 14px;
}

.tl-view-value {
  font-weight: 700;
  font-size: 14px;
  color: var(--el-color-primary);
  min-width: 40px;
  text-align: right;
}

/* ===== Responsive ===== */
@media (max-width: 1200px) {
  .toolbar-row {
    grid-template-columns: 1fr 1fr;
  }

  .create-btn {
    justify-self: stretch;
  }
}

@media (max-width: 960px) {
  .step-header {
    flex-direction: column;
    align-items: flex-start;
    gap: 8px;
  }
}
</style>
