<template>
  <el-dialog
    title="创建模拟器"
    :model-value="visible"
    @update:model-value="emit('update:visible', $event)"
    width="500px"
  >
    <el-form
      ref="formRef"
      :model="form"
      :rules="rules"
      label-width="100px"
    >
      <el-form-item label="名称" prop="name">
        <el-input v-model="form.name" placeholder="请输入模拟器名称" />
      </el-form-item>

      <el-form-item label="协议" prop="protocol">
        <el-select v-model="form.protocol" placeholder="请选择协议" style="width: 100%">
          <el-option
            v-for="p in protocols"
            :key="p.name"
            :label="`${p.name} - ${p.description}`"
            :value="p.name"
          />
        </el-select>
      </el-form-item>

      <el-form-item label="绑定地址" prop="bind_addr">
        <el-input v-model="form.bind_addr" placeholder="0.0.0.0" />
      </el-form-item>

      <el-form-item label="端口" prop="port">
        <el-input-number
          v-model="form.port"
          :min="1"
          :max="65535"
          style="width: 100%"
        />
      </el-form-item>

      <el-form-item label="自动启动">
        <el-switch v-model="form.auto_start" />
      </el-form-item>
    </el-form>

    <template #footer>
      <el-button @click="emit('update:visible', false)">取消</el-button>
      <el-button type="primary" :loading="loading" @click="handleSubmit">
        创建
      </el-button>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { ref, reactive, watch } from 'vue'
import type { FormInstance, FormRules } from 'element-plus'
import { useSimulatorStore } from '@/stores/simulator'
import type { ProtocolInfo, CreateSimulatorRequest } from '@/types/simulator'

const props = defineProps<{
  visible: boolean
  protocols: ProtocolInfo[]
}>()

const emit = defineEmits<{
  'update:visible': [value: boolean]
  'created': []
}>()

const store = useSimulatorStore()
const formRef = ref<FormInstance>()
const loading = ref(false)

const form = reactive<CreateSimulatorRequest>({
  name: '',
  protocol: '',
  bind_addr: '0.0.0.0',
  port: 5000,
  auto_start: true,
})

const rules: FormRules = {
  name: [{ required: true, message: '请输入名称', trigger: 'blur' }],
  protocol: [{ required: true, message: '请选择协议', trigger: 'change' }],
  port: [{ required: true, message: '请输入端口', trigger: 'blur' }],
}

// 监听协议变化，自动设置默认端口
watch(() => form.protocol, (protocol) => {
  const p = props.protocols.find(p => p.name === protocol)
  if (p) {
    form.port = p.default_port
  }
})

// 对话框关闭时重置表单
watch(() => props.visible, (visible) => {
  if (!visible) {
    formRef.value?.resetFields()
  }
})

async function handleSubmit() {
  try {
    await formRef.value?.validate()
    loading.value = true
    await store.createSimulator(form)
    emit('created')
  } catch {
    // 验证失败或请求失败
  } finally {
    loading.value = false
  }
}
</script>
