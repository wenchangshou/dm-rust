<template>
    <div class="template-list">
        <el-card class="table-card">
            <template #header>
                <div class="table-header">
                    <span class="header-title">模板列表</span>
                    <span class="header-count">共 {{ templates.length }} 个</span>
                    <div class="header-actions">
                        <el-button type="success" size="small" @click="openCreateDialog">
                            <el-icon>
                                <Plus />
                            </el-icon> 从模拟器创建
                        </el-button>
                        <el-button type="primary" size="small" @click="openCreateNewDialog">
                            <el-icon>
                                <DocumentAdd />
                            </el-icon> 新建模板
                        </el-button>
                        <el-button type="primary" size="small" @click="fetchTemplates">
                            <el-icon>
                                <Refresh />
                            </el-icon> 刷新
                        </el-button>
                    </div>
                </div>
            </template>

            <el-table :data="templates" v-loading="loading" style="width: 100%">
                <el-table-column prop="name" label="名称" width="200">
                    <template #default="{ row }">
                        <span class="template-name">{{ row.name }}</span>
                    </template>
                </el-table-column>
                <el-table-column prop="protocol" label="协议" width="120">
                    <template #default="{ row }">
                        <el-tag size="small" class="protocol-tag">{{ row.protocol }}</el-tag>
                    </template>
                </el-table-column>
                <el-table-column prop="description" label="描述" min-width="250" show-overflow-tooltip />
                <el-table-column prop="created_at" label="创建时间" width="180">
                    <template #default="{ row }">
                        <span class="time-text">{{ formatDate(row.created_at) }}</span>
                    </template>
                </el-table-column>
                <el-table-column label="操作" width="180" fixed="right" align="center">
                    <template #default="{ row }">
                        <el-button type="primary" size="small" class="action-btn" @click="handleEdit(row)">
                            <el-icon>
                                <Edit />
                            </el-icon> 编辑
                        </el-button>
                        <el-button type="danger" size="small" class="action-btn" @click="handleDelete(row)">
                            <el-icon>
                                <Delete />
                            </el-icon> 删除
                        </el-button>
                    </template>
                </el-table-column>
            </el-table>
        </el-card>

        <!-- 从模拟器创建模板对话框 -->
        <el-dialog v-model="showCreateDialog" title="从现有模拟器创建" width="500px">
            <el-alert title="将现有模拟器的配置保存为模板" type="info" show-icon :closable="false" style="margin-bottom: 20px" />
            <el-form :model="createForm" label-width="100px">
                <el-form-item label="来源模拟器" required>
                    <el-select v-model="createForm.sourceId" placeholder="请选择模拟器" style="width: 100%"
                        @change="handleSourceChange">
                        <el-option v-for="sim in simulators" :key="sim.id" :label="`${sim.name} (${sim.protocol})`"
                            :value="sim.id" />
                    </el-select>
                </el-form-item>
                <el-form-item label="模板名称" required>
                    <el-input v-model="createForm.name" placeholder="请输入模板名称" />
                </el-form-item>
                <el-form-item label="描述">
                    <el-input v-model="createForm.description" type="textarea" :rows="3" placeholder="请输入模板描述" />
                </el-form-item>
            </el-form>
            <template #footer>
                <el-button @click="showCreateDialog = false">取消</el-button>
                <el-button type="primary" :loading="creating" @click="submitCreate">创建</el-button>
            </template>
        </el-dialog>

        <!-- 新建模板对话框 -->
        <el-dialog v-model="showCreateNewDialog" title="新建模板" width="500px">
            <el-form :model="createNewForm" label-width="100px">
                <el-form-item label="名称" required>
                    <el-input v-model="createNewForm.name" placeholder="请输入模板名称" />
                </el-form-item>
                <el-form-item label="协议" required>
                    <el-select v-model="createNewForm.protocol" placeholder="请选择协议" style="width: 100%">
                        <el-option label="Modbus TCP" value="modbus" />
                        <el-option label="Custom (自定义)" value="custom" />
                    </el-select>
                </el-form-item>
                <el-form-item label="描述">
                    <el-input v-model="createNewForm.description" type="textarea" :rows="3" placeholder="请输入描述" />
                </el-form-item>
            </el-form>
            <template #footer>
                <el-button @click="showCreateNewDialog = false">取消</el-button>
                <el-button type="primary" :loading="creatingNew" @click="submitCreateNew">创建</el-button>
            </template>
        </el-dialog>

        <!-- 编辑模板对话框 -->
        <el-dialog v-model="showEditDialog" title="编辑模板" width="800px" top="5vh">
            <el-tabs v-model="editActiveTab">
                <el-tab-pane label="基本信息" name="info">
                    <el-form :model="editForm" label-width="80px" style="margin-top: 20px">
                        <el-form-item label="名称" required>
                            <el-input v-model="editForm.name" />
                        </el-form-item>
                        <el-form-item label="描述">
                            <el-input v-model="editForm.description" type="textarea" :rows="3" />
                        </el-form-item>
                    </el-form>
                </el-tab-pane>
                <el-tab-pane label="配置的内容" name="config" v-if="editForm.protocol === 'custom'">
                    <RuleEditor v-if="editConfig.custom" v-model="editConfig.custom" :height="'500px'" />
                    <el-empty v-else description="无配置数据" />
                </el-tab-pane>
                <el-tab-pane label="寄存器配置" name="config"
                    v-else-if="editForm.protocol === 'modbus_tcp' || editForm.protocol === 'modbus'">
                    <ModbusTemplateEditor v-if="editConfig.modbus" v-model="editConfig.modbus" />
                    <el-empty v-else description="无配置数据" />
                </el-tab-pane>
            </el-tabs>

            <template #footer>
                <el-button @click="showEditDialog = false">取消</el-button>
                <el-button type="primary" :loading="submitting" @click="submitEdit">保存</el-button>
            </template>
        </el-dialog>
    </div>
</template>

<script setup lang="ts">
import { ref, onMounted, reactive } from 'vue'
import { Delete, Refresh, Edit, Plus, DocumentAdd } from '@element-plus/icons-vue'
import { ElMessageBox, ElMessage } from 'element-plus'
import * as simulatorApi from '@/api/simulator'
import type { SimulatorTemplate, SimulatorInfo } from '@/types/simulator'
import RuleEditor from './RuleEditor.vue'
import ModbusTemplateEditor from './ModbusTemplateEditor.vue'

const templates = ref<SimulatorTemplate[]>([])
const loading = ref(false)

// 从模拟器创建相关
const showCreateDialog = ref(false)
const creating = ref(false)
const simulators = ref<SimulatorInfo[]>([])
const createForm = reactive({
    sourceId: '',
    name: '',
    description: ''
})

// 新建模板相关
const showCreateNewDialog = ref(false)
const creatingNew = ref(false)
const createNewForm = reactive({
    name: '',
    protocol: 'modbus',
    description: ''
})

// 编辑相关
const showEditDialog = ref(false)
const submitting = ref(false)
const editActiveTab = ref('info')
const editForm = reactive({
    id: '',
    name: '',
    description: '',
    protocol: ''
})
const editConfig = reactive({
    custom: null as any,
    modbus: null as any
})

onMounted(() => {
    fetchTemplates()
})

async function fetchTemplates() {
    loading.value = true
    try {
        templates.value = await simulatorApi.listTemplates()
    } catch (e: any) {
        ElMessage.error(e.message || '获取模板列表失败')
    } finally {
        loading.value = false
    }
}

async function openCreateDialog() {
    try {
        simulators.value = await simulatorApi.listSimulators()
        createForm.sourceId = ''
        createForm.name = ''
        createForm.description = ''
        showCreateDialog.value = true
    } catch (e: any) {
        ElMessage.error(e.message || '获取模拟器列表失败')
    }
}

function openCreateNewDialog() {
    createNewForm.name = ''
    createNewForm.protocol = 'modbus'
    createNewForm.description = ''
    showCreateNewDialog.value = true
}

function handleSourceChange(id: string) {
    const sim = simulators.value.find(s => s.id === id)
    if (sim && !createForm.name) {
        createForm.name = `${sim.name}_模板`
    }
}

async function submitCreate() {
    if (!createForm.sourceId) {
        ElMessage.warning('请选择来源模拟器')
        return
    }
    if (!createForm.name) {
        ElMessage.warning('请输入名称')
        return
    }

    creating.value = true
    try {
        await simulatorApi.saveAsTemplate(createForm.sourceId, {
            name: createForm.name,
            description: createForm.description
        })
        ElMessage.success('模板创建成功')
        showCreateDialog.value = false
        fetchTemplates()
    } catch (e: any) {
        ElMessage.error(e.message || '创建失败')
    } finally {
        creating.value = false
    }
}

async function submitCreateNew() {
    if (!createNewForm.name) {
        ElMessage.warning('请输入名称')
        return
    }

    creatingNew.value = true
    try {
        // Initial config based on protocol
        let initialConfig: any = {}
        if (createNewForm.protocol === 'custom') {
            initialConfig = {
                checksum: { type: 'None' },
                rules: []
            }
        } else if (createNewForm.protocol === 'modbus') {
            initialConfig = {
                defaultSlaveId: 1,
                slaves: []
            }
        }

        await simulatorApi.createTemplate({
            name: createNewForm.name,
            protocol: createNewForm.protocol,
            transport: 'tcp',
            description: createNewForm.description,
            config: initialConfig
        })
        ElMessage.success('模板创建成功')
        showCreateNewDialog.value = false
        fetchTemplates()
    } catch (e: any) {
        ElMessage.error(e.message || '创建失败')
    } finally {
        creatingNew.value = false
    }
}

function handleEdit(template: SimulatorTemplate) {
    editForm.id = template.id
    editForm.name = template.name
    editForm.description = template.description
    editForm.protocol = template.protocol
    editActiveTab.value = 'info'

    // Load config
    if (template.protocol === 'custom') {
        editConfig.custom = JSON.parse(JSON.stringify(template.config || { checksum: { type: 'None' }, rules: [] }))
        editConfig.modbus = null
    } else if (template.protocol === 'modbus_tcp' || template.protocol === 'modbus') {
        editConfig.modbus = JSON.parse(JSON.stringify(template.config || { defaultSlaveId: 1, slaves: [] }))
        editConfig.custom = null
    } else {
        editConfig.custom = null
        editConfig.modbus = null
    }

    showEditDialog.value = true
}

async function submitEdit() {
    if (!editForm.name) {
        ElMessage.warning('请输入名称')
        return
    }

    submitting.value = true
    try {
        const updateData: any = {
            name: editForm.name,
            description: editForm.description
        }

        if (editForm.protocol === 'custom' && editConfig.custom) {
            updateData.config = editConfig.custom
        } else if ((editForm.protocol === 'modbus_tcp' || editForm.protocol === 'modbus') && editConfig.modbus) {
            updateData.config = editConfig.modbus
        }

        await simulatorApi.updateTemplate(editForm.id, updateData)
        ElMessage.success('模板更新成功')
        showEditDialog.value = false
        fetchTemplates()
    } catch (e: any) {
        ElMessage.error(e.message || '更新失败')
    } finally {
        submitting.value = false
    }
}

async function handleDelete(template: SimulatorTemplate) {
    try {
        await ElMessageBox.confirm(
            `确定要删除模板 "${template.name}" 吗？`,
            '确认删除',
            { type: 'warning' }
        )
        await simulatorApi.deleteTemplate(template.id)
        ElMessage.success('模板已删除')
        fetchTemplates()
    } catch {
        // cancelled
    }
}

function formatDate(dateStr: string) {
    try {
        return new Date(dateStr).toLocaleString()
    } catch {
        return dateStr
    }
}

defineExpose({
    refresh: fetchTemplates
})
</script>

<style lang="scss" scoped>
.table-card {
    border-radius: 16px;

    .table-header {
        display: flex;
        align-items: center;
        gap: 12px;

        .header-title {
            font-weight: 600;
            color: var(--text-primary);
        }

        .header-count {
            font-size: 13px;
            color: var(--text-muted);
            background: var(--bg-input);
            padding: 4px 10px;
            border-radius: 20px;
        }

        .header-actions {
            margin-left: auto;
            display: flex;
            gap: 10px;
        }
    }
}

.template-name {
    font-weight: 600;
    color: #667eea;
}

.protocol-tag {
    background: rgba(102, 126, 234, 0.15);
    border-color: rgba(102, 126, 234, 0.3);
    color: #667eea;
    font-weight: 500;
}

.time-text {
    font-family: 'JetBrains Mono', monospace;
    font-size: 13px;
    color: var(--text-secondary);
}

.action-btn {
    padding: 6px 12px;
}
</style>
