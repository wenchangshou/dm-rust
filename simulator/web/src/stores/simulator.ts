import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type {
  SimulatorInfo,
  ProtocolInfo,
  ModbusSlaveConfig,
  AddModbusSlaveRequest,
  SetModbusRegisterRequest,
  DeleteModbusRegisterRequest,
  UpdateModbusRegisterRequest,
  BatchUpdateModbusRegistersRequest,
} from '@/types/simulator'
import * as simulatorApi from '@/api/simulator'

export const useSimulatorStore = defineStore('simulator', () => {
  // 状态
  const simulators = ref<SimulatorInfo[]>([])
  const protocols = ref<ProtocolInfo[]>([])
  const loading = ref(false)
  const currentSimulator = ref<SimulatorInfo | null>(null)

  // 计算属性
  const runningCount = computed(() =>
    simulators.value.filter(s => s.status === 'running').length
  )

  const stoppedCount = computed(() =>
    simulators.value.filter(s => s.status === 'stopped').length
  )

  const totalConnections = computed(() =>
    simulators.value.reduce((sum, s) => sum + s.state.stats.active_connections, 0)
  )

  // 方法
  async function fetchSimulators() {
    loading.value = true
    try {
      simulators.value = await simulatorApi.listSimulators()
    } finally {
      loading.value = false
    }
  }

  async function fetchProtocols() {
    protocols.value = await simulatorApi.getProtocols()
  }

  async function fetchSimulator(id: string) {
    loading.value = true
    try {
      currentSimulator.value = await simulatorApi.getSimulator(id)
    } finally {
      loading.value = false
    }
  }

  // 静默刷新，不触发 loading 状态，用于自动刷新场景
  async function refreshSimulatorSilently(id: string) {
    try {
      const data = await simulatorApi.getSimulator(id)
      // 只更新变化的部分，避免整体替换导致闪屏
      if (currentSimulator.value && currentSimulator.value.id === id) {
        // 深度更新 state.values
        currentSimulator.value.state.values = data.state.values
        currentSimulator.value.state.stats = data.state.stats
        currentSimulator.value.state.online = data.state.online
        currentSimulator.value.state.fault = data.state.fault
        currentSimulator.value.status = data.status
      } else {
        currentSimulator.value = data
      }
    } catch {
      // 静默刷新忽略错误
    }
  }

  async function createSimulator(data: Parameters<typeof simulatorApi.createSimulator>[0]) {
    const simulator = await simulatorApi.createSimulator(data)
    simulators.value.push(simulator)
    return simulator
  }

  async function deleteSimulator(id: string) {
    await simulatorApi.deleteSimulator(id)
    simulators.value = simulators.value.filter(s => s.id !== id)
  }

  async function startSimulator(id: string) {
    const updated = await simulatorApi.startSimulator(id)
    updateSimulatorInList(updated)
    if (currentSimulator.value?.id === id) {
      currentSimulator.value = updated
    }
  }

  async function stopSimulator(id: string) {
    const updated = await simulatorApi.stopSimulator(id)
    updateSimulatorInList(updated)
    if (currentSimulator.value?.id === id) {
      currentSimulator.value = updated
    }
  }

  async function setOnline(id: string, online: boolean) {
    const updated = await simulatorApi.setOnline(id, online)
    updateSimulatorInList(updated)
    if (currentSimulator.value?.id === id) {
      currentSimulator.value = updated
    }
  }

  async function triggerFault(id: string, faultType: string) {
    const updated = await simulatorApi.triggerFault(id, { fault_type: faultType })
    updateSimulatorInList(updated)
    if (currentSimulator.value?.id === id) {
      currentSimulator.value = updated
    }
  }

  async function clearFault(id: string) {
    const updated = await simulatorApi.clearFault(id)
    updateSimulatorInList(updated)
    if (currentSimulator.value?.id === id) {
      currentSimulator.value = updated
    }
  }

  function updateSimulatorInList(simulator: SimulatorInfo) {
    const index = simulators.value.findIndex(s => s.id === simulator.id)
    if (index !== -1) {
      simulators.value[index] = simulator
    }
  }

  // ============ Modbus 相关状态和方法 ============

  const modbusSlaves = ref<ModbusSlaveConfig[]>([])

  async function fetchModbusSlaves(id: string) {
    modbusSlaves.value = await simulatorApi.getModbusSlaves(id)
  }

  async function addModbusSlave(id: string, data: AddModbusSlaveRequest) {
    const updated = await simulatorApi.addModbusSlave(id, data)
    updateSimulatorInList(updated)
    if (currentSimulator.value?.id === id) {
      currentSimulator.value = updated
    }
    return updated
  }

  async function deleteModbusSlave(id: string, slaveId: number) {
    const updated = await simulatorApi.deleteModbusSlave(id, slaveId)
    updateSimulatorInList(updated)
    if (currentSimulator.value?.id === id) {
      currentSimulator.value = updated
    }
    return updated
  }

  async function setModbusRegister(id: string, data: SetModbusRegisterRequest) {
    const updated = await simulatorApi.setModbusRegister(id, data)
    updateSimulatorInList(updated)
    if (currentSimulator.value?.id === id) {
      currentSimulator.value = updated
    }
    return updated
  }

  async function deleteModbusRegister(id: string, data: DeleteModbusRegisterRequest) {
    const updated = await simulatorApi.deleteModbusRegister(id, data)
    updateSimulatorInList(updated)
    if (currentSimulator.value?.id === id) {
      currentSimulator.value = updated
    }
    return updated
  }

  async function updateModbusRegisterValue(id: string, data: UpdateModbusRegisterRequest) {
    const updated = await simulatorApi.updateModbusRegisterValue(id, data)
    updateSimulatorInList(updated)
    if (currentSimulator.value?.id === id) {
      currentSimulator.value = updated
    }
    return updated
  }

  async function batchUpdateModbusRegisters(id: string, data: BatchUpdateModbusRegistersRequest) {
    const updated = await simulatorApi.batchUpdateModbusRegisters(id, data)
    updateSimulatorInList(updated)
    if (currentSimulator.value?.id === id) {
      currentSimulator.value = updated
    }
    return updated
  }

  return {
    // 状态
    simulators,
    protocols,
    loading,
    currentSimulator,
    modbusSlaves,
    // 计算属性
    runningCount,
    stoppedCount,
    totalConnections,
    // 方法
    fetchSimulators,
    fetchProtocols,
    fetchSimulator,
    refreshSimulatorSilently,
    createSimulator,
    deleteSimulator,
    startSimulator,
    stopSimulator,
    setOnline,
    triggerFault,
    clearFault,
    // Modbus 方法
    fetchModbusSlaves,
    addModbusSlave,
    deleteModbusSlave,
    setModbusRegister,
    deleteModbusRegister,
    updateModbusRegisterValue,
    batchUpdateModbusRegisters,
  }
})
