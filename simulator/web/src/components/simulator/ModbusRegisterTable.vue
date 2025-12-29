<template>
  <div class="register-table">
    <el-table :data="sortedRegisters" v-show="registers.length > 0" :row-key="getRowKey">
      <el-table-column prop="address" label="地址" width="100">
        <template #default="{ row }">
          <code class="address-code">{{ formatAddress(row.address) }}</code>
        </template>
      </el-table-column>

      <el-table-column prop="name" label="名称" min-width="150">
        <template #default="{ row }">
          {{ row.name || '-' }}
        </template>
      </el-table-column>

      <el-table-column prop="dataType" label="数据类型" width="100">
        <template #default="{ row }">
          <el-tag size="small" :type="getDataTypeTagType(row.dataType)">
            {{ formatDataType(row.dataType) }}
          </el-tag>
        </template>
      </el-table-column>

      <el-table-column label="生成模式" width="100">
        <template #default="{ row }">
          <el-tag v-if="row.generator" size="small" :type="getGeneratorTagType(row.generator.mode)">
            {{ formatGeneratorMode(row.generator.mode) }}
          </el-tag>
          <span v-else class="text-muted">固定</span>
        </template>
      </el-table-column>

      <el-table-column label="当前值" width="180">
        <template #default="{ row }">
          <!-- Bit 类型使用开关 -->
          <template v-if="isBitType(row)">
            <el-switch :model-value="!!row.value" :disabled="isReadonly"
              @change="(val: boolean) => emit('valueChange', row, val)" active-text="ON" inactive-text="OFF"
              inline-prompt />
          </template>
          <!-- 数值类型使用输入框 -->
          <template v-else>
            <el-input-number :model-value="row.value as number" :disabled="isReadonly"
              :precision="row.dataType === 'float32' ? 4 : 0" size="small" controls-position="right"
              @change="(val: number) => emit('valueChange', row, val ?? 0)" />
          </template>
        </template>
      </el-table-column>

      <el-table-column label="操作" width="120" fixed="right">
        <template #default="{ row }">
          <div class="action-buttons">
            <el-button text type="primary" @click="emit('edit', row)">
              <el-icon>
                <Edit />
              </el-icon>
            </el-button>
            <el-button text type="danger" @click="emit('delete', row)">
              <el-icon>
                <Delete />
              </el-icon>
            </el-button>
          </div>
        </template>
      </el-table-column>
    </el-table>

    <el-empty v-show="registers.length === 0" :description="`暂无${registerTypeLabel}，点击上方添加`" />
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { Edit, Delete } from '@element-plus/icons-vue'
import type { ModbusRegisterConfig, ModbusRegisterType, ModbusDataType, GeneratorMode } from '@/types/simulator'

interface Props {
  registers: ModbusRegisterConfig[]
  registerType: ModbusRegisterType
}

const props = defineProps<Props>()

const emit = defineEmits<{
  edit: [register: ModbusRegisterConfig]
  delete: [register: ModbusRegisterConfig]
  valueChange: [register: ModbusRegisterConfig, value: number | boolean]
}>()

const sortedRegisters = computed(() =>
  [...props.registers].sort((a, b) => a.address - b.address)
)

const isReadonly = computed(() =>
  props.registerType === 'discrete_input' || props.registerType === 'input_register'
)

const registerTypeLabel = computed(() => {
  const labels: Record<ModbusRegisterType, string> = {
    coil: '线圈',
    discrete_input: '离散输入',
    holding_register: '保持寄存器',
    input_register: '输入寄存器',
  }
  return labels[props.registerType]
})

function isBitType(register: ModbusRegisterConfig): boolean {
  return register.dataType === 'bit'
}

function formatAddress(address: number): string {
  return address.toString().padStart(5, '0')
}

function getRowKey(row: ModbusRegisterConfig): string {
  return `${row.type}-${row.address}`
}

function formatDataType(dataType: ModbusDataType): string {
  const labels: Record<ModbusDataType, string> = {
    bit: 'Bit',
    uint16: 'UInt16',
    int16: 'Int16',
    uint32: 'UInt32',
    int32: 'Int32',
    float32: 'Float32',
  }
  return labels[dataType]
}

function getDataTypeTagType(dataType: ModbusDataType): 'primary' | 'success' | 'warning' | 'info' {
  switch (dataType) {
    case 'bit':
      return 'info'
    case 'uint16':
    case 'int16':
      return 'primary'
    case 'uint32':
    case 'int32':
      return 'warning'
    case 'float32':
      return 'success'
    default:
      return 'info'
  }
}

function formatGeneratorMode(mode: GeneratorMode): string {
  const labels: Record<GeneratorMode, string> = {
    fixed: '固定',
    random: '随机',
    increment: '递增',
    decrement: '递减',
    sine: '正弦',
    toggle: '切换',
    sequence: '序列',
  }
  return labels[mode] || mode
}

function getGeneratorTagType(mode: GeneratorMode): 'primary' | 'success' | 'warning' | 'info' | 'danger' {
  switch (mode) {
    case 'random':
      return 'warning'
    case 'increment':
    case 'decrement':
      return 'primary'
    case 'sine':
      return 'success'
    case 'toggle':
      return 'danger'
    case 'sequence':
      return 'info'
    default:
      return 'info'
  }
}
</script>

<style lang="scss" scoped>
.register-table {
  .address-code {
    font-family: 'JetBrains Mono', monospace;
    font-size: 12px;
    background: var(--bg-input);
    padding: 2px 6px;
    border-radius: 4px;
    color: var(--text-secondary);
  }

  .text-muted {
    color: var(--text-muted);
    font-size: 12px;
  }

  .action-buttons {
    display: flex;
    gap: 4px;
  }
}
</style>
