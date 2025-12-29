import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { SimulatorInfo, ProtocolInfo } from '@/types/simulator'
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

  return {
    // 状态
    simulators,
    protocols,
    loading,
    currentSimulator,
    // 计算属性
    runningCount,
    stoppedCount,
    totalConnections,
    // 方法
    fetchSimulators,
    fetchProtocols,
    fetchSimulator,
    createSimulator,
    deleteSimulator,
    startSimulator,
    stopSimulator,
    setOnline,
    triggerFault,
    clearFault,
  }
})
