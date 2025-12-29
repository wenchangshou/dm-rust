<template>
  <el-dialog
    v-model="visible"
    :title="isEdit ? '编辑寄存器' : '添加寄存器'"
    width="600px"
    @close="handleClose"
  >
    <el-form
      ref="formRef"
      :model="form"
      :rules="rules"
      label-width="100px"
    >
      <el-form-item label="寄存器类型" prop="type">
        <el-select v-model="form.type" placeholder="选择寄存器类型" style="width: 100%">
          <el-option label="线圈 (Coil)" value="coil" />
          <el-option label="离散输入 (Discrete Input)" value="discrete_input" />
          <el-option label="保持寄存器 (Holding)" value="holding_register" />
          <el-option label="输入寄存器 (Input)" value="input_register" />
        </el-select>
      </el-form-item>

      <el-form-item label="地址" prop="address">
        <el-input-number
          v-model="form.address"
          :min="0"
          :max="65535"
          style="width: 100%"
          placeholder="0-65535"
        />
      </el-form-item>

      <el-form-item label="数据类型" prop="dataType" v-if="showDataType">
        <el-select v-model="form.dataType" placeholder="选择数据类型" style="width: 100%">
          <el-option label="无符号16位 (uint16)" value="uint16" />
          <el-option label="有符号16位 (int16)" value="int16" />
          <el-option label="无符号32位 (uint32)" value="uint32" />
          <el-option label="有符号32位 (int32)" value="int32" />
          <el-option label="32位浮点 (float32)" value="float32" />
        </el-select>
      </el-form-item>

      <el-form-item label="名称" prop="name">
        <el-input v-model="form.name" placeholder="可选，寄存器描述" />
      </el-form-item>

      <el-form-item label="初始值" prop="value">
        <el-switch
          v-if="isBitType"
          v-model="boolValue"
          active-text="ON"
          inactive-text="OFF"
        />
        <el-input-number
          v-else
          v-model="numericValue"
          :min="minValue"
          :max="maxValue"
          :precision="isFloatType ? 4 : 0"
          style="width: 100%"
        />
      </el-form-item>

      <el-divider content-position="left">值生成器（可选）</el-divider>

      <el-form-item label="生成模式">
        <el-select v-model="generatorMode" placeholder="选择生成模式" style="width: 100%">
          <el-option label="固定值" value="fixed" />
          <el-option label="随机值" value="random" />
          <el-option label="递增" value="increment" />
          <el-option label="递减" value="decrement" />
          <el-option label="正弦波" value="sine" />
          <el-option v-if="isBitType" label="开关切换" value="toggle" />
          <el-option label="序列循环" value="sequence" />
        </el-select>
      </el-form-item>

      <!-- 随机/递增/递减/正弦 共用参数 -->
      <template v-if="showRangeParams">
        <el-form-item label="最小值">
          <el-input-number
            v-model="generator.min"
            :precision="isFloatType ? 4 : 0"
            style="width: 100%"
          />
        </el-form-item>
        <el-form-item label="最大值">
          <el-input-number
            v-model="generator.max"
            :precision="isFloatType ? 4 : 0"
            style="width: 100%"
          />
        </el-form-item>
      </template>

      <!-- 递增/递减 步长 -->
      <el-form-item v-if="showStepParam" label="步长">
        <el-input-number
          v-model="generator.step"
          :min="0.001"
          :precision="isFloatType ? 4 : 0"
          style="width: 100%"
        />
      </el-form-item>

      <!-- 正弦/切换 周期 -->
      <el-form-item v-if="showPeriodParam" label="周期(ms)">
        <el-input-number
          v-model="generator.period"
          :min="100"
          :step="100"
          style="width: 100%"
        />
      </el-form-item>

      <!-- 序列值 -->
      <el-form-item v-if="generatorMode === 'sequence'" label="序列值">
        <el-input
          v-model="sequenceInput"
          placeholder="用逗号分隔，如: 0, 10, 20, 30"
          style="width: 100%"
        />
      </el-form-item>

      <!-- 更新间隔 (所有非固定模式) -->
      <el-form-item v-if="generatorMode !== 'fixed'" label="更新间隔(ms)">
        <el-input-number
          v-model="generator.interval"
          :min="100"
          :step="100"
          style="width: 100%"
        />
      </el-form-item>
    </el-form>

    <template #footer>
      <el-button @click="handleClose">取消</el-button>
      <el-button type="primary" @click="handleSubmit" :loading="loading">
        {{ isEdit ? '保存' : '添加' }}
      </el-button>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { ref, computed, watch, reactive } from 'vue'
import type { FormInstance, FormRules } from 'element-plus'
import type { ModbusRegisterConfig, ModbusRegisterType, ModbusDataType, GeneratorMode, GeneratorConfig } from '@/types/simulator'

interface Props {
  modelValue: boolean
  register?: ModbusRegisterConfig | null
  loading?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  register: null,
  loading: false,
})

const emit = defineEmits<{
  'update:modelValue': [value: boolean]
  'submit': [register: ModbusRegisterConfig]
}>()

const visible = computed({
  get: () => props.modelValue,
  set: (val) => emit('update:modelValue', val),
})

const isEdit = computed(() => !!props.register)

const formRef = ref<FormInstance>()

const form = ref({
  type: 'holding_register' as ModbusRegisterType,
  address: 0,
  dataType: 'uint16' as ModbusDataType,
  name: '',
})

const boolValue = ref(false)
const numericValue = ref(0)

// 生成器相关
const generatorMode = ref<GeneratorMode>('fixed')
const generator = reactive({
  min: 0,
  max: 100,
  step: 1,
  period: 1000,
  interval: 1000,
})
const sequenceInput = ref('')

const isBitType = computed(() =>
  form.value.type === 'coil' || form.value.type === 'discrete_input'
)

const isFloatType = computed(() => form.value.dataType === 'float32')

const showDataType = computed(() =>
  form.value.type === 'holding_register' || form.value.type === 'input_register'
)

const showRangeParams = computed(() =>
  ['random', 'increment', 'decrement', 'sine'].includes(generatorMode.value)
)

const showStepParam = computed(() =>
  ['increment', 'decrement'].includes(generatorMode.value)
)

const showPeriodParam = computed(() =>
  ['sine', 'toggle'].includes(generatorMode.value)
)

const minValue = computed(() => {
  switch (form.value.dataType) {
    case 'int16':
      return -32768
    case 'int32':
      return -2147483648
    case 'float32':
      return -3.4e38
    default:
      return 0
  }
})

const maxValue = computed(() => {
  switch (form.value.dataType) {
    case 'uint16':
      return 65535
    case 'int16':
      return 32767
    case 'uint32':
      return 4294967295
    case 'int32':
      return 2147483647
    case 'float32':
      return 3.4e38
    default:
      return 65535
  }
})

const rules: FormRules = {
  type: [{ required: true, message: '请选择寄存器类型', trigger: 'change' }],
  address: [{ required: true, message: '请输入寄存器地址', trigger: 'blur' }],
  dataType: [{ required: true, message: '请选择数据类型', trigger: 'change' }],
}

watch(() => props.register, (reg) => {
  if (reg) {
    form.value.type = reg.type
    form.value.address = reg.address
    form.value.dataType = reg.dataType
    form.value.name = reg.name || ''
    if (typeof reg.value === 'boolean') {
      boolValue.value = reg.value
    } else {
      numericValue.value = reg.value as number
    }
    // 加载生成器配置
    if (reg.generator) {
      generatorMode.value = reg.generator.mode
      generator.min = reg.generator.min ?? 0
      generator.max = reg.generator.max ?? 100
      generator.step = reg.generator.step ?? 1
      generator.period = reg.generator.period ?? 1000
      generator.interval = reg.generator.interval
      if (reg.generator.sequence) {
        sequenceInput.value = reg.generator.sequence.join(', ')
      }
    } else {
      resetGenerator()
    }
  } else {
    resetForm()
  }
}, { immediate: true })

watch(() => form.value.type, (type) => {
  if (type === 'coil' || type === 'discrete_input') {
    form.value.dataType = 'bit'
  } else if (form.value.dataType === 'bit') {
    form.value.dataType = 'uint16'
  }
})

function resetGenerator() {
  generatorMode.value = 'fixed'
  generator.min = 0
  generator.max = 100
  generator.step = 1
  generator.period = 1000
  generator.interval = 1000
  sequenceInput.value = ''
}

function resetForm() {
  form.value = {
    type: 'holding_register',
    address: 0,
    dataType: 'uint16',
    name: '',
  }
  boolValue.value = false
  numericValue.value = 0
  resetGenerator()
}

function handleClose() {
  visible.value = false
  formRef.value?.resetFields()
  resetForm()
}

function buildGeneratorConfig(): GeneratorConfig | undefined {
  if (generatorMode.value === 'fixed') {
    return undefined
  }

  const config: GeneratorConfig = {
    mode: generatorMode.value,
    interval: generator.interval,
  }

  if (showRangeParams.value) {
    config.min = generator.min
    config.max = generator.max
  }

  if (showStepParam.value) {
    config.step = generator.step
  }

  if (showPeriodParam.value) {
    config.period = generator.period
  }

  if (generatorMode.value === 'sequence' && sequenceInput.value) {
    config.sequence = sequenceInput.value
      .split(',')
      .map(s => parseFloat(s.trim()))
      .filter(n => !isNaN(n))
  }

  return config
}

async function handleSubmit() {
  if (!formRef.value) return

  await formRef.value.validate()

  const register: ModbusRegisterConfig = {
    type: form.value.type,
    address: form.value.address,
    dataType: isBitType.value ? 'bit' : form.value.dataType,
    name: form.value.name || undefined,
    value: isBitType.value ? boolValue.value : numericValue.value,
    readonly: form.value.type === 'discrete_input' || form.value.type === 'input_register',
    generator: buildGeneratorConfig(),
  }

  emit('submit', register)
}
</script>
