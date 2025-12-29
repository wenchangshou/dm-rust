<template>
  <div class="modbus-simulator-panel">
    <!-- 统一的 Modbus 管理面板 -->
    <el-card class="unified-card">
      <template #header>
        <div class="card-header">
          <div class="header-left">
            <span class="card-title">Modbus 寄存器管理</span>
            <el-tag size="small" type="info">{{ slaves.length }} 个 Slave</el-tag>
          </div>
          <div class="header-actions">
            <el-checkbox v-model="autoRefresh" size="small">
              实时刷新
            </el-checkbox>
            <el-select v-model="refreshInterval" size="small" class="interval-select" :disabled="!autoRefresh">
              <el-option :value="500" label="0.5秒" />
              <el-option :value="1000" label="1秒" />
              <el-option :value="2000" label="2秒" />
              <el-option :value="5000" label="5秒" />
            </el-select>
            <el-button size="small" @click="handleManualRefresh" :loading="loading">
              <el-icon>
                <Refresh />
              </el-icon>
            </el-button>
            <el-divider direction="vertical" />
            <el-button type="primary" size="small" @click="showAddSlaveDialog = true">
              <el-icon>
                <Plus />
              </el-icon>
              添加 Slave
            </el-button>
          </div>
        </div>
      </template>

      <!-- Slave Tabs - 寄存器作为 Tab 内容 -->
      <div class="slave-content" v-if="slaves.length > 0">
        <el-tabs v-model="activeSlaveId" type="border-card" @tab-remove="handleDeleteSlave" class="slave-tabs">
          <el-tab-pane v-for="slave in slaves" :key="slave.slaveId" :label="`Slave ${slave.slaveId}`"
            :name="slave.slaveId.toString()" :closable="slaves.length > 1">
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
                    @edit="handleEditRegister" @delete="handleDeleteRegister" @value-change="handleValueChange" />
                </el-tab-pane>
                <el-tab-pane name="discrete_input">
                  <template #label>
                    <span class="tab-label">离散输入 <el-badge :value="getRegisterCount(slave, 'discrete_input')"
                        :hidden="getRegisterCount(slave, 'discrete_input') === 0" /></span>
                  </template>
                  <RegisterTable :registers="getRegistersByType(slave, 'discrete_input')" register-type="discrete_input"
                    @edit="handleEditRegister" @delete="handleDeleteRegister" @value-change="handleValueChange" />
                </el-tab-pane>
                <el-tab-pane name="holding_register">
                  <template #label>
                    <span class="tab-label">保持寄存器 <el-badge :value="getRegisterCount(slave, 'holding_register')"
                        :hidden="getRegisterCount(slave, 'holding_register') === 0" /></span>
                  </template>
                  <RegisterTable :registers="getRegistersByType(slave, 'holding_register')"
                    register-type="holding_register" @edit="handleEditRegister" @delete="handleDeleteRegister"
                    @value-change="handleValueChange" />
                </el-tab-pane>
                <el-tab-pane name="input_register">
                  <template #label>
                    <span class="tab-label">输入寄存器 <el-badge :value="getRegisterCount(slave, 'input_register')"
                        :hidden="getRegisterCount(slave, 'input_register') === 0" /></span>
                  </template>
                  <RegisterTable :registers="getRegistersByType(slave, 'input_register')" register-type="input_register"
                    @edit="handleEditRegister" @delete="handleDeleteRegister" @value-change="handleValueChange" />
                </el-tab-pane>
              </el-tabs>
            </div>
          </el-tab-pane>
        </el-tabs>
      </div>
      <el-empty v-else description="暂无 Slave，请添加" />
    </el-card>

    <!-- 添加 Slave 对话框 -->
    <el-dialog v-model="showAddSlaveDialog" title="添加 Modbus Slave" width="400px">
      <el-form :model="newSlaveForm" label-width="100px">
        <el-form-item label="Slave ID">
          <el-input-number v-model="newSlaveForm.slaveId" :min="1" :max="247" style="width: 100%" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="showAddSlaveDialog = false">取消</el-button>
        <el-button type="primary" @click="handleAddSlave" :loading="loading">添加</el-button>
      </template>
    </el-dialog>

    <!-- 寄存器编辑对话框 -->
    <ModbusRegisterDialog v-model="showRegisterDialog" :register="editingRegister" :loading="loading"
      @submit="handleRegisterSubmit" />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from 'vue'
import { Plus, Refresh } from '@element-plus/icons-vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { useSimulatorStore } from '@/stores/simulator'
import type {
  ModbusSlaveConfig,
  ModbusRegisterConfig,
  ModbusRegisterType,
  ModbusSimulatorValues,
} from '@/types/simulator'
import ModbusRegisterDialog from './ModbusRegisterDialog.vue'
import RegisterTable from './ModbusRegisterTable.vue'

interface Props {
  simulatorId: string
}

const props = defineProps<Props>()

const store = useSimulatorStore()
const loading = ref(false)

// 实时刷新相关
const autoRefresh = ref(true)
const refreshInterval = ref(1000) // 刷新间隔（毫秒）
let refreshTimer: ReturnType<typeof setInterval> | null = null

// 启动自动刷新
function startAutoRefresh() {
  if (refreshTimer) return
  refreshTimer = setInterval(async () => {
    if (autoRefresh.value) {
      // 使用静默刷新，不触发 loading 状态，避免闪屏
      await store.refreshSimulatorSilently(props.simulatorId)
    }
  }, refreshInterval.value)
}

// 停止自动刷新
function stopAutoRefresh() {
  if (refreshTimer) {
    clearInterval(refreshTimer)
    refreshTimer = null
  }
}

// 手动刷新
async function handleManualRefresh() {
  loading.value = true
  try {
    await store.fetchSimulator(props.simulatorId)
  } finally {
    loading.value = false
  }
}

// 监听自动刷新开关变化
watch(autoRefresh, (enabled) => {
  if (enabled) {
    startAutoRefresh()
  } else {
    stopAutoRefresh()
  }
})

// 监听刷新间隔变化
watch(refreshInterval, () => {
  if (autoRefresh.value) {
    stopAutoRefresh()
    startAutoRefresh()
  }
})

onMounted(() => {
  if (autoRefresh.value) {
    startAutoRefresh()
  }
})

onUnmounted(() => {
  stopAutoRefresh()
})

// Slave 相关状态
const slaves = computed<ModbusSlaveConfig[]>(() => {
  const values = store.currentSimulator?.state.values as ModbusSimulatorValues | undefined
  return values?.slaves || []
})

const activeSlaveId = ref('')
const currentSlave = computed(() =>
  slaves.value.find(s => s.slaveId.toString() === activeSlaveId.value)
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
watch(slaves, (newSlaves) => {
  if (newSlaves.length > 0 && !activeSlaveId.value) {
    activeSlaveId.value = newSlaves[0].slaveId.toString()
  }
}, { immediate: true })

async function handleAddSlave() {
  if (slaves.value.some(s => s.slaveId === newSlaveForm.value.slaveId)) {
    ElMessage.warning(`Slave ID ${newSlaveForm.value.slaveId} 已存在`)
    return
  }

  loading.value = true
  try {
    await store.addModbusSlave(props.simulatorId, {
      slaveId: newSlaveForm.value.slaveId,
      registers: [],
    })
    ElMessage.success('Slave 添加成功')
    showAddSlaveDialog.value = false

    // 刷新数据后设置选中的 Slave
    await store.fetchSimulator(props.simulatorId)
    activeSlaveId.value = newSlaveForm.value.slaveId.toString()
    newSlaveForm.value.slaveId = getNextSlaveId()
  } catch {
    // 错误已处理
  } finally {
    loading.value = false
  }
}

async function handleDeleteSlave(slaveIdStr: string) {
  const slaveId = parseInt(slaveIdStr)
  try {
    await ElMessageBox.confirm(
      `确定要删除 Slave ${slaveId} 吗？所有寄存器配置将丢失。`,
      '删除确认',
      { type: 'warning' }
    )

    loading.value = true
    await store.deleteModbusSlave(props.simulatorId, slaveId)
    ElMessage.success('Slave 已删除')

    // 刷新数据
    await store.fetchSimulator(props.simulatorId)

    // 切换到其他 Slave
    if (activeSlaveId.value === slaveIdStr && slaves.value.length > 0) {
      activeSlaveId.value = slaves.value[0].slaveId.toString()
    } else if (slaves.value.length === 0) {
      activeSlaveId.value = ''
    }
  } catch {
    // 取消或错误
  } finally {
    loading.value = false
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

    loading.value = true
    await store.deleteModbusRegister(props.simulatorId, {
      slaveId: currentSlave.value.slaveId,
      registerType: register.type,
      address: register.address,
    })
    ElMessage.success('寄存器已删除')
  } catch {
    // 取消或错误
  } finally {
    loading.value = false
  }
}

async function handleRegisterSubmit(register: ModbusRegisterConfig) {
  if (!currentSlave.value) return

  loading.value = true
  try {
    await store.setModbusRegister(props.simulatorId, {
      slaveId: currentSlave.value.slaveId,
      register,
    })
    ElMessage.success(editingRegister.value ? '寄存器已更新' : '寄存器已添加')
    showRegisterDialog.value = false

    // 强制刷新模拟器详情以确保 UI 更新
    await store.fetchSimulator(props.simulatorId)
  } catch {
    // 错误已处理
  } finally {
    loading.value = false
  }
}

async function handleValueChange(register: ModbusRegisterConfig, newValue: number | boolean) {
  if (!currentSlave.value) return

  try {
    await store.updateModbusRegisterValue(props.simulatorId, {
      slaveId: currentSlave.value.slaveId,
      registerType: register.type,
      address: register.address,
      value: newValue,
    })
  } catch {
    // 错误已处理
  }
}

function getNextSlaveId(): number {
  if (slaves.value.length === 0) return 1
  const maxId = Math.max(...slaves.value.map(s => s.slaveId))
  return Math.min(maxId + 1, 247)
}
</script>

<style lang="scss" scoped>
.modbus-simulator-panel {
  .unified-card {
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

  .header-actions {
    display: flex;
    align-items: center;
    gap: 8px;

    .el-divider--vertical {
      height: 20px;
    }
  }

  .interval-select {
    width: 100px;
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

  // 避免表格内容变化导致闪烁
  :deep(.el-table) {
    min-height: 200px;
  }

  :deep(.el-tabs__content) {
    min-height: 250px;
  }
}
</style>
