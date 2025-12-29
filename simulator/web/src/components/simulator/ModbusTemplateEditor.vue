<template>
    <div class="modbus-template-editor">
        <el-card class="editor-card">
            <template #header>
                <div class="card-header">
                    <div class="header-left">
                        <span class="card-title">Modbus 寄存器配置</span>
                        <el-tag size="small" type="info">{{ localConfig.slaves.length }} 个 Slave</el-tag>
                    </div>
                    <div class="header-actions">
                        <el-button type="primary" size="small" @click="showAddSlaveDialog = true">
                            <el-icon>
                                <Plus />
                            </el-icon>
                            添加 Slave
                        </el-button>
                    </div>
                </div>
            </template>

            <!-- Slave Tabs -->
            <div class="slave-content" v-if="localConfig.slaves.length > 0">
                <el-tabs v-model="activeSlaveId" type="border-card" @tab-remove="handleDeleteSlave" class="slave-tabs">
                    <el-tab-pane v-for="slave in localConfig.slaves" :key="slave.slaveId"
                        :label="`Slave ${slave.slaveId}`" :name="slave.slaveId.toString()" :closable="true">
                        <!-- 当前 Slave 的寄存器管理 -->
                        <div class="slave-registers">
                            <div class="register-header">
                                <span class="register-count">共 {{ getSlaveRegisters(slave).length }} 个寄存器</span>
                                <el-button type="primary" size="small" @click="handleAddRegister">
                                    <el-icon>
                                        <Plus />
                                    </el-icon>
                                    添加寄存器
                                </el-button>
                            </div>

                            <!-- 寄存器类型 Tabs -->
                            <el-tabs v-model="activeRegisterType" type="card" class="register-type-tabs">
                                <el-tab-pane name="coil">
                                    <template #label>
                                        <span class="tab-label">线圈 <el-badge :value="getRegisterCount(slave, 'coil')"
                                                :hidden="getRegisterCount(slave, 'coil') === 0" /></span>
                                    </template>
                                    <RegisterTable :registers="getRegistersByType(slave, 'coil')" register-type="coil"
                                        @edit="handleEditRegister" @delete="handleDeleteRegister"
                                        @value-change="handleValueChange" />
                                </el-tab-pane>
                                <el-tab-pane name="discrete_input">
                                    <template #label>
                                        <span class="tab-label">离散输入 <el-badge
                                                :value="getRegisterCount(slave, 'discrete_input')"
                                                :hidden="getRegisterCount(slave, 'discrete_input') === 0" /></span>
                                    </template>
                                    <RegisterTable :registers="getRegistersByType(slave, 'discrete_input')"
                                        register-type="discrete_input" @edit="handleEditRegister"
                                        @delete="handleDeleteRegister" @value-change="handleValueChange" />
                                </el-tab-pane>
                                <el-tab-pane name="holding_register">
                                    <template #label>
                                        <span class="tab-label">保持寄存器 <el-badge
                                                :value="getRegisterCount(slave, 'holding_register')"
                                                :hidden="getRegisterCount(slave, 'holding_register') === 0" /></span>
                                    </template>
                                    <RegisterTable :registers="getRegistersByType(slave, 'holding_register')"
                                        register-type="holding_register" @edit="handleEditRegister"
                                        @delete="handleDeleteRegister" @value-change="handleValueChange" />
                                </el-tab-pane>
                                <el-tab-pane name="input_register">
                                    <template #label>
                                        <span class="tab-label">输入寄存器 <el-badge
                                                :value="getRegisterCount(slave, 'input_register')"
                                                :hidden="getRegisterCount(slave, 'input_register') === 0" /></span>
                                    </template>
                                    <RegisterTable :registers="getRegistersByType(slave, 'input_register')"
                                        register-type="input_register" @edit="handleEditRegister"
                                        @delete="handleDeleteRegister" @value-change="handleValueChange" />
                                </el-tab-pane>
                            </el-tabs>
                        </div>
                    </el-tab-pane>
                </el-tabs>
            </div>
            <el-empty v-else description="暂无 Slave，请添加" />
        </el-card>

        <!-- 添加 Slave 对话框 -->
        <el-dialog v-model="showAddSlaveDialog" title="添加 Modbus Slave" width="400px" append-to-body>
            <el-form :model="newSlaveForm" label-width="100px">
                <el-form-item label="Slave ID">
                    <el-input-number v-model="newSlaveForm.slaveId" :min="1" :max="247" style="width: 100%" />
                </el-form-item>
            </el-form>
            <template #footer>
                <el-button @click="showAddSlaveDialog = false">取消</el-button>
                <el-button type="primary" @click="handleAddSlave">添加</el-button>
            </template>
        </el-dialog>

        <!-- 寄存器编辑对话框 -->
        <ModbusRegisterDialog v-model="showRegisterDialog" :register="editingRegister" @submit="handleRegisterSubmit" />
    </div>
</template>

<script setup lang="ts">
import { ref, watch, computed } from 'vue'
import { Plus } from '@element-plus/icons-vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import type {
    ModbusSlaveConfig,
    ModbusRegisterConfig,
    ModbusRegisterType,
} from '@/types/simulator'
import ModbusRegisterDialog from './ModbusRegisterDialog.vue'
import RegisterTable from './ModbusRegisterTable.vue'

interface ModbusConfig {
    slaves: ModbusSlaveConfig[]
    defaultSlaveId?: number
}

interface Props {
    modelValue: ModbusConfig
}

const props = defineProps<Props>()

const emit = defineEmits<{
    'update:modelValue': [value: ModbusConfig]
}>()

// 本地配置副本，避免直接修改 props
const localConfig = ref<ModbusConfig>({
    slaves: [],
    defaultSlaveId: 1
})

// 监听 props 变化
watch(() => props.modelValue, (newVal) => {
    if (newVal) {
        // 防止无限循环：如果新值和当前本地值相同，则不更新
        if (JSON.stringify(newVal) === JSON.stringify(localConfig.value)) {
            return
        }

        // 简单的深拷贝
        localConfig.value = JSON.parse(JSON.stringify(newVal))
        if (!localConfig.value.slaves) {
            localConfig.value.slaves = []
        }
        // 确保每个 slave 都有 registers 数组
        localConfig.value.slaves.forEach(slave => {
            if (!slave.registers) {
                slave.registers = []
            }
        })
    }
}, { immediate: true, deep: true })

// 监听本地配置变化，同步回父组件
watch(localConfig, (newVal) => {
    emit('update:modelValue', newVal)
}, { deep: true })

const activeSlaveId = ref('')
const currentSlave = computed(() =>
    localConfig.value.slaves.find(s => s.slaveId.toString() === activeSlaveId.value)
)

// 寄存器类型 Tab
const activeRegisterType = ref<ModbusRegisterType>('holding_register')

// 获取 Slave 的所有寄存器
function getSlaveRegisters(slave: ModbusSlaveConfig): ModbusRegisterConfig[] {
    return slave.registers || []
}

// 按类型获取寄存器
function getRegistersByType(slave: ModbusSlaveConfig, type: ModbusRegisterType): ModbusRegisterConfig[] {
    return slave.registers?.filter(r => r.type === type) || []
}

// 获取指定类型的寄存器数量
function getRegisterCount(slave: ModbusSlaveConfig, type: ModbusRegisterType): number {
    return getRegistersByType(slave, type).length
}

// 添加 Slave 对话框
const showAddSlaveDialog = ref(false)
const newSlaveForm = ref({
    slaveId: 1,
})

// 寄存器对话框
const showRegisterDialog = ref(false)
const editingRegister = ref<ModbusRegisterConfig | null>(null)

// 初始化时选中第一个 Slave
watch(() => localConfig.value.slaves, (newSlaves) => {
    if (newSlaves.length > 0 && !activeSlaveId.value) {
        activeSlaveId.value = newSlaves[0].slaveId.toString()
    }
    // 更新默认 Slave ID生成逻辑
    const maxId = newSlaves.length > 0 ? Math.max(...newSlaves.map(s => s.slaveId)) : 0
    if (newSlaveForm.value.slaveId <= maxId) {
        newSlaveForm.value.slaveId = Math.min(maxId + 1, 247)
    }
}, { deep: true })

function handleAddSlave() {
    if (localConfig.value.slaves.some(s => s.slaveId === newSlaveForm.value.slaveId)) {
        ElMessage.warning(`Slave ID ${newSlaveForm.value.slaveId} 已存在`)
        return
    }

    localConfig.value.slaves.push({
        slaveId: newSlaveForm.value.slaveId,
        registers: [],
    })

    ElMessage.success('Slave 添加成功')
    showAddSlaveDialog.value = false
    activeSlaveId.value = newSlaveForm.value.slaveId.toString()
}

async function handleDeleteSlave(slaveIdStr: string) {
    const slaveId = parseInt(slaveIdStr)
    try {
        await ElMessageBox.confirm(
            `确定要删除 Slave ${slaveId} 吗？所有寄存器配置将丢失。`,
            '删除确认',
            { type: 'warning' }
        )

        const index = localConfig.value.slaves.findIndex(s => s.slaveId === slaveId)
        if (index !== -1) {
            localConfig.value.slaves.splice(index, 1)
            ElMessage.success('Slave 已删除')

            // 切换到其他 Slave
            if (activeSlaveId.value === slaveIdStr && localConfig.value.slaves.length > 0) {
                activeSlaveId.value = localConfig.value.slaves[0].slaveId.toString()
            } else if (localConfig.value.slaves.length === 0) {
                activeSlaveId.value = ''
            }
        }
    } catch {
        // 取消
    }
}

function handleAddRegister() {
    editingRegister.value = null
    showRegisterDialog.value = true
}

function handleEditRegister(register: ModbusRegisterConfig) {
    editingRegister.value = { ...register }
    showRegisterDialog.value = true
}

async function handleDeleteRegister(register: ModbusRegisterConfig) {
    if (!currentSlave.value) return

    try {
        await ElMessageBox.confirm(
            `确定要删除地址 ${register.address} 的寄存器吗？`,
            '删除确认',
            { type: 'warning' }
        )

        const slave = currentSlave.value
        if (slave.registers) {
            const idx = slave.registers.findIndex(r => r.type === register.type && r.address === register.address)
            if (idx !== -1) {
                slave.registers.splice(idx, 1)
                ElMessage.success('寄存器已删除')
            }
        }
    } catch {
        // 取消
    }
}

function handleRegisterSubmit(register: ModbusRegisterConfig) {
    if (!currentSlave.value) return

    const slave = currentSlave.value
    if (!slave.registers) slave.registers = []

    const idx = slave.registers.findIndex(r => r.type === register.type && r.address === register.address)

    if (idx !== -1) {
        // 更新
        slave.registers[idx] = register
        ElMessage.success('寄存器已更新')
    } else {
        // 添加
        slave.registers.push(register)
        ElMessage.success('寄存器已添加')
    }

    showRegisterDialog.value = false
}

function handleValueChange(register: ModbusRegisterConfig, newValue: number | boolean) {
    if (!currentSlave.value || !currentSlave.value.registers) return

    const idx = currentSlave.value.registers.findIndex(r => r.type === register.type && r.address === register.address)
    if (idx !== -1) {
        currentSlave.value.registers[idx].value = newValue
    }
}
</script>

<style lang="scss" scoped>
.modbus-template-editor {
    .editor-card {
        :deep(.el-card__body) {
            padding: 0;
        }
    }

    .card-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        flex-wrap: wrap;
        gap: 12px;

        .header-left {
            display: flex;
            align-items: center;
            gap: 10px;

            .card-title {
                font-weight: 600;
                font-size: 16px;
            }
        }
    }

    .slave-content {
        .slave-tabs {
            border: none;
            box-shadow: none;

            :deep(.el-tabs__header) {
                background: var(--bg-secondary);
                border-radius: 0;
                margin: 0;
            }

            :deep(.el-tabs__content) {
                padding: 0;
            }
        }
    }

    .slave-registers {
        padding: 16px;

        .register-header {
            display: flex;
            justify-content: space-between;
            align-items: center;
            margin-bottom: 16px;

            .register-count {
                color: var(--text-muted);
                font-size: 14px;
            }
        }

        .register-type-tabs {
            :deep(.el-tabs__header) {
                margin-bottom: 12px;
            }

            .tab-label {
                display: flex;
                align-items: center;
                gap: 6px;

                :deep(.el-badge__content) {
                    transform: scale(0.8);
                }
            }
        }
    }

    :deep(.el-table) {
        min-height: 200px;
    }

    :deep(.el-tabs__content) {
        min-height: 250px;
    }
}
</style>
